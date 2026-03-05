use std::process::Stdio;

use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::mpsc;

/// Manages a child process (e.g. a dev server) spawned via `sh -c`.
///
/// The process runs in its own process group so the entire tree can be
/// killed cleanly on shutdown.
pub struct ChildProcess {
    child: tokio::process::Child,
    #[cfg(unix)]
    pid: u32,
}

impl ChildProcess {
    /// Spawn a command via `sh -c` and capture stdout/stderr into a log channel.
    pub fn spawn(command: &str) -> Result<(Self, mpsc::UnboundedReceiver<String>), std::io::Error> {
        let mut cmd = Command::new("sh");
        cmd.args(["-c", command]);
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
        cmd.kill_on_drop(true);

        // Create a new process group so we can kill the entire tree.
        #[cfg(unix)]
        unsafe {
            cmd.pre_exec(|| {
                libc::setpgid(0, 0);
                Ok(())
            });
        }

        let mut child = cmd.spawn()?;

        #[cfg(unix)]
        let pid = child.id().expect("child must have pid");

        let (tx, rx) = mpsc::unbounded_channel();

        // Read stdout lines into the channel.
        let stdout = child.stdout.take().expect("stdout must be piped");
        let tx_out = tx.clone();
        tokio::spawn(async move {
            let reader = BufReader::new(stdout);
            let mut lines = reader.lines();
            while let Ok(Some(line)) = lines.next_line().await {
                if tx_out.send(line).is_err() {
                    break;
                }
            }
        });

        // Read stderr lines into the same channel.
        let stderr = child.stderr.take().expect("stderr must be piped");
        tokio::spawn(async move {
            let reader = BufReader::new(stderr);
            let mut lines = reader.lines();
            while let Ok(Some(line)) = lines.next_line().await {
                if tx.send(line).is_err() {
                    break;
                }
            }
        });

        Ok((
            Self {
                child,
                #[cfg(unix)]
                pid,
            },
            rx,
        ))
    }

    /// Gracefully shut down the child process.
    ///
    /// On Unix: sends SIGTERM to the process group, waits up to 3 seconds,
    /// then sends SIGKILL if still running. On other platforms: kills directly.
    pub async fn shutdown(mut self) {
        #[cfg(unix)]
        {
            // SIGTERM to entire process group (negative pid).
            unsafe {
                libc::kill(-(self.pid as i32), libc::SIGTERM);
            }

            // Wait briefly for graceful shutdown.
            tokio::select! {
                _ = self.child.wait() => return,
                _ = tokio::time::sleep(std::time::Duration::from_secs(3)) => {}
            }

            // Force kill if still running.
            unsafe {
                libc::kill(-(self.pid as i32), libc::SIGKILL);
            }
        }

        #[cfg(not(unix))]
        {
            let _ = self.child.kill().await;
        }

        let _ = self.child.wait().await;
    }
}
