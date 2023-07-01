use std::io::Read;
use std::time::Duration;

use indicatif::{ProgressBar, ProgressStyle};

/// Spinner state
enum SpinnerState {
    /// Spinner is running
    Running,
    /// Spinner is stopped
    Stopped,
    /// Spinner is silent
    Silent,
    /// Spinner is errored
    Errored,
}

pub struct Spinner {
    progress_bar: ProgressBar,
    state: SpinnerState,
}

impl Spinner {
    /// Creates a new Spinner
    pub fn new(silent: bool) -> Self {
        let progress_bar = if silent {
            ProgressBar::hidden()
        } else {
            let progress_bar = ProgressBar::new_spinner();
            progress_bar.enable_steady_tick(Duration::from_millis(100));
            progress_bar.set_style(
                ProgressStyle::with_template("{spinner:.magenta} {msg}")
                    .unwrap()
                    .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
            );
            progress_bar
        };
        Self {
            state: if silent {
                SpinnerState::Silent
            } else {
                SpinnerState::Running
            },
            progress_bar,
        }
    }

    pub fn print(&mut self, msg: &str) {
        if let SpinnerState::Running = self.state {
            self.progress_bar.suspend(|| {
                print!("{}", msg);
            });
        }
    }

    /// Stops the spinner successfully
    pub fn ok(&mut self) {
        if let SpinnerState::Running = self.state {
            self.state = SpinnerState::Stopped;
            self.progress_bar.finish_and_clear()
        }
    }

    /// Stops the spinner with an error
    pub fn err(&mut self, msg: &str) {
        if let SpinnerState::Running = self.state {
            self.progress_bar.abandon_with_message(msg.to_string());
            self.state = SpinnerState::Errored;
        }
    }
}

/// Reads stdin and retusn a string with its content.
pub fn read_from_stdin() -> Result<String, std::io::Error> {
    let mut stdin = Vec::new();
    log::debug!("Reading from stdin...");
    let mut lock = std::io::stdin().lock();
    lock.read_to_end(&mut stdin)?;
    Ok(String::from_utf8_lossy(&stdin).to_string())
}
