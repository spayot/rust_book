use std::{
    sync::{mpsc, Arc, Mutex},
    thread,
};

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

impl ThreadPool {
    /// Creates a new ThreadPool
    ///
    /// The size is the number of threads in the pool.
    ///
    /// # Panics
    ///
    /// the `new` function will panic if the pool size is set to zero.
    pub fn new(size: usize) -> Self {
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        Self {
            workers,
            sender: Some(sender),
        }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        self.sender.as_ref().unwrap().send(job).unwrap();
    }
}

struct Worker {
    id: usize,
    handle: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Self {
        let handle = thread::spawn(move || loop {
            let message = receiver.lock().unwrap().recv();
            match message {
                Ok(job) => {
                    println!("worker {id} got a job; executing.");
                    job()
                }
                Err(_) => {
                    println!("Worker {id} is shutting down.");
                    break;
                }
            }
        });
        Self {
            id: id,
            handle: Some(handle),
        }
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        for worker in &mut self.workers {
            drop(self.sender.take());
            println!("Shutting down worker {}.", worker.id);
            if let Some(handle) = worker.handle.take() {
                handle.join().unwrap();
            }
        }
    }
}
