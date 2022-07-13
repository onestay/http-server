use std::thread::{self, JoinHandle};
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;

pub(crate) struct ThreadPool {
    threads: Vec<Worker>,
    sender: mpsc::Sender<Job>
}

impl ThreadPool {
    pub(crate) fn new(num_threads: usize) -> Self {
        assert!(num_threads > 0);
        let mut threads = Vec::with_capacity(num_threads);

        let (sender, recviever) = mpsc::channel();
        let recviever = Arc::new(Mutex::new(recviever));

        for i in 0..num_threads {
            let worker = Worker::new(i, Arc::clone(&recviever)); 
            threads.push(worker);
        }

        ThreadPool { threads, sender }
    }

    pub(crate) fn execute<T>(&self, job: T)
    where
        T: FnOnce() + Send + 'static
    {
        let job = Box::new(job);

        self.sender.send(job).unwrap();        
    }
}

struct Worker{
    id: usize,
    thread: JoinHandle<()>
}

impl Worker {
    fn new(id: usize, recviever: Arc<Mutex<mpsc::Receiver<Job>>>) -> Self {
        let thread =thread::spawn(move || loop {
            let job = recviever.lock().unwrap().recv().unwrap();

            println!("Worker {} got a job; executing.", id);

            job();
        });
        Worker { id, thread  }
    }
}

type Job = Box<dyn FnOnce() + Send + 'static>;
