use indicatif::{ProgressBar, ProgressStyle};
use rand::seq::SliceRandom;
use std::{sync::Arc, time::Duration};
use tokio::sync::Mutex;
use tokio_stream::StreamExt;

/// List of phrases used for the phrases spinner.
static PHRASES: [&str; 113] = [
    "Lubricating the hamsters...",
    "Winding up the rubber band...",
    "Giving the bits a stern talking to...",
    "Bribing the loading bar...",
    "Feeding carrots to the loading donkey...",
    "Pressing the 'Any' key...",
    "Herding the CPU sheep...",
    "Pruning redundant binary trees...",
    "Debugging the DeLorean's flux capacitor...",
    "Realigning the dilithium crystals...",
    "Untangling the interwebs...",
    "Petting the tribbles...",
    "Locating the missing Oxford comma...",
    "Rounding up missing semicolons...",
    "Tightening loose CSS floats...",
    "Plugging memory leaks...",
    "Updating Adobe Reader...",
    "Disinfecting droids...",
    "Defragmenting the hard drive...",
    "Polishing pixels...",
    "Resetting the odometer...",
    "Priming the warp core...",
    "Tuning the holodeck...",
    "Spinning up the flux inverter...",
    "Starting engines...",
    "Lubricating wheels...",
    "Corraling the penguins...",
    "Baking pixels...",
    "Putting the 'fun' in fundamental algorithms...",
    "Herding cats...",
    "Inserting witty loading message...",
    "Teaching monkeys to type...",
    "Untangling the wires...",
    "Locating sense of purpose...",
    "Spinning up the hamster wheels...",
    "Fluffing the pillows...",
    "Straightening the rug...",
    "Feeding the fish...",
    "Photon alignment...",
    "Reconfiguring quibits...",
    "Translating whims into action items...",
    "Activating sloths...",
    "Priming the pump...",
    "Twiddling the bits...",
    "Polishing the monocle...",
    "Herding the cats...",
    "Locating my marbles...",
    "Unsticking stuck pixels...",
    "Untangling the yarn...",
    "Alphabetizing the library...",
    "Dusting the cobwebs...",
    "Poking the angry badger...",
    "Re-hydrating the fish...",
    "Counting the grains of sand...",
    "Tuning the orchestra...",
    "Beating the high score...",
    "Photocopying the paperwork...",
    "Proofreading the dictionary...",
    "Translating to Pig Latin...",
    "De-wrinkling the fabric of space-time...",
    "Rounding up the unicorns...",
    "Sharpening the pencils...",
    "Milking the concrete cow...",
    "Herding Schrödinger's cats...",
    "Digitizing the analog...",
    "Retouching the masterpiece...",
    "Debugging the dreamweaver...",
    "Restocking the water cooler...",
    "Reloading the motivation...",
    "Calibrating the mood rings...",
    "Fluffing the pillows...",
    "Coiling the garden hose...",
    "Putting on my wizard hat...",
    "Baking your cookies...",
    "Locating my eye patch...",
    "Polishin' my monocle...",
    "Fetching my quill and parchment...",
    "Saddling the centaurs...",
    "Rounding up the hedgehogs...",
    "Tuning my banjo...",
    "Anthropomorphizing the mushrooms...",
    "Driving the snails...",
    "Turning the cranks and tightening the springs...",
    "Poking the bees...",
    "Rustling the jimmies...",
    "Counting the spiderwebs...",
    "Organizing my sock drawer...",
    "Auditioning the crickets...",
    "Testing the waters...",
    "Routing the pigeons...",
    "Pulling the levers...",
    "Winding the clock...",
    "Consulting the ancient scrolls...",
    "Polishing the pixels...",
    "Reticulating splines...",
    "Herding cats...",
    "Aligning the flux capacitor...",
    "Spinning up the hamster wheel...",
    "Baking the cookies...",
    "Calibrating the confetti cannons...",
    "Ironing out the electrons...",
    "Rebooting the sugar rush...",
    "Inserting witty message here...",
    "Activating imagination modules...",
    "Distracting you with this message...",
    "Gathering 1's and 0's...",
    "Locating sense of purpose...",
    "Brewing another pot of coffee...",
    "Start the reactor... (imagine heavy machinery noise)",
    "Raise the mizzenmast...",
    "Turn the crank...",
    "Wind up the gramophone...",
    "Ready the trebuchet...",
];

/// Struct to hold the spinner implementation.
pub struct Spinner {
    progress_bar: ProgressBar,
    state: State,
}

/// The State represents the status of the spinner.
#[derive(Debug, PartialEq)]
enum State {
    /// The spinner is running
    Run,
    /// The spinner is stopped
    Stop,
    /// The spinner errored out
    Error,
}

impl Spinner {
    /// Creates a new Spinner
    pub fn new() -> Self {
        let progress_bar = ProgressBar::new_spinner();
        progress_bar.enable_steady_tick(Duration::from_millis(100));
        progress_bar.set_style(
            ProgressStyle::with_template("{spinner:.magenta} {msg}")
                .unwrap()
                .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
        );

        Self {
            progress_bar,
            state: State::Run,
        }
    }

    /// Cretes a new Spinner that will be constantly chainging its loading message.
    pub fn new_with_checky_messages(millis: u64) -> std::sync::Arc<tokio::sync::Mutex<Spinner>> {
        let spinner_arc = Arc::new(Mutex::new(Spinner::new()));
        let spinner_clone = Arc::clone(&spinner_arc);

        let mut rng = rand::thread_rng();
        let mut phrases = PHRASES.to_vec();
        phrases.shuffle(&mut rng);

        tokio::spawn(async move {
            let mut stream = tokio_stream::iter(phrases.iter());
            while let Some(phrase) = stream.next().await {
                let mut spinner = spinner_arc.lock().await;
                if spinner.state != State::Run {
                    break;
                }
                spinner.message(phrase);
                tokio::time::sleep(Duration::from_millis(millis)).await;
            }
        });

        spinner_clone
    }

    /// Prints a message along the spinner.
    pub fn message(&mut self, msg: &str) {
        if self.state == State::Run {
            self.progress_bar.set_message(msg.to_string());
        }
    }

    /// Prints a message along the spinner.
    pub fn print(&mut self, msg: &str) {
        if self.state == State::Run {
            self.progress_bar.suspend(|| {
                println!("{}", msg);
            });
        }
    }

    // Stops the execution of the spinner.
    pub fn stop(&mut self) {
        if self.state == State::Run {
            self.state = State::Stop;
            self.progress_bar.finish_and_clear();
            tracing::event!(tracing::Level::INFO, "Done");
        }
    }

    // Stops the execution of the spinner with an error.
    pub fn error(&mut self, msg: &str) {
        if self.state == State::Run {
            self.state = State::Error;
            self.progress_bar.abandon_with_message(msg.to_string());
            tracing::event!(tracing::Level::ERROR, "{}", msg);
        }
    }
}

impl Default for Spinner {
    fn default() -> Self {
        Self::new()
    }
}
