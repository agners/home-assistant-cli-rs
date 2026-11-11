//! Output helpers: the Rust counterpart of `ShowJSONResponse`,
//! `StreamTextResponse`, `PrintError` in the Go `client` package.

use std::fmt::Display;
use std::io::{IsTerminal, Read, Write};

use serde_json::Value;

use crate::client::{self, ApiResponse, Options, Response};
use crate::goyaml;

/// `PrintErrorString`: `Error: <message>` on stderr, the prefix in bold red
/// when stderr is a terminal.
pub fn print_error_string(message: &str) {
    if std::io::stderr().is_terminal() {
        eprintln!("\x1b[1;31mError:\x1b[0m {message}");
    } else {
        eprintln!("Error: {message}");
    }
}

/// `PrintError`
pub fn print_error(err: &impl Display) {
    print_error_string(&err.to_string());
}

/// Write raw bytes to stdout, unbuffered like Go's `fmt.Print`.
pub fn print_bytes(bytes: &[u8]) {
    let mut out = std::io::stdout().lock();
    let _ = out.write_all(bytes);
    let _ = out.flush();
}

/// The usual tail of a command: print the error, or show the response.
/// Returns `true` on success (Go: `ExitWithError = !ShowJSONResponse(resp)`).
pub fn show(result: client::Result<Response>) -> bool {
    match result {
        Ok(mut resp) => show_json_response(&mut resp),
        Err(err) => {
            print_error(&err);
            false
        }
    }
}

/// `ShowJSONResponse` formats a JSON response for human readers.
pub fn show_json_response(resp: &mut Response) -> bool {
    show_json_response_transform(resp, None)
}

/// `ShowJSONResponseTransform`: like [`show_json_response`], applying
/// `transform` to the data of a successful response before rendering it. In
/// raw JSON mode the transform is not applied.
pub fn show_json_response_transform(resp: &mut Response, transform: Option<&dyn Fn(&mut Options)>) -> bool {
    if client::raw_json() {
        // when we are returning raw JSON, all handling is for the consumer of the JSON
        let mut body = Vec::new();
        if resp.reader().read_to_end(&mut body).is_ok() {
            print_bytes(&body);
        }
        return true;
    }
    let data = &mut resp.data;
    match data.result.as_str() {
        "ok" => {
            if data.data.is_empty() {
                println!("Command completed successfully.");
            } else {
                if let Some(transform) = transform {
                    transform(&mut data.data);
                }
                if !print_yaml(&Value::Object(data.data.clone())) {
                    return false;
                }
            }
            true
        }
        "error" => {
            print_error_string(&data.message);
            false
        }
        _ => {
            print_yaml(&envelope_value(data));
            false
        }
    }
}

/// Print a JSON value as YAML exactly like the Go CLI does. Returns `false`
/// (after printing the error) if the value cannot be rendered.
pub fn print_yaml(value: &Value) -> bool {
    match goyaml::to_yaml(value) {
        Ok(yaml) => {
            print_bytes(yaml.as_bytes());
            true
        }
        Err(err) => {
            print_error_string(&err);
            false
        }
    }
}

/// The envelope as Go would marshal the `Response` struct (`omitempty` on
/// message and data).
fn envelope_value(data: &ApiResponse) -> Value {
    let mut map = Options::new();
    map.insert("result".to_string(), Value::String(data.result.clone()));
    if !data.message.is_empty() {
        map.insert("message".to_string(), Value::String(data.message.clone()));
    }
    if !data.data.is_empty() {
        map.insert("data".to_string(), Value::Object(data.data.clone()));
    }
    Value::Object(map)
}

/// `StreamTextResponse` streams out a text response, like the ones from
/// `/host/logs`.
pub fn stream_text_response(resp: &mut Response) -> bool {
    let mut reader = resp.reader();
    let mut buf = [0u8; 4096];
    loop {
        match reader.read(&mut buf) {
            Ok(0) => return true,
            Ok(n) => print_bytes(&buf[..n]),
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => {}
            Err(e) => {
                println!("{e}");
                return false;
            }
        }
    }
}
