use std::time::Duration;

use indicatif::{ProgressBar, ProgressStyle};

pub struct Spinner {
    progress_bar: ProgressBar,
    silent: bool,
}

impl Spinner {
    /// Creates a new Spinner
    pub fn new(silent: bool) -> Self {
        let progress_bar = if silent {
            ProgressBar::hidden()
        } else {
            let progress_bar = ProgressBar::new_spinner();
            progress_bar.enable_steady_tick(Duration::from_millis(120));
            progress_bar.set_style(
                ProgressStyle::with_template("{spinner:.magenta} {msg}")
                    .unwrap()
                    .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
            );
            progress_bar
        };
        Self {
            silent,
            progress_bar,
        }
    }

    /// Stops the spinner successfully
    pub fn ok(&self) {
        if !self.silent {
            self.progress_bar.finish_and_clear();
        }
    }

    /// Stops the spinner with an error
    pub fn err(&self, msg: &str) {
        if !self.silent {
            self.progress_bar.abandon_with_message(msg.to_string());
        }
    }
}
