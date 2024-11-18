use nix::sys::wait::{waitpid, WaitPidFlag};
use nix::unistd::Pid;
use std::collections::HashMap;

pub struct JobControl {
    pub jobs: HashMap<u32, Pid>,
    pub next_job_id: u32,
}

impl JobControl {
    pub fn new() -> Self {
        Self {
            jobs: HashMap::new(),
            next_job_id: 1,
        }
    }

    pub fn add_job(&mut self, pid: Pid) -> u32 {
        let job_id = self.next_job_id;
        self.jobs.insert(job_id, pid);
        self.next_job_id += 1;
        job_id
    }

    pub fn remove_job(&mut self, job_id: u32) {
        self.jobs.remove(&job_id);
    }

    pub fn list_jobs(&self) {
        for (job_id, pid) in &self.jobs {
            println!("[{}] Running PID {}", job_id, pid);
        }
    }

    pub fn get_job(&self, job_id: u32) -> Option<Pid> {
        self.jobs.get(&job_id).copied()
    }

    pub fn bring_job_to_foreground(&mut self, job_id: u32, pid: Pid) {
        println!("Bringing job [{}] to foreground", job_id);
        waitpid(pid, None).expect("Failed to wait on child");
        self.remove_job(job_id);
    }
}
