pub mod data_models;

use anyhow::Result;
use std::{io::Write, collections::HashMap, cell::{RefCell, Cell}, rc::Rc, ops::Add};
use crate::data_models::*;

pub trait NodeHandler {
    /// This is called once when this instance is passed to `NodeRunner`'s `assign_handler()` method.
    fn init(&mut self, node_id: NodeId, node_ids:Vec<NodeId>);

    /// This is called any time a message is received for the 'NodeType' passed to the `assign_handler()` method.
    fn handle_msg(&mut self, msg: NodeMessage, runner: &NodeRunner) -> Option<Vec<NodeMessage>>;
}


#[derive(Default)]
pub struct NodeRunner<'a> {
    
    /// NodeId of this process's node.
    node_id: NodeId,

    /// NodeId's of all the nodes in our 'network'
    node_ids: Vec<NodeId>,

    // tracks the current 'next message id'
    next_msg_id: Cell<MsgId>,

    handlers: HashMap<String, Rc<RefCell<&'a mut dyn NodeHandler>>>
}

impl<'a> NodeRunner<'a> {

    /// create a new runner instance that initializes with the 'node id' for this process
    /// 
    /// (ie automatically handles the one-time 'init' message)
    pub fn new() -> Self {
        let mut runner = NodeRunner::default();

        let mut input = std::io::stdin().lines();
        let mut output = std::io::stdout().lock();

        // handle init
        let init_msg = input.next().expect("no init message!").expect("bad init message!");
        let init_reply = runner.handle_init(init_msg);
        output.write_all(init_reply.as_bytes()).expect("unable to write init reply!");
        output.flush().expect("unable to flush output!");
        
        drop(input);
        drop(output);

        runner
    }

    /// this should be called after `new()` and before `run_node()`.
    /// 
    /// This sets up a mapping to message type -> handlers.
    pub fn assign_handler<T: NodeHandler>(&mut self, handler: &'a mut T, for_types: &[NodeType]) {
        handler.init(self.node_id.clone(), self.node_ids.clone());

        let handler_ref = Rc::new(RefCell::new(handler as &mut dyn NodeHandler));
        for node_type in for_types {
            self.handlers.insert(node_type.to_string(), handler_ref.clone());
        }
    }

    /// runs the 'main loop' where stdin is read line-by-line and passed to the 'handler' set via the `assign_handler()` method
    pub fn run_node(&mut self) -> Result<()> {

        let mut input = std::io::stdin().lines();
        let mut output = std::io::stdout().lock();

        // now get on to the run loop:
        while let Some(Ok(next_line)) = input.next() {
            let next_msg = serde_json::from_str::<NodeMessage>(next_line.as_str()).expect("unable to deserialize msg");

            eprintln!("received msg:  {:?}", next_msg);

            let msg_type: NodeType;
            match &next_msg.body {
                // echo messages
                Body::Echo { msg_id: _, echo: _ } => msg_type = NodeType::Echo,
                Body::EchoOk { msg_id: _, in_reply_to: _, echo: _ } => {
                    eprintln!("received echo_ok: {:?}", next_msg);
                    continue;
                },

                // generate messages
                Body::Generate { msg_id: _ } => msg_type = NodeType::Generate,
                Body::GenerateOk { msg_id: _, id: _, in_reply_to: _ } => {
                    eprintln!("received generate_ok: {:?}", next_msg);
                    continue;
                }
                
                // broadcast messages
                Body::Topology { msg_id: _, topology: _ } => msg_type = NodeType::Broadcast,
                Body::TopologyOk { msg_id: _, in_reply_to: _ } => {
                    eprintln!("received topology_ok: {:?}", next_msg);
                    continue;
                }
                Body::Broadcast { msg_id: _, message: _ } => msg_type = NodeType::Broadcast,
                Body::BroadcastOk { msg_id: _, in_reply_to: _ } => {
                    eprintln!("received broadcast_ok: {:?}", next_msg);
                    continue;
                }
                Body::Read { msg_id: _ } => msg_type = NodeType::Broadcast,
                Body::ReadOk { msg_id: _, in_reply_to: _, messages: _ } => {
                    eprintln!("received read_ok: {:?}", next_msg);
                    continue;
                }
            }

            let key = msg_type.to_string();
            if let Some(handler_rc) = self.handlers.get(&key) {
                let mut handler = handler_rc.borrow_mut();
                
                let responses = handler.handle_msg(next_msg, self);
                if responses.is_none() {
                    eprintln!("no response for current message");
                    continue;
                }

                for resp in responses.unwrap().iter() {
                    let reply = serde_json::to_string(&resp).expect("couldnt serialize response");
                    output.write_all(reply.add("\n").as_bytes())?;
                    output.flush()?;
                }

            } else {
                eprintln!("no handler for message type: {}", key);
            }
        }

        eprintln!("processed all messages, exiting successfully");

        Ok(())
    }

    pub fn get_next_msg_id(&self) -> MsgId {
        let next_id = self.next_msg_id.get().wrapping_add(1);
        self.next_msg_id.set(next_id);

        next_id
    }

    fn handle_init(&mut self, init_msg: String) -> String {
        eprintln!("received init: \n{:?}", init_msg);
        
        let msg = serde_json::from_str::<InitMessage>(init_msg.as_str()).expect("unable to parse init msg");

        let response: InitMessage;
        match msg.body {
            InitBody::Init { msg_id, node_id, node_ids: _ } => {
                self.node_id = node_id;
                response = InitMessage {
                    id:     0,
                    src:    self.node_id.clone(),
                    dest:   msg.src.clone(),
                    body:   InitBody::InitOk { in_reply_to: msg_id },
                }
            },
            InitBody::InitOk { in_reply_to: _ } => panic!("unexpected init_ok msg!"),
        };

        
        let reply = serde_json::to_string(&response).expect("couldnt serialize init_ok response");
        
        eprintln!("prepared init reply: \n{:?}", reply);
        reply.add("\n")
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runner_handles_init_msg() {
        let mut runner = NodeRunner::default();

        let response = runner.handle_init("{\"src\":\"c1\",\"dest\":\"n3\",\"body\":{\"type\":\"init\",\"msg_id\":1,\"node_id\":\"n3\",\"node_ids\":[\"n1\",\"n2\",\"n3\"]}}".to_string());

        eprintln!("init response: {}", response);

        assert!(runner.node_id == "n3".to_string());
    }
}
