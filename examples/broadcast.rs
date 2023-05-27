
use anyhow::Result;
use rand::seq::SliceRandom;
use std::collections::{HashSet, HashMap};
use chaos::{NodeRunner, NodeHandler, data_models::*};

use tokio;


#[tokio::main]
pub async fn main() -> Result<()>{
    let mut node = NodeRunner::new();
    
    eprintln!("broadcasting...");
    
    let mut handler = BroadcastNode::default();
    node.register_handler(&mut handler, &[ NodeType::Broadcast ]);
    node.run_node().await?;

    eprintln!("completed broadcasting");
    
    Ok(())
}

const MIN_WIDOWING_SIZE:usize = 5;

#[derive(Debug, Default)]
struct BroadcastNode {
    node_id: NodeId,
    neighbors: Vec<NodeId>,

    neighbors_known_msgs: HashMap<NodeId, HashSet<usize>>,

    known_msgs: HashSet<usize>,
}

impl BroadcastNode {
    fn update_neighbors(&mut self, node_ids: Vec<NodeId>) {
        self.neighbors = node_ids;
    }
}

impl NodeHandler for BroadcastNode {
    fn init(&mut self, node_id: NodeId, node_ids:Vec<NodeId>) {
        self.node_id = node_id;
        self.update_neighbors(node_ids);
    }

    fn handle_msg(&mut self, msg: NodeMessage, runner: &NodeRunner) -> Option<Vec<NodeMessage>> {
        
        // ensure we always have a value for `msg.src` here
        // so we can indiscriminantly unwrap the .get(&msg.src) Option
        if !self.neighbors_known_msgs.contains_key(&msg.src) { 
            self.neighbors_known_msgs.insert(msg.src.clone(), HashSet::new());
        }

        match msg.body { 
        Body::Topology { msg_id, topology } => {
            if let Some(our_neighbors) = topology.get(&self.node_id) {
                self.update_neighbors(our_neighbors.clone());
            }
            
            let resp_msg_id = runner.get_next_msg_id();
            Some(vec![NodeMessage {
                src: self.node_id.clone(),
                dest: msg.src,
                body: Body::TopologyOk { 
                    msg_id: resp_msg_id, 
                    in_reply_to: msg_id 
                },
            }])
        },
        Body::Broadcast { msg_id, message } => { 

            // first, create the 'ok' response:
            let mut messages = vec![ 
                NodeMessage {
                    src: self.node_id.clone(), 
                    dest: msg.src.clone(),
                    body: Body::BroadcastOk { 
                        msg_id: runner.get_next_msg_id(), 
                        in_reply_to: msg_id 
                    }, 
                },
            ];

            // then all the broadcast messages:
            messages.extend(
                self.neighbors.iter()
                .map(|dest| {
                    NodeMessage {
                        src: self.node_id.clone(),
                        dest: dest.clone(),
                        body: Body::Broadcast {
                            msg_id:runner.get_next_msg_id(), 
                            message: message.clone(),
                        },
                    }
                })
            );

            // now add this to the 'known' messages for the src node.
            let src_known = self.neighbors_known_msgs.get_mut(&msg.src).unwrap();
            src_known.insert(message);
            
            // finally, add this to our 'known' messages
            self.known_msgs.insert(message);

            Some(messages) 
        },
        Body::Read { msg_id } => {
            let resp_msg_id = runner.get_next_msg_id();

            // start by getting all the values we know the src node doesn't know 
            let src_known = self.neighbors_known_msgs.get(&msg.src).unwrap();
            let mut src_unknown: HashSet<_> = self.known_msgs.difference(src_known).copied().collect();


            // Now extend that list with an extra set of values, 
            // randomly selected from the list of all our known values. 
            let list:Vec<_> = self.known_msgs.iter().copied().collect();
            let mut window_size = MIN_WIDOWING_SIZE.max(list.len()/5);
            window_size = window_size.min(list.len());

            let mut rng = rand::thread_rng();
            let extras = list.choose_multiple(&mut rng, window_size).cloned();

            src_unknown.extend(extras);
            
            // `src_unknown` is unchanged at this point, and `extras` is not exhausted...???

            // Finally, construct the response
            Some(vec![NodeMessage {
                src: self.node_id.clone(),
                dest: msg.src,
                body: Body::ReadOk { 
                    msg_id: resp_msg_id, 
                    in_reply_to: msg_id, 
                    messages: src_unknown, 
                },
            }])
        },

        Body::ReadOk { msg_id: _, in_reply_to: _, messages } => {
            
            // keep track of what our peers know.
            let src_known = self.neighbors_known_msgs.get_mut(&msg.src).unwrap();
            src_known.extend(messages.clone());

            // and add this to what we know.
            self.known_msgs.extend(messages);

            None
        },

        // and we don't handle any other messages
        _ => None,
        }
    }

    fn handle_interval(&mut self, _tag: String, _elapsed: std::time::Duration, _runner: &NodeRunner) {

    }
}


#[cfg(test)]
mod broadcast_tests {
    use super::*;

    #[test]
    fn sends_broadcast() {
        let mut node = BroadcastNode::default();

        node.node_id = "n1".to_string();
        node.neighbors.push("c1".to_string());
        node.neighbors.push("c2".to_string());

        let msgs = node.handle_msg(
            NodeMessage {
                src: "c1".to_string(), 
                dest: "n1".to_string(), 
                body: Body::Broadcast { msg_id: 0, message: 1 },
            }, 
            &NodeRunner::default(),
        );

        match msgs {
            Some(msgs) => {
                match &msgs[0].body {
                    Body::BroadcastOk { msg_id:_, in_reply_to } => {
                        assert!(*in_reply_to == 0)
                    },
                    _ => assert!(false, "'broadcast' did not produce a 'broadcast_ok' message"),
                }

                eprintln!("messages:");
                msgs.iter().for_each(|msg| {
                    eprintln!("        {:?}", msg);
                });
            },
            None => assert!(false, "no response from 'Broadcast'!"),
        }
    }

    #[test]
    fn read_message_adds_extras() {
        let mut node = BroadcastNode::default();
        let mut known_set = HashSet::new();
        known_set.extend(0..100);

        node.node_id = "n1".to_string();
        node.neighbors.push("c1".to_string());
        node.known_msgs.extend(0..100);
        node.neighbors_known_msgs.insert("c1".to_string(), known_set);

        let msg = node.handle_msg(
            NodeMessage {
                src: "c1".to_string(), 
                dest: "n1".to_string(), 
                body: Body::Read { msg_id: 0 },
            }, 
            &NodeRunner::default(),
        );

        match msg {
            Some(msg) => {
                assert!(msg.len() == 1);
                match &msg[0].body {
                    Body::ReadOk { msg_id:_, in_reply_to:_, messages } => {
                        assert!(messages.len() == 20)
                    },
                    _ => assert!(false, "'read' did not produce a 'read_ok' message"),
                }
            },
            None => assert!(false, "no response from 'Read'!"),
        }
    }

}