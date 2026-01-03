use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::time::Duration;

const HEALTH_CHECK_TIMEOUT_MS: u64 = 500;
const MAX_STARTUP_ATTEMPTS: u32 = 20;
const STARTUP_POLL_INTERVAL_MS: u64 = 500;

pub struct OpenCodeManager {
    url: String,
    port: u16,
    child: Option<Child>,
}

impl OpenCodeManager {
    pub fn new(url: &str) -> Self {
        let port = url
            .split(':')
            .last()
            .and_then(|p| p.parse().ok())
            .unwrap_or(4096);

        Self {
            url: url.to_string(),
            port,
            child: None,
        }
    }

    /// Ensure OpenCode server is running, starting it if necessary
    pub async fn ensure_running(&mut self) -> anyhow::Result<()> {
        tracing::info!("Checking OpenCode server at {}...", self.url);

        // First, check if server is already running
        if self.health_check().await {
            tracing::info!("OpenCode server already running");
            return Ok(());
        }

        // Server not running, try to start it
        tracing::info!("Starting OpenCode server on port {}...", self.port);

        self.spawn_server().await?;

        tracing::info!("OpenCode server ready");

        Ok(())
    }

    /// Find the opencode binary
    fn find_binary() -> Option<PathBuf> {
        // First, check ~/.opencode/bin/opencode
        if let Some(home) = dirs::home_dir() {
            let opencode_path = home.join(".opencode").join("bin").join("opencode");
            if opencode_path.exists() {
                return Some(opencode_path);
            }
        }

        // Fallback to PATH
        which::which("opencode").ok()
    }

    /// Check if the server is responding
    async fn health_check(&self) -> bool {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_millis(HEALTH_CHECK_TIMEOUT_MS))
            .build()
            .ok();

        let Some(client) = client else {
            return false;
        };

        let url = format!("{}/doc", self.url);
        client.get(&url).send().await.is_ok()
    }

    /// Spawn the OpenCode server process
    async fn spawn_server(&mut self) -> anyhow::Result<()> {
        let binary = Self::find_binary().ok_or_else(|| {
            anyhow::anyhow!(
                "OpenCode binary not found.\n\n\
                 Install with: curl -fsSL https://opencode.ai/install.sh | sh\n\
                 Or ensure 'opencode' is in your PATH."
            )
        })?;

        tracing::debug!("Starting OpenCode from: {:?}", binary);

        let child = Command::new(&binary)
            .args(["serve", "--port", &self.port.to_string()])
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| anyhow::anyhow!("Failed to start OpenCode server from {:?}: {}", binary, e))?;

        self.child = Some(child);

        // Wait for server to be ready
        for attempt in 1..=MAX_STARTUP_ATTEMPTS {
            tokio::time::sleep(Duration::from_millis(STARTUP_POLL_INTERVAL_MS)).await;

            if self.health_check().await {
                return Ok(());
            }

            tracing::debug!(
                "OpenCode server not ready yet (attempt {}/{})",
                attempt,
                MAX_STARTUP_ATTEMPTS
            );
        }

        // Server didn't start in time
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();

            // Try to get stderr for debugging
            if let Some(stderr) = child.stderr.take() {
                use std::io::Read;
                let mut error_output = String::new();
                let mut stderr = stderr;
                let _ = stderr.read_to_string(&mut error_output);
                if !error_output.is_empty() {
                    tracing::error!("OpenCode server stderr: {}", error_output);
                }
            }
        }

        anyhow::bail!(
            "OpenCode server failed to start within {}s",
            (MAX_STARTUP_ATTEMPTS * STARTUP_POLL_INTERVAL_MS as u32) / 1000
        )
    }

    /// Gracefully shutdown the server if we started it
    pub fn shutdown(&mut self) {
        if let Some(mut child) = self.child.take() {
            tracing::debug!("Shutting down OpenCode server");
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

impl Drop for OpenCodeManager {
    fn drop(&mut self) {
        self.shutdown();
    }
}
