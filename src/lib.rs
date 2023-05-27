pub mod data_models;
pub mod io;
mod init;

use anyhow::{Result, anyhow};
use init::InitBody;
use io::{StdinSource, StdoutSink};
use std::{io::Write, collections::HashMap, cell::{RefCell, Cell}, rc::Rc, ops::Add, time::{Duration, Instant}};

use crate::data_models::*;


type Tag = String;

pub trait NodeHandler {
    /// This is called once when this instance is passed to `NodeRunner`'s `assign_handler()` method.
    fn init(&mut self, node_id: NodeId, node_ids:Vec<NodeId>);

    /// This is called any time a message is received for the 'NodeType' passed to the `assign_handler()` method.
    fn handle_msg(&mut self, msg: NodeMessage, runner: &NodeRunner) -> Option<Vec<NodeMessage>>;

    /// This is called anytime a registered interval is triggered.  
    ///   -- `tag` is the tag associated with the interval when the interval was registered.
    ///   -- `elapsed` is the duration elapsed since the `NodeRunner` started `run_node()`
    fn handle_interval(&mut self, tag: Tag, elapsed: Duration, runner: &NodeRunner);
}


#[derive(Default)]
pub struct NodeRunner<'a> {
    
    /// NodeId of this process
    node_id: NodeId,
    
    /// NodeId's of all the nodes in our 'network'
    node_ids: Vec<NodeId>,
    
    // tracks the current 'next message id'
    next_msg_id: Cell<MsgId>,
    
    running: bool,
    start_time: Option<Instant>,

    handlers: HashMap<Workload, Rc<RefCell<&'a mut dyn NodeHandler>>>,
    intervals: HashMap<Tag, Duration>,

    msg_source: StdinSource,
    msg_sink: StdoutSink,
}

impl<'a> NodeRunner<'a> {

    /// create a new runner instance that initializes with the 'node id' for this process
    /// 
    /// (ie automatically handles the one-time 'init' message)
    pub fn new() -> Self {
        if let InitBody::Init { msg_id: _, node_id, node_ids } = init::handle_init() {
            return NodeRunner {
                node_id,
                node_ids,
                ..Default::default()
            }
        }
        unreachable!("we must receive an Init variant");
    }

    /// this should be called after `new()` and before `run_node()`.
    /// 
    /// This sets up a mapping to message type -> handlers.
    pub fn register_handler<T: NodeHandler>(&mut self, handler: &'a mut T, for_types: &[NodeType]) {
        if self.running { return; }
        
        handler.init(self.node_id.clone(), self.node_ids.clone());

        let handler_ref = Rc::new(RefCell::new(handler as &mut dyn NodeHandler));
        for node_type in for_types {
            self.handlers.insert(node_type.to_string(), handler_ref.clone());
        }
    }

    pub fn register_interval(&mut self, tag: Tag, interval: Duration) {
        self.intervals.insert(tag, interval);
    }

    /// runs the 'main loop' where stdin is read line-by-line and passed to the 'handler' set via the `assign_handler()` method
    pub async fn run_node(&mut self) -> Result<()> {
        self.running = true;
        self.start_time = Some(Instant::now());

        if self.handlers.len() == 0 { return Err(anyhow!("no handlers registered")); }

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
                Body::ReadOk { msg_id: _, in_reply_to: _, messages: _ } => msg_type = NodeType::Broadcast,
            }

            let key: Workload = msg_type.to_string();
            if let Some(handler_rc) = self.handlers.get(&key) {
                let mut handler = handler_rc.borrow_mut();
                
                let responses = handler.handle_msg(next_msg, self);
                if responses.is_none() {
                    eprintln!("no response for current message");
                    continue;
                }

                for resp in responses.unwrap().iter() {
                    eprintln!("sending msg: {:?}", resp);
                    let reply = serde_json::to_string(&resp).expect("couldnt serialize response");
                    output.write_all(reply.add("\n").as_bytes())?;
                    output.flush()?;
                }

            } else {
                eprintln!("no handler for workload: {}", key);
            }
        }

        eprintln!("processed all messages, exiting successfully");

        Ok(())
    }


    /// assigns the message the next available `msg_id`
    /// then handles sending it
    pub async fn send_msg(&self, mut msg: NodeMessage) {
        msg.body.set_msg_id(0);
        self.msg_sink.send_msg(msg).await;
    }

    /// NodeHandlers should use this to generate unique msg_ids for all their outgoing messages.
    pub fn get_next_msg_id(&self) -> MsgId {
        let next_id = self.next_msg_id.get().wrapping_add(1);
        self.next_msg_id.set(next_id);

        next_id
    }


    
}

