use std::collections::HashMap;

use anyhow::Result;
use chaos::{NodeRunner, NodeHandler, data_models::*};


pub fn main() -> Result<()>{

    // setup the initialized `NodeRunner`
    let mut node = NodeRunner::new();
    
    eprintln!("generating unique ids...");
    let mut handler = GeneratorNode::default();
    node.register_handler(&mut handler, &[ NodeType::Generate ]);

    node.run_node()?;
    eprintln!("completed generating unique ids");

    Ok(())
}

#[derive(Debug, Default)]
struct GeneratorNode {
    node_id: NodeId,
    id_map: HashMap<NodeId, usize>
}

impl GeneratorNode {
    fn generate_id(&mut self, node_id: &NodeId) -> String {
        let current_id_for_node = self.id_map.get(node_id).unwrap_or(&0).clone();
        self.id_map.insert(node_id.clone(), current_id_for_node+1);

        format!("{}-{}", node_id, current_id_for_node).to_string()
    }
}

impl NodeHandler for GeneratorNode {
    fn init(&mut self, node_id: NodeId, _node_ids:Vec<NodeId>) {
        self.node_id = node_id;
    }

    fn handle_msg(&mut self, msg: NodeMessage, runner: &NodeRunner) -> Option<Vec<NodeMessage>> {
        if let Body::Generate { msg_id } = msg.body {
            let resp_msg_id = runner.get_next_msg_id();
            let unique_id = self.generate_id(&msg.src);
            Some(vec![NodeMessage {
                id:0,
                src: self.node_id.clone(),
                dest: msg.src,
                body: Body::GenerateOk { msg_id: resp_msg_id, id: unique_id, in_reply_to: msg_id }
            }])
        } else {
            None
        }
    }

    fn handle_interval(&mut self, _tag: String, _elapsed: std::time::Duration, _runner: &NodeRunner) { }
}