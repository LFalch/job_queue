pub mod queue;

use std::collections::VecDeque;
use std::marker::PhantomData;
use std::num::NonZeroUsize;
use std::sync::{Arc, Mutex, Condvar};
use std::thread::{self, JoinHandle};

pub use queue::Queue;

#[derive(Debug)]
pub struct JobQueue<T, Q: Queue<T>> {
    capacity: Option<NonZeroUsize>,
    queue: Mutex<Option<Q>>,
    _phantom: PhantomData<T>,
    cond_non_empty: Condvar,
    cond_non_full: Condvar,
    cond_empty: Condvar,
}

impl<T> JobQueue<T, VecDeque<T>> {
    #[inline]
    fn new() -> Self {
        Self::new_from(VecDeque::new(), None)
    }
    #[inline]
    fn with_capacity(cap: NonZeroUsize) -> Self {
        Self::new_from(VecDeque::with_capacity(cap.get()), Some(cap))
    }
}
impl<T, Q: Queue<T>> JobQueue<T, Q> {
    pub fn new_from(q: Q, capacity: Option<NonZeroUsize>) -> Self {
        JobQueue {
            capacity,
            queue: Mutex::new(Some(q)),
            _phantom: PhantomData,
            cond_non_empty: Condvar::new(),
            cond_non_full: Condvar::new(),
            cond_empty: Condvar::new(),
        }
    }
    fn destroy(&self) {
        let lock = self.queue.lock().unwrap();

        // If it's already destroyed, do nothing
        if lock.is_none() { return }

        let mut lock = self.cond_empty
            .wait_while(lock, |opt| !opt.as_ref().unwrap().is_empty())
            .unwrap();

        *lock = None; 
        // Notify all threads waiting for it to be non-empty
        // because now it never will be
        self.cond_non_empty.notify_all();
    }
    fn push(&self, job: T) -> Result<(), ()> {
        let lock = self.queue.lock().unwrap();

        let mut lock = if let Some(cap) = self.capacity {
            let cap = cap.get();
            self.cond_non_full
                .wait_while(lock, |opt| opt.as_ref().map(|q| q.len() >= cap).unwrap_or(false))
                .unwrap()
        } else {
            lock
        };

        if let Some(queue) = &mut *lock {
            queue.enqueue(job);

            self.cond_non_empty.notify_one();

            Ok(())
        } else {
            Err(())
        }
    }
    fn pop(&self) -> Result<T, ()> {
        let lock = self.queue.lock().unwrap();

        let mut lock = self.cond_non_empty
            .wait_while(lock, |opt| {
                opt.as_ref().map(|q| q.is_empty()).unwrap_or(false)
            })
            .unwrap();

        if let Some(queue) = &mut *lock {
            let ret = queue.dequeue().unwrap();

            self.cond_non_full.notify_one();
            if queue.is_empty() {
                self.cond_empty.notify_all();
            }

            Ok(ret)
        } else {
            Err(())
        }
    }
}

pub trait JobHandler<T> {
    fn handle(&mut self, job: T);
}

pub struct ThreadPool<T: Send, Q: Queue<T>> {
    jobs: Arc<JobQueue<T, Q>>,
    threads: Vec<JoinHandle<()>>,
}

impl<T: 'static + Send + Sync> ThreadPool<T, VecDeque<T>> {
    pub fn new<H: 'static + Clone + Send + JobHandler<T>>(thread_count: usize, job_handler: H) -> Self {
        Self::with_jobs(JobQueue::new(), thread_count, job_handler)
    }
    pub fn with_capacity<H: 'static + Clone + Send + JobHandler<T>>(capacity: NonZeroUsize, thread_count: usize, job_handler: H) -> Self {
        Self::with_jobs(JobQueue::with_capacity(capacity), thread_count, job_handler)
    }
}

impl<T: 'static + Send + Sync, Q: 'static + Send + Queue<T>> ThreadPool<T, Q> {
    pub fn with_jobs<H: 'static + Clone + Send + JobHandler<T>>(jobs: JobQueue<T, Q>, thread_count: usize, job_handler: H) -> Self {
        let jobs = Arc::new(jobs);
        assert!(thread_count > 0, "cannot spawn zero threads");
        ThreadPool {
            threads: (0..thread_count).map(|_| {
                let jobs = jobs.clone();
                let mut job_handler = job_handler.clone();
                thread::spawn(move || {
                    while let Ok(job) = jobs.pop() {
                        job_handler.handle(job);
                    }
                })
            }).collect(),
            jobs,
        }
    }
    pub fn add_job(&self, job: T) {
        self.jobs.push(job).unwrap();
    }
}

impl<T: Send, Q: Queue<T>> Drop for ThreadPool<T, Q> {
    fn drop(&mut self) {
        self.jobs.destroy();

        for thread in self.threads.drain(..) {
            thread.join().unwrap_or(())
        }
    }
}