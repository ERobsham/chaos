use std::{fmt::Display, collections::{HashMap, HashSet}};
use serde::{Serialize, Deserialize};

pub type MsgId = usize;
pub type NodeId = String;

#[derive(Deserialize, Serialize, Debug)]
pub struct NodeMessage {
    // meta data maelstrom must use?  might be helpful to see in some responses.
    #[serde(skip_serializing, default)]
    pub id: usize,

    pub src: NodeId,
    pub dest: NodeId,
    pub body: Body,
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
