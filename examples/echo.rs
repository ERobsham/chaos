use anyhow::Result;
use chaos::{NodeRunner, NodeHandler, data_models::*};


pub fn main() -> Result<()>{

    // setup the initialized `NodeRunner`
    let mut node = NodeRunner::new();
    
    eprintln!("echoing...");
    let mut handler = EchoNode::default();
    node.assign_handler(&mut handler, &[ NodeType::Echo ]);

    node.run_node()?;
    eprintln!("completed echo");

    Ok(())
}

#[derive(Debug, Default)]
struct EchoNode {
    node_id: NodeId,
}

impl NodeHandler for EchoNode {
    fn init(&mut self, node_id: NodeId, _node_ids:Vec<NodeId>) {
        self.node_id = node_id;
    }

    fn handle_msg(&mut self, msg: NodeMessage, runner: &NodeRunner) -> Option<Box<[NodeMessage]>> {
        if let Body::Echo { msg_id, echo } = msg.body {
            let resp_msg_id = runner.get_next_msg_id();    
            Some(Box::new([
                NodeMessage {
                    id:0,
                    src: self.node_id.clone(),
                    dest: msg.src,
                    body: Body::EchoOk { msg_id: resp_msg_id, in_reply_to: msg_id, echo: echo }
                },
            ]))
        } else {
            None
        }
    }
}