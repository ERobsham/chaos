use crate::data_models::NodeMessage;

use std::{io::{self, Write}, thread::{self, JoinHandle}};
use tokio::sync::{oneshot, mpsc};


pub(crate) struct StdinSource {
    msg_rx: mpsc::Receiver<NodeMessage>,
}

impl StdinSource {
    pub fn new() -> Self {        
        let (tx, rx) = mpsc::channel(10);

        thread::spawn(move || {
            let mut input = io::stdin().lines();
            while let Some(Ok(line)) = input.next() {
                let next_msg = serde_json::from_str::<NodeMessage>(line.as_str()).expect("should deserialize to a NodeMessage");

                let _ = tx.blocking_send(next_msg).expect("should send NodeMessage via channel");
            }
        });

        Self {
            msg_rx: rx,
        }
    }

    pub async fn next_msg(&mut self) -> NodeMessage {
        self.msg_rx.recv().await.expect("should receive NodeMessage via channel")
    }
}

impl Default for StdinSource {
    fn default() -> Self { Self::new() }
}




pub(crate) struct StdoutSink {
    msg_tx: mpsc::Sender<NodeMessage>,
    cancel_tx: Option<oneshot::Sender<()>>,
    writer_handle: Option<JoinHandle<()>>,
}

impl StdoutSink {
    pub fn new() -> Self {        
        let (msg_tx, mut msg_rx) = mpsc::channel(10);
        let (cancel_tx, mut cancel_rx) = oneshot::channel();


        let handle =thread::spawn(move || {
            let mut output = io::stdout().lock();
            
            loop {
                match cancel_rx.try_recv() {
                    Err(oneshot::error::TryRecvError::Empty) => (),
                    _ => break,
                }
                
                match msg_rx.try_recv() {
                    Ok(msg) => {
                        let data = serde_json::to_string(&msg).expect("message should serialize");
                        output.write_all(data.as_bytes()).expect("stdout should accept data");
                        output.flush().expect("stdout should flush");
                        continue;
                    },
                    Err(mpsc::error::TryRecvError::Empty) => (),
                    Err(mpsc::error::TryRecvError::Disconnected) => break,
                }
                
                // lets not spin at 100% cpu on this thread...  maybe
                thread::sleep(std::time::Duration::from_millis(1));
            }

            cancel_rx.close();
            msg_rx.close();

            while let Some(msg) = msg_rx.blocking_recv() {
                let data = serde_json::to_string(&msg).expect("message should serialize");
                output.write_all(data.as_bytes()).expect("should be able to write to stdout");
            }
        });

        Self {
            msg_tx,
            cancel_tx: Some(cancel_tx),
            writer_handle: Some(handle),
        }
    }

    pub async fn send_msg(&self, msg: NodeMessage) {
        self.msg_tx.send(msg).await
            .expect("should send NodeMessage via channel")
    }

}

impl Default for StdoutSink {
    fn default() -> Self { Self::new() }
}

impl Drop for StdoutSink {
    fn drop(&mut self) {
        if let Some(tx) = self.cancel_tx.take() {
            let _ = tx.send(());
        }

        if let Some(handle) = self.writer_handle.take() {
            let _ = handle.join();
        }
    }
}