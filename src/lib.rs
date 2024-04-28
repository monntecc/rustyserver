// Library crate
use std::{
    sync::{mpsc, Arc, Mutex}, // Import necessary synchronization primitives from the standard library
    thread,                   // Import threading functionalities
};

#[allow(dead_code)]
pub struct ThreadPool {
    // Vector to store the worker threads
    workers: Vec<Worker>,
    // Sender channel to send jobs to workers
    sender: mpsc::Sender<Message>,
}

// Define the type of job - a boxed function pointer with ownership semantics
type Job = Box<dyn FnOnce() + Send + 'static>;

#[allow(dead_code)]
enum Message {
    // Message to signal a new job for the worker
    NewJob(Job),
    // Message to signal termination for the worker
    Terminate,
}

// Implement the ThreadPool
impl ThreadPool {
    /// Create a new ThreadPool.
    ///
    /// The size is the number of threads in the pool.
    ///
    /// # Panics
    ///
    /// The `new` function will panic if the size is zero.
    pub fn new(size: usize) -> Self {
        assert!(size > 0); // Ensure thread pool size is greater than zero

        let (sender, receiver) = mpsc::channel(); // Create a channel for communication
        let receiver = Arc::new(Mutex::new(receiver)); // Make the receiver thread-safe using Arc and Mutex

        let mut workers = Vec::with_capacity(size); // Initialize an empty worker vector with capacity

        for id in 0..size {
            // Add worker threads to the pool with a reference to the shared receiver channel
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        Self { workers, sender } // Return the ThreadPool instance
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static, // Specify the closure type for the job
    {
        let job = Box::new(f); // Box the closure to transfer ownership
        self.sender.send(Message::NewJob(job)).unwrap(); // Send the job over the channel
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        println!("Sending terminate message to all workers!");

        for _ in &self.workers {
            // Send termination message to all workers
            self.sender.send(Message::Terminate).unwrap();
        }

        for worker in &mut self.workers {
            println!("Shutting down worker: {}", worker.id);

            if let Some(thread) = worker.thread.take() {
                // Join the worker thread to wait for completion
                thread.join().unwrap();
            }
        }
    }
}

#[allow(dead_code)]
struct Worker {
    id: usize,
    // Handle to the worker thread, initially holding the join handle
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Self {
        let thread = thread::spawn(move || loop {
            // Get a job from the channel (receiver) with locking for thread safety
            let message = receiver.lock().unwrap().recv().unwrap();

            match message {
                Message::NewJob(job) => {
                    println!("Worker {} got a job; executing..", id);
                    job(); // Execute the received job
                }
                Message::Terminate => {
                    println!("Worker {} was told to terminate.", id);
                    break; // Exit the loop on termination message
                }
            }
        });

        Self {
            id,
            thread: Some(thread),
        }
    }
}
