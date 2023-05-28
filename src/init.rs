use serde::{Serialize, Deserialize};
use std::{io::Write, ops::Add};

use crate::data_models::*;


//
// Internal models for the 'init' message.
//
// this will hide these details from any specific 'node' implementation, 
// as all nodes should utilize the 'NodeRunner'
//

#[derive(Deserialize, Serialize, Debug)]
pub(crate) struct InitMessage {
    pub(crate) src: NodeId,
    pub(crate) dest: NodeId,
    pub(crate) body: InitBody,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub(crate) enum InitBody {
    Init{
        msg_id:   MsgId,
        node_id:  NodeId,
        node_ids: Vec<NodeId>,
    },
    InitOk{
        in_reply_to:   MsgId,
    },
}

// ------------------------------------------------------------------------------------
// private handler for the common 'init' work.
//

/// Handles the common 'init' workload via stdin/stdout.
/// Returns the `node_id`
pub(crate) fn handle_init() -> InitBody {

    let mut input = std::io::stdin().lines();
    let line = input.next()
        .expect("input should have data")
        .expect("input should be a valid string");
    drop(input);

    let msg = serde_json::from_str::<InitMessage>(line.as_str())
        .expect("line should be a valid init message");
    eprintln!("received init: \n{:?}", msg);

    let response: InitMessage;
    let id: String;
    match &msg.body {
        InitBody::Init { msg_id, node_id, node_ids: _ } => {
            id = node_id.clone();
            response = InitMessage {
                src:    id.clone(),
                dest:   msg.src.clone(),
                body:   InitBody::InitOk { in_reply_to: msg_id.clone() },
            }
        },
        InitBody::InitOk { in_reply_to: _ } => unreachable!("should not be receiving init_ok msg as a node"),
    };

    
    eprintln!("prepared init response: \n{:?}", response);
    let mut reply = serde_json::to_string(&response)
        .expect("init_ok response should serialize");
    reply = reply.add("\n");

    let mut output = std::io::stdout().lock();
    output.write_all(reply.as_bytes())
        .expect("stdout should accept data");
    output.flush()
        .expect("stdout should flush data");
    drop(output);

    msg.body
}