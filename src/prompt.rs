//! Interactive prompts: `AskForConfirmation`, `ReadInteger`, `ReadPassword`.

use std::io::{BufRead, BufReader, Read, Write};
use std::sync::Mutex;

use crate::client::{Error, Result, errno_text};

/// Unbuffered stdin (file descriptor 0).
struct RawStdin;

impl Read for RawStdin {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        // SAFETY: `buf` is valid for writes of `buf.len()` bytes.
        let n = unsafe { libc::read(0, buf.as_mut_ptr().cast(), buf.len()) };
        if n < 0 {
            Err(std::io::Error::last_os_error())
        } else {
            Ok(n as usize)
        }
    }
}

/// A fresh buffered reader on stdin for every prompt, like Go's
/// `bufio.NewReader(os.Stdin)` (including its read-ahead behaviour when
/// input is piped).
fn stdin_reader() -> BufReader<RawStdin> {
    BufReader::with_capacity(4096, RawStdin)
}

/// `reader.ReadString('\n')`: a line including the newline, `EOF` otherwise.
fn read_line(reader: &mut impl BufRead) -> Result<String> {
    let mut line = Vec::new();
    match reader.read_until(b'\n', &mut line) {
        Ok(_) if line.ends_with(b"\n") => Ok(String::from_utf8_lossy(&line).into_owned()),
        Ok(_) => Err(Error::new("EOF")),
        Err(e) => Err(Error::new(errno_text(&e))),
    }
}

fn prompt(text: &str) {
    print!("{text}");
    let _ = std::io::stdout().flush();
}

/// `AskForConfirmation`: asks `<prompt> [enter YES to confirm] ` up to
/// `tries` times (2 if `tries <= 0`). `yes` (any case) confirms, an answer
/// starting with `n` declines, anything else asks again.
pub fn ask_for_confirmation(text: &str, tries: i32) -> Result<bool> {
    let mut reader = stdin_reader();
    let tries = if tries <= 0 { 2 } else { tries };
    for _ in 0..tries {
        prompt(&format!("{text} [enter YES to confirm] "));
        let res = match read_line(&mut reader) {
            Ok(res) => res,
            Err(e) => {
                println!();
                return Err(e);
            }
        };
        // Require YES or yes explicitly to confirm
        let res = res.trim().to_lowercase();
        if res == "yes" {
            return Ok(true);
        }
        // If user enters no or n then stop. Else retry since they entered something unknown
        if res.starts_with('n') {
            return Ok(false);
        }
    }
    Ok(false)
}

/// `ReadInteger`: asks `<prompt> [min-max]: ` for a number in range.
pub fn read_integer(text: &str, tries: i32, min: i64, max: i64) -> Result<i64> {
    let mut reader = stdin_reader();
    let tries = if tries <= 0 { 2 } else { tries };
    for _ in 0..tries {
        prompt(&format!("{text} [{min}-{max}]: "));
        let res = match read_line(&mut reader) {
            Ok(res) => res,
            Err(e) => {
                println!();
                return Err(e);
            }
        };
        let res = res.trim();
        if res.is_empty() {
            continue;
        }
        match res.parse::<i64>() {
            Ok(val) if val >= min && val <= max => return Ok(val),
            _ => println!("Invalid value. Must be between {min} and {max}."),
        }
    }
    Err(Error::new("maximum tries exceeded"))
}

/// Terminal state to restore when a signal arrives during password entry.
static INITIAL_STATE: Mutex<Option<libc::termios>> = Mutex::new(None);

extern "C" fn restore_terminal_and_exit(_signal: libc::c_int) {
    // SAFETY: tcsetattr and _exit are async-signal-safe; try_lock never blocks.
    unsafe {
        if let Ok(guard) = INITIAL_STATE.try_lock()
            && let Some(state) = guard.as_ref()
        {
            libc::tcsetattr(0, libc::TCSANOW, state);
        }
        libc::_exit(1);
    }
}

fn os_error() -> Error {
    Error::new(errno_text(&std::io::Error::last_os_error()))
}

/// `term.ReadPassword(stdin)`: read a line with echo disabled.
fn read_password_line() -> Result<String> {
    // SAFETY: plain termios calls on fd 0 with valid local storage.
    let original = unsafe {
        let mut state: libc::termios = std::mem::zeroed();
        if libc::tcgetattr(0, &mut state) != 0 {
            return Err(os_error());
        }
        let mut new_state = state;
        new_state.c_lflag &= !libc::ECHO;
        new_state.c_lflag |= libc::ICANON | libc::ISIG;
        new_state.c_iflag |= libc::ICRNL;
        if libc::tcsetattr(0, libc::TCSANOW, &new_state) != 0 {
            return Err(os_error());
        }
        state
    };
    let mut stdin = RawStdin;
    let mut ret: Vec<u8> = Vec::new();
    let mut buf = [0u8; 1];
    let result = loop {
        match stdin.read(&mut buf) {
            // x/term's passwordReader returns (0, nil) at end of input, which
            // readPasswordLine ignores: Ctrl-D does not end the prompt.
            Ok(0) => {}
            Ok(_) => match buf[0] {
                0x08 => {
                    ret.pop();
                }
                b'\n' => break Ok(()),
                c => ret.push(c),
            },
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => {}
            Err(e) => break Err(Error::new(errno_text(&e))),
        }
    };
    // SAFETY: restores the state read above.
    unsafe {
        libc::tcsetattr(0, libc::TCSANOW, &original);
    }
    result.map(|_| String::from_utf8_lossy(&ret).into_owned())
}

/// `ReadPassword`: prompt for a password (twice if `repeat`) without echo.
/// The terminal is restored if the process is interrupted meanwhile.
pub fn read_password(repeat: bool) -> Result<String> {
    // SAFETY: tcgetattr with valid storage; signal handlers are installed
    // with a handler that only uses async-signal-safe functions.
    unsafe {
        let mut state: libc::termios = std::mem::zeroed();
        if libc::tcgetattr(0, &mut state) != 0 {
            return Err(os_error());
        }
        if let Ok(mut guard) = INITIAL_STATE.lock() {
            *guard = Some(state);
        }
        // Make sure terminal is restored on termination
        let handler = restore_terminal_and_exit as extern "C" fn(libc::c_int) as libc::sighandler_t;
        libc::signal(libc::SIGINT, handler);
        libc::signal(libc::SIGTERM, handler);
    }

    prompt("Password: ");
    let password = read_password_line();
    println!();
    let password = password?;

    if repeat {
        prompt("Password (again): ");
        let password2 = read_password_line();
        println!();
        if password != password2? {
            return Err(Error::new("passwords do not match"));
        }
    }
    Ok(password)
}
