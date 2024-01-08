use std::sync::mpsc::Receiver;
use crate::classify::MessageToWorker;

fn classify_worker(receiver: Receiver<MessageToWorker>) {
    loop {
        let in_message = receiver.recv().unwrap();
        match in_message {
            MessageToWorker::DataPoint(_) => {}
            MessageToWorker::Shutdown => {
                break
            }
        }
    }
}