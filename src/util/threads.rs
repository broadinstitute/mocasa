use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread::{JoinHandle, spawn};
use crate::error::Error;

pub(crate) trait InMessage: Send {
    fn i_thread(&self) -> usize;
}

pub(crate) trait OutMessage: Send + Clone {
    const SHUTDOWN: Self;
}

pub(crate) struct Threads<I, O> where I: InMessage + 'static, O: OutMessage + 'static {
    pub(crate) in_receiver: Receiver<I>,
    pub(crate) out_senders: Vec<Sender<O>>,
    pub(crate) join_handles: Vec<JoinHandle<()>>,
}

pub(crate) trait WorkerLauncher<I, O>: Clone + Send where {
    fn launch(self, in_sender: Sender<I>, out_receiver: Receiver<O>, i_thread: usize);
}

impl<I, O> Threads<I, O> where I: InMessage + 'static, O: OutMessage + 'static {
    pub(crate) fn new<L>(launcher: L, n_threads: usize) -> Threads<I, O>
        where L: WorkerLauncher<I, O> + 'static {
        let (in_sender, in_receiver) = channel::<I>();
        let (join_handles, out_senders) =
            launch_workers(in_sender.clone(), launcher, n_threads);
        Threads { in_receiver, out_senders, join_handles }
    }
    pub(crate) fn n_threads(&self) -> usize { self.join_handles.len() }
    pub(crate) fn broadcast(&self, out_message: O) -> Result<(), Error> {
        for out_sender in &self.out_senders {
            out_sender.send(out_message.clone())?;
        }
        Ok(())
    }
    pub(crate) fn responses_from_all(&self) -> Result<Vec<I>, Error> {
        let mut responses_opts: Vec<Option<I>> = (0..self.n_threads()).map(|_| None).collect();
        while responses_opts.iter().any(|response_opt| response_opt.is_none()) {
            let response = self.in_receiver.recv()?;
            let i_thread = response.i_thread();
            responses_opts[i_thread] = Some(response);
        }
        let responses: Vec<I> = responses_opts.into_iter().flatten().collect();
        Ok(responses)
    }
    pub(crate) fn task_queue<T: Iterator<Item=O>>(&self, out_messages: T) -> Result<Vec<I>, Error> {
        let mut maybe_more_out = true;
        let mut waiting_for_in = false;
        let mut task_by_thread: Vec<Option<usize>> =
            (0..self.n_threads()).map(|_| None).collect();
        let mut in_messages: Vec<Option<I>> = Vec::new();
        let mut out_iter = out_messages.enumerate();
        while maybe_more_out || waiting_for_in {
            if maybe_more_out {
                while let Some((i_thread_out, _)) =
                    task_by_thread.iter().enumerate()
                        .find(|(_, i_task)| i_task.is_none()) {
                    match out_iter.next() {
                        None => { maybe_more_out = false }
                        Some((i_task, out_message)) => {
                            self.out_senders[i_thread_out].send(out_message)?;
                            task_by_thread[i_thread_out] = Some(i_task);
                            waiting_for_in = true;
                        }
                    }
                }
            }
            if waiting_for_in {
                let in_message = self.in_receiver.recv()?;
                let i_thread_in = in_message.i_thread();
                let i_task = task_by_thread[i_thread_in].unwrap();
                println!("Completed task {}", i_task);
                task_by_thread[i_thread_in] = None;
                if in_messages.len() < i_task + 1 {
                    while in_messages.len() < i_task {
                        in_messages.push(None);
                    }
                    in_messages.push(Some(in_message))
                } else {
                    in_messages[i_task] = Some(in_message);
                }
                waiting_for_in =
                    task_by_thread.iter().any(|i_task_opt| i_task_opt.is_some())
            }
        }
        let in_messages: Vec<I> = in_messages.into_iter().flatten().collect();
        println!("Completed all {} tasks", in_messages.len());
        Ok(in_messages)
    }
}

impl<I, O> Drop for Threads<I, O> where I: InMessage + 'static, O: OutMessage + 'static {
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
