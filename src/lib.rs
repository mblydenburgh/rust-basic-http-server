use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;

// Thread pool has a list of all available Workers as well as a Sender
// to share jobs across multiple threads
pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Message>,
}
// Holds job in a closure
type Job = Box<dyn FnOnce() + Send + 'static>;

impl ThreadPool {
    /// Creates a new ThreadPool.
    ///
    /// The size is the given number of threads in the pool
    ///
    /// # Panics
    ///
    /// The `new` function will panic if size == 0
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        // Create a new sender/receiver channel to handler multithreaded requests.
        // The threadpool is created with the sender, and each Worker has a receiver to
        // get the code to execute in a closure. Arc is needed to share ownership
        // across multiple Workers.
        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        for i in 0..size {
            // Create threads to store in vector
            // The std thread::spawn needs code that will run right away,
            // here we need to create threads but not have them execute
            // until needed by the webserver. Send receiver to each Worker
            // in order ot give it its execution code.
            let worker = Worker::new(i, Arc::clone(&receiver));
            workers.push(worker);
        }

        ThreadPool { workers, sender }
    }

    // Execute takes a closure definition, and uses ThreadPool's sender to give a worker
    // a closure to execute.
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);

        self.sender.send(Message::NewJob(job)).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        println!("Sending terminate message to all workers");

        for _ in &self.workers {
            self.sender.send(Message::Terminate).unwrap();
        }

        println!("Shutting down all workers");

        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

// Worker is a intermediary data structure to sit inbetween the ThreadPool
// and the actual threads. It enables the code to spawn a new thread but not
// immediately execute its closure
pub struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    /// Creates a new thread Worker or terminates an existing one.
    /// 
    /// id is a number assigned to each worker for identification
    /// receiver is a Arc<Mutex<Receiver<Message>>> messenger component
    /// 
    /// # Panics
    ///
    /// Will panic if another thread failed to unlock the messager
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let message = receiver.lock().unwrap().recv().unwrap();
            match message {
                Message::NewJob(job) => {
                    println!("Worker {} got a job; executing.", id);
                    job();
                }
                Message::Terminate => {
                    println!("Worker {} was told to terminate.", id);
                    break;
                }
            }
        });
        Worker {
            id,
            thread: Some(thread),
        }
    }
}

enum Message {
    NewJob(Job),
    Terminate,
}