use rhai::{Dynamic, Map};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, Mutex};
use std::thread;

pub struct JobMessage {
    pub name: String,
    pub payload: Dynamic,
}

#[derive(Clone)]
pub struct JobQueue {
    sender: Sender<JobMessage>,
    queued_count: Arc<AtomicUsize>,
    completed_count: Arc<AtomicUsize>,
    recent_jobs: Arc<Mutex<Vec<Map>>>,
}

impl JobQueue {
    pub fn new(num_workers: usize) -> Self {
        let (sender, receiver) = channel::<JobMessage>();
        let receiver = Arc::new(Mutex::new(receiver));
        let queued_count = Arc::new(AtomicUsize::new(0));
        let completed_count = Arc::new(AtomicUsize::new(0));
        let recent_jobs = Arc::new(Mutex::new(Vec::new()));

        let workers = if num_workers == 0 { 2 } else { num_workers };

        for _ in 0..workers {
            let rx = Arc::clone(&receiver);
            let queued = Arc::clone(&queued_count);
            let completed = Arc::clone(&completed_count);
            let recent = Arc::clone(&recent_jobs);

            thread::spawn(move || loop {
                let msg = {
                    let lock = rx.lock().unwrap();
                    lock.recv()
                };

                match msg {
                    Ok(job) => {
                        queued.fetch_sub(1, Ordering::SeqCst);
                        completed.fetch_add(1, Ordering::SeqCst);

                        let mut job_info = Map::new();
                        job_info.insert("name".into(), Dynamic::from(job.name));
                        job_info.insert("status".into(), Dynamic::from("completed"));
                        job_info.insert("payload".into(), job.payload);

                        let mut recents = recent.lock().unwrap();
                        if recents.len() >= 20 {
                            recents.remove(0);
                        }
                        recents.push(job_info);
                    }
                    Err(_) => break, // Channel disconnected
                }
            });
        }

        Self {
            sender,
            queued_count,
            completed_count,
            recent_jobs,
        }
    }

    pub fn dispatch(&self, name: &str, payload: Dynamic) -> bool {
        self.queued_count.fetch_add(1, Ordering::SeqCst);
        self.sender
            .send(JobMessage {
                name: name.to_string(),
                payload,
            })
            .is_ok()
    }

    pub fn stats(&self) -> Map {
        let mut map = Map::new();
        map.insert(
            "queued".into(),
            Dynamic::from(self.queued_count.load(Ordering::SeqCst) as i64),
        );
        map.insert(
            "completed".into(),
            Dynamic::from(self.completed_count.load(Ordering::SeqCst) as i64),
        );

        let recents = self.recent_jobs.lock().unwrap();
        let arr: rhai::Array = recents
            .iter()
            .cloned()
            .map(Dynamic::from)
            .collect();
        map.insert("recent".into(), Dynamic::from(arr));

        map
    }
}
