//! Progress spinner (port of the Go CLI's stripped-down briandowns/spinner).
//!
//! The spinner writes to stderr, and only when enabled with
//! [`enable_output`] (the root command does that when stdout is a terminal
//! and `--no-progress` is not given). Otherwise it is a silent no-op, like
//! the Go spinner writing to `io.Discard`.

use std::io::Write;
use std::sync::Mutex;
use std::time::Duration;

const CHARS: [&str; 8] = ["⣾", "⣽", "⣻", "⢿", "⡿", "⣟", "⣯", "⣷"];
const DELAY: Duration = Duration::from_millis(125);
const SUFFIX: &str = " Processing...";
const FINAL_MSG: &str = "Processing... Done.\n\n";

struct State {
    active: bool,
    output: bool,
    last_output: String,
    /// Incremented on every start so that an old animation thread stops.
    generation: u64,
}

static STATE: Mutex<State> = Mutex::new(State {
    active: false,
    output: false,
    last_output: String::new(),
    generation: 0,
});

fn write(state: &State, text: &str) {
    if state.output {
        let mut err = std::io::stderr().lock();
        let _ = err.write_all(text.as_bytes());
        let _ = err.flush();
    }
}

/// Delete the characters written last. Caller holds the lock.
fn erase(state: &mut State) {
    let n = state.last_output.chars().count();
    write(state, &format!("\r{}\r", " ".repeat(n)));
    state.last_output.clear();
}

/// Let the spinner write to stderr (`ProgressSpinner.Writer = os.Stderr`).
pub fn enable_output() {
    if let Ok(mut state) = STATE.lock() {
        state.output = true;
    }
}

/// `ProgressSpinner.Active()`
pub fn active() -> bool {
    STATE.lock().map(|s| s.active).unwrap_or(false)
}

/// `ProgressSpinner.Start()`
pub fn start() {
    let generation = {
        let Ok(mut state) = STATE.lock() else { return };
        if state.active {
            return;
        }
        state.active = true;
        state.generation += 1;
        state.generation
    };
    std::thread::spawn(move || {
        loop {
            for c in CHARS {
                {
                    let Ok(mut state) = STATE.lock() else { return };
                    if !state.active || state.generation != generation {
                        return;
                    }
                    erase(&mut state);
                    let out = format!("{c}{SUFFIX} ");
                    write(&state, &out);
                    state.last_output = out;
                }
                std::thread::sleep(DELAY);
            }
        }
    });
}

/// `ProgressSpinner.Stop()`: erase the spinner and print the final message.
pub fn stop() {
    let Ok(mut state) = STATE.lock() else { return };
    if state.active {
        state.active = false;
        erase(&mut state);
        write(&state, FINAL_MSG);
    }
}
