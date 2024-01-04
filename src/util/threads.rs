use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread::{JoinHandle, spawn};

pub(crate) trait OutMessage: Send {
    const SHUTDOWN: Self;
}

pub(crate) struct Threads<I, O> where I: Send + 'static, O: OutMessage + 'static {
    pub(crate) in_receiver: Receiver<I>,
    pub(crate) out_senders: Vec<Sender<O>>,
    pub(crate) join_handles: Vec<JoinHandle<()>>,
}

pub(crate) trait WorkerLauncher<I, O>: Clone + Send where {
    fn launch(self, in_sender: Sender<I>, out_receiver: Receiver<O>, i_thread: usize);
}

impl<I, O> Threads<I, O> where I: Send + 'static, O: OutMessage + 'static {
    pub(crate) fn new<L>(launcher: L, n_threads: usize) -> Threads<I, O>
        where L: WorkerLauncher<I, O> + 'static {
        let (in_sender, in_receiver) = channel::<I>();
        let (join_handles, out_senders) =
            launch_workers(in_sender.clone(), launcher, n_threads);
        Threads { in_receiver, out_senders, join_handles }
    }
}

impl<I, O> Drop for Threads<I, O> where I: Send + 'static, O: OutMessage + 'static {
    fn drop(&mut self) {
        for (i, sender) in self.out_senders.iter().enumerate() {
            match sender.send(O::SHUTDOWN) {
                Ok(_) => { println!("Sent to worker {} request to shut down.", i) }
                Err(_) => { println!("Could not reach worker {}.", i) }
            };
        }
        for (i, join_handle)
        in self.join_handles.drain(0..self.join_handles.len()).enumerate() {
            match join_handle.join() {
                Ok(_) => { println!("Worker {} has shut down.", i) }
                Err(_) => { println!("Worker {} has crashed.", i) }
            }
        }
    }
}

fn launch_workers<I, O, L>(in_sender: Sender<I>, launcher: L, n_threads: usize)
                           -> (Vec<JoinHandle<()>>, Vec<Sender<O>>)
    where I: Send + 'static, O: OutMessage + 'static, L: WorkerLauncher<I, O> + 'static
{
    let mut join_handles: Vec<JoinHandle<()>> = Vec::with_capacity(n_threads);
    let mut senders: Vec<Sender<O>> = Vec::with_capacity(n_threads);
    for i_thread in 0..n_threads {
        let in_sender = in_sender.clone();
        let (out_sender, out_receiver) = channel::<O>();
        let launcher = launcher.clone();
        let join_handle = spawn(move || {
            launcher.launch(in_sender, out_receiver, i_thread);
        });
        join_handles.push(join_handle);
        senders.push(out_sender);
    }
    (join_handles, senders)
}
