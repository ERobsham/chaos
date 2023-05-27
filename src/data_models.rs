use std::{fmt::Display, collections::{HashMap, HashSet}};
use serde::{Serialize, Deserialize};

pub type MsgId = usize;
pub type NodeId = String;

#[derive(Deserialize, Serialize, Debug)]
pub struct NodeMessage {
    pub src: NodeId,
    pub dest: NodeId,
    pub body: Body,
}

impl NodeMessage {
    pub(crate) fn as_node_type(&self) -> Option<NodeType> {
        let msg_type: NodeType;
        match &self.body {
            // echo messages
            Body::Echo { msg_id: _, echo: _ } => msg_type = NodeType::Echo,

            // generate messages
            Body::Generate { msg_id: _ } => msg_type = NodeType::Generate,
            
            // broadcast messages
            Body::Topology { msg_id: _, topology: _ } => msg_type = NodeType::Broadcast,
            Body::Broadcast { msg_id: _, message: _ } => msg_type = NodeType::Broadcast,
            Body::Read { msg_id: _ } => msg_type = NodeType::Broadcast,
            Body::ReadOk { msg_id: _, in_reply_to: _, messages: _ } => msg_type = NodeType::Broadcast,

            _ => return None,
        }
        
        Some(msg_type)
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Body {
    // Echo types
    Echo { 
        msg_id: MsgId, 
        echo: String,
    },
    EchoOk {
        msg_id: MsgId,
        in_reply_to: MsgId,
        echo: String,
     },
     
     // Generate Unique ID
     Generate { 
         msg_id: MsgId, 
     },
     GenerateOk {
         msg_id: MsgId,
         in_reply_to: MsgId,
         id: String,
      },

     // Broadcast Workload :
     // - Topology / TopologyOk
     // - Broadcast / BroadcastOk
     // - Read / ReadOk
     Topology { 
         msg_id: MsgId,
         topology: HashMap<NodeId, Vec<NodeId>>,
     },
     TopologyOk {
         msg_id: MsgId,
         in_reply_to: MsgId,
      },
     Broadcast { 
         msg_id: MsgId, 
         message: usize,
     },
     BroadcastOk {
         msg_id: MsgId,
         in_reply_to: MsgId,
      },
     Read { 
         msg_id: MsgId,
     },
     ReadOk {
         msg_id: MsgId,
         in_reply_to: MsgId,
         messages: HashSet<usize>,
      },
     // ... TODO: fill in the rest of the types.
}


impl Body {
    pub fn set_msg_id(&mut self, new_id: MsgId) {
        match self {
            Body::Echo { msg_id, echo: _ } => 
                *msg_id = new_id,
            Body::EchoOk { msg_id, in_reply_to: _, echo: _ } => 
                *msg_id = new_id,
            Body::Generate { msg_id } => 
                *msg_id = new_id,
            Body::GenerateOk { msg_id, in_reply_to: _, id: _ } => 
                *msg_id = new_id,
            Body::Topology { msg_id, topology: _ } => 
                *msg_id = new_id,
            Body::TopologyOk { msg_id, in_reply_to: _ } => 
                *msg_id = new_id,
            Body::Broadcast { msg_id, message: _ } => 
                *msg_id = new_id,
            Body::BroadcastOk { msg_id, in_reply_to: _ } => 
                *msg_id = new_id,
            Body::Read { msg_id } => 
                *msg_id = new_id,
            Body::ReadOk { msg_id, in_reply_to: _, messages: _ } => 
                *msg_id = new_id,
        }
    }
}


pub type Workload = String;

pub enum NodeType {
    Echo,
    Generate,
    Broadcast,
    // ... TODO: fill in the rest of the types.
}

impl Display for NodeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NodeType::Echo => write!(f, "echo"),
            NodeType::Generate => write!(f, "generate"),
            NodeType::Broadcast => write!(f, "broadcast"),

            // ... TODO: fill in the rest of the types.
        }
    }
}
