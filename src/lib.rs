// Library crate

use std::{
    sync::{mpsc, Arc, Mutex}, // Import necessary synchronization primitives from the standard library
    thread,                   // Import threading functionalities
};

#[allow(dead_code)]
pub struct ThreadPool {
    workers: Vec<Worker>,      // Vector to store the worker threads
    sender: mpsc::Sender<Job>, // Sender channel to send jobs to workers
}

type Job = Box<dyn FnOnce() + Send + 'static>; // Define the type of job - a boxed function pointer

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
            workers.push(Worker::new(id, Arc::clone(&receiver))); // Add worker threads to the pool
        }

        Self { workers, sender } // Return the ThreadPool instance
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static, // Specify the closure type for the job
    {
        let job = Box::new(f); // Box the closure to transfer ownership
        self.sender.send(job).unwrap(); // Send the job over the channel
    }
}

#[allow(dead_code)]
struct Worker {
    id: usize,
    thread: thread::JoinHandle<()>, // Handle to the worker thread
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Self {
        let thread = thread::spawn(move || loop {
            // Get a job from the channel
            let job = receiver.lock().unwrap().recv().unwrap();
            println!("Worker {} got a job; executing..", id);
            job(); // Execute the received job
        });

        Self { id, thread }
    }
}
