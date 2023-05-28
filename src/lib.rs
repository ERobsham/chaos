pub mod data_models;
pub mod io;
mod init;

use anyhow::{Result, anyhow};
use init::InitBody;
use io::{StdinSource, StdoutSink};
use tokio::{time, select, sync::mpsc};
use std::{collections::HashMap, cell::{RefCell, Cell}, rc::Rc, time::{Duration, Instant}};

use crate::data_models::*;


type Tag = String;

pub trait NodeHandler {
    /// This is called once when this instance is passed to `NodeRunner`'s `assign_handler()` method.
    fn init(&mut self, node_id: NodeId, node_ids:Vec<NodeId>);

    /// This is called any time a message is received for the 'NodeType' passed to the `assign_handler()` method.
    fn handle_msg(&mut self, msg: NodeMessage) -> Option<Vec<NodeMessage>>;

    /// This is called anytime a registered interval is triggered.  
    ///   -- `tag` is the tag associated with the interval when the interval was registered.
    ///   -- `elapsed` is the duration elapsed since the `NodeRunner` started `run_node()`
    fn handle_interval(&mut self, _tag: Tag, _elapsed: Duration) -> Option<Vec<NodeMessage>> {
        None
    }
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
    pub fn register_handler<T: NodeHandler>(&mut self, handler: &'a mut T, for_types: &[NodeType]) -> bool {
        if self.running { return false; }
        
        handler.init(self.node_id.clone(), self.node_ids.clone());

        let handler_ref = Rc::new(RefCell::new(handler as &mut dyn NodeHandler));
        for node_type in for_types {
            self.handlers.insert(node_type.to_string(), handler_ref.clone());
        }
        true
    }

    pub fn register_interval(&mut self, tag: Tag, interval: Duration) -> bool {
        if self.running { return false; }

        self.intervals.insert(tag, interval);
        true
    }

    /// runs the 'main loop' where stdin is read line-by-line and passed to the 'handler' set via the `assign_handler()` method
    pub async fn run_node(&mut self) -> Result<()> {
        self.running = true;
        self.start_time = Some(Instant::now());

        if self.handlers.len() == 0 { return Err(anyhow!("no handlers registered")); }
    
        // setup any 'intervals'
        let (int_tx, mut int_rx) = mpsc::channel(10);
        self.intervals
            .iter()
            .for_each(|(t, d)| { 
                let tx = int_tx.clone();
                let tag = t.clone();
                let dur = d.clone();
                tokio::task::spawn(async move { 
                    let mut interval = time::interval(dur);
                    let fut_tag = tag;

                    interval.tick().await;
                    loop {
                        let loop_tag = fut_tag.clone();
                        interval.tick().await;
                        let _ = tx.send(loop_tag).await;
                    }
                });
            });
        
        let (sig_tx, mut sig_rx) = mpsc::channel(10);
        ctrlc::set_handler(move || {
            let _ = sig_tx.blocking_send(());
        }).expect("should set SIGINT handler");


        loop {
            select! {
                msg = self.msg_source.next_msg() => {
                    // eprintln!("run_node dispatching msg:  {:?}", msg
                    if let Some(msg_type) = msg.as_node_type() {
                        let key: Workload = msg_type.to_string();
                        if let Some(handler_rc) = self.handlers.get(&key) {
                            let mut handler = handler_rc.borrow_mut();
                            
                            if let Some(responses) = handler.handle_msg(msg) {
                                self.send_msgs(responses).await;
                            }
                        } else {
                            eprintln!("no handler for workload: {}", key);
                        }
                    }
                },
                tag = int_rx.recv() => {
                    if let Some(tag) = tag {
                        for (_, handler_rc) in &self.handlers  {
                            let mut handler = handler_rc.borrow_mut();
                            let start_time = self.start_time.unwrap();
                            if let Some(msgs) = handler.handle_interval(tag.clone(), start_time.elapsed()) {
                                self.send_msgs(msgs).await;
                            }
                        }
                    }
                },
                _ = sig_rx.recv() => {
                    break
                },
            }
        }

        eprintln!("processed all messages, exiting successfully");

        Ok(())
    }


    /// assigns the message the next available `msg_id`
    /// then handles sending it
    async fn send_msgs(&self, msgs: Vec<NodeMessage>) {
        for mut msg in msgs {
            msg.body.set_msg_id(self.get_next_msg_id());
            self.msg_sink.send_msg(msg).await;
        }
    }

    /// NodeHandlers should use this to generate unique msg_ids for all their outgoing messages.
    fn get_next_msg_id(&self) -> MsgId {
        let next_id = self.next_msg_id.get().wrapping_add(1);
        self.next_msg_id.set(next_id);

        next_id
    }
    
}

