use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;

pub struct DiffJob {
    pub seq: u64,
    pub hash: String,
}

pub struct DiffResult {
    pub seq: u64,
    pub text: String,
}

pub struct DiffWorker {
    sender: mpsc::Sender<DiffJob>,
    receiver: mpsc::Receiver<DiffResult>,
}

impl DiffWorker {
    pub fn new(repo: std::path::PathBuf, delta: Option<String>) -> Self {
        let (job_tx, job_rx) = mpsc::channel::<DiffJob>();
        let (res_tx, res_rx) = mpsc::channel::<DiffResult>();
        thread::spawn(move || {
            while let Ok(job) = job_rx.recv() {
                let text = fetch_diff(&repo, &job.hash, delta.as_deref());
                let _ = res_tx.send(DiffResult { seq: job.seq, text });
            }
        });
        Self {
            sender: job_tx,
            receiver: res_rx,
        }
    }

    pub fn dispatch(&self, seq: u64, hash: &str) {
        let _ = self.sender.send(DiffJob {
            seq,
            hash: hash.to_string(),
        });
    }

    pub fn poll(&self) -> Option<DiffResult> {
        self.receiver.try_recv().ok()
    }
}

fn fetch_diff(repo: &Path, hash: &str, delta: Option<&str>) -> String {
    let patch = crate::git::show_diff(repo, hash).unwrap_or_else(|| {
        "(no diff available — root commit or binary-only change)".to_string()
    });
    if let Some(delta_cmd) = delta {
        if let Ok(styled) = pipe_through_delta(delta_cmd, &patch) {
            return styled;
        }
    }
    patch
}

fn pipe_through_delta(delta_cmd: &str, input: &str) -> Result<String, ()> {
    let mut child = Command::new(delta_cmd)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|_| ())?;
    if let Some(mut stdin) = child.stdin.take() {
        use std::io::Write;
        let _ = stdin.write_all(input.as_bytes());
    }
    let out = child.wait_with_output().map_err(|_| ())?;
    if out.status.success() {
        Ok(String::from_utf8_lossy(&out.stdout).to_string())
    } else {
        Err(())
    }
}

pub fn format_commit_details(repo: &Path, hash: &str) -> String {
    crate::git::show_commit(repo, hash).unwrap_or_else(|| format!("commit {hash}\n(no details)"))
}
