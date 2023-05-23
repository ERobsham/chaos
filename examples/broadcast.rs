use anyhow::Result;
use chaos::{NodeRunner, NodeHandler, data_models::*};


pub fn main() -> Result<()>{

    // setup the initialized `NodeRunner`
    let mut node = NodeRunner::new();
    
    eprintln!("broadcasting...");
    let mut handler = BroadcastNode::default();
    node.assign_handler(&mut handler, &[ NodeType::Broadcast ]);

    node.run_node()?;
    eprintln!("completed broadcasting");

    Ok(())
}

#[derive(Debug, Default)]
struct BroadcastNode {
    node_id: NodeId,
    neighbors: Vec<NodeId>,

    known_msgs: Vec<usize>,
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
        match msg.body { 
        Body::Topology { msg_id, topology } => {
            if let Some(our_neighbors) = topology.get(&self.node_id) {
                self.update_neighbors(our_neighbors.clone());
            }
            
            let resp_msg_id = runner.get_next_msg_id();
            Some(vec![NodeMessage {
                id:0,
                src: self.node_id.clone(),
                dest: msg.src,
                body: Body::TopologyOk { 
                    msg_id: resp_msg_id, 
                    in_reply_to: msg_id 
                },
            }])
        },
        Body::Broadcast { msg_id, message } => { 

            // create the 'ok' response:
            let mut messages = vec![ 
                NodeMessage { 
                    id: 0, 
                    src: self.node_id.clone(), 
                    dest: msg.src, 
                    body: Body::BroadcastOk { 
                        msg_id: runner.get_next_msg_id(), 
                        in_reply_to: msg_id 
                    }, 
                },
            ];

            self.known_msgs.push(message.clone());
            
            // now all the broadcast messages:
            messages.extend(
                self.neighbors.iter()
                .map(|dest| { NodeMessage {
                        id:0,
                        src: self.node_id.clone(),
                        dest: dest.clone(),
                        body: Body::Broadcast { 
                            msg_id:runner.get_next_msg_id(), 
                            message: message.clone(), 
                        },
                    }
                })
            );

            Some(messages) 
        },
        Body::Read { msg_id } => {
            let resp_msg_id = runner.get_next_msg_id();

            // the trivial / naive implementation
            Some(vec![NodeMessage {
                id:0,
                src: self.node_id.clone(),
                dest: msg.src,
                body: Body::ReadOk { 
                    msg_id: resp_msg_id, 
                    in_reply_to: msg_id, 
                    messages: self.known_msgs.clone(), 
                },
            }])

            // we can (and should) do better if we memoize 
            // some of the data we know our neighbors already know.
        },

        // and we don't handle any other messages
        _ => None,
        }
    }
}