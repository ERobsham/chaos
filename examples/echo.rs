use anyhow::Result;
use chaos::{NodeRunner, NodeHandler, data_models::*};

#[tokio::main]
pub async fn main() -> Result<()>{
    let mut node = NodeRunner::new();
    
    eprintln!("echoing...");

    let mut handler = EchoNode::default();
    node.register_handler(&mut handler, &[ NodeType::Echo ]);
    node.run_node().await?;

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

    fn handle_msg(&mut self, msg: NodeMessage, runner: &NodeRunner) -> Option<Vec<NodeMessage>> {
        if let Body::Echo { msg_id, echo } = msg.body {
            let resp_msg_id = runner.get_next_msg_id();    
            Some(vec![
                NodeMessage {
                    src: self.node_id.clone(),
                    dest: msg.src,
                    body: Body::EchoOk { msg_id: resp_msg_id, in_reply_to: msg_id, echo: echo }
                },
            ])
        } else {
            None
        }
    }

    fn handle_interval(&mut self, _tag: String, _elapsed: std::time::Duration, _runner: &NodeRunner) { }
}