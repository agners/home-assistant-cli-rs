//! HTTP client for the Supervisor API: the Rust counterpart of the Go
//! `client` package (`URLHelper`, `GetRequest*`, `GenericJSON*`,
//! `GenericJSONErrorHandling`) plus a small request builder covering what the
//! Go commands do with resty.

use std::fmt;
use std::io::Read;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use serde_json::{Map, Value};

use crate::{config, debug, gofmt};

pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);
pub const CONTAINER_OPERATION_TIMEOUT: Duration = Duration::from_secs(10 * 60);
pub const CONTAINER_DOWNLOAD_TIMEOUT: Duration = Duration::from_secs(60 * 60);
pub const OS_DOWNLOAD_TIMEOUT: Duration = Duration::from_secs(60 * 60);
pub const BACKUP_TIMEOUT: Duration = Duration::from_secs(3 * 60 * 60);
pub const REBOOT_TIMEOUT: Duration = Duration::from_secs(90);
pub const MOUNT_TIMEOUT: Duration = Duration::from_secs(5 * 60);
/// No timeout at all (the Go code passes `0`, e.g. for following logs).
pub const NO_TIMEOUT: Duration = Duration::ZERO;

/// JSON object used for request bodies and response data
/// (Go: `map[string]any`).
pub type Options = Map<String, Value>;

/// `helper.RawJSON`: print API responses verbatim instead of YAML.
static RAW_JSON: AtomicBool = AtomicBool::new(false);

pub fn raw_json() -> bool {
    RAW_JSON.load(Ordering::Relaxed)
}

pub fn set_raw_json(value: bool) {
    RAW_JSON.store(value, Ordering::Relaxed);
}

/// A client error; its text is what `helper.PrintError(err)` would print.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Error(pub String);

impl Error {
    pub fn new(message: impl Into<String>) -> Error {
        Error(message.into())
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

pub type Result<T> = std::result::Result<T, Error>;

/// The default JSON response from the Home Assistant Supervisor
/// (Go: `client.Response`).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ApiResponse {
    pub result: String,
    pub message: String,
    pub data: Options,
}

// ---------------------------------------------------------------------------
// URL handling (net/url compatible where it matters)
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq)]
enum Encode {
    PathSegment,
    Path,
    Query,
}

fn should_escape(c: u8, mode: Encode) -> bool {
    if c.is_ascii_alphanumeric() || matches!(c, b'-' | b'_' | b'.' | b'~') {
        return false;
    }
    match c {
        b'$' | b'&' | b'+' | b',' | b'/' | b':' | b';' | b'=' | b'?' | b'@' => match mode {
            Encode::Path => c == b'?',
            Encode::PathSegment => matches!(c, b'/' | b';' | b',' | b'?'),
            Encode::Query => true,
        },
        _ => true,
    }
}

fn escape(s: &str, mode: Encode) -> String {
    let mut out = String::with_capacity(s.len());
    for &c in s.as_bytes() {
        if c == b' ' && mode == Encode::Query {
            out.push('+');
        } else if should_escape(c, mode) {
            out.push_str(&format!("%{c:02X}"));
        } else {
            out.push(c as char);
        }
    }
    out
}

/// Go's `url.PathEscape`.
pub fn path_escape(s: &str) -> String {
    escape(s, Encode::PathSegment)
}

/// Go's `url.QueryEscape`.
pub fn query_escape(s: &str) -> String {
    escape(s, Encode::Query)
}

/// Two hex digits at `b[i..i + 2]`.
fn hex2(b: &[u8], i: usize) -> Option<u8> {
    let digit = |c: u8| (c as char).to_digit(16);
    Some((digit(*b.get(i)?)? * 16 + digit(*b.get(i + 1)?)?) as u8)
}

/// Decode `%XX` sequences; invalid ones are kept as they are.
fn unescape(s: &str) -> String {
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'%'
            && let Some(v) = hex2(b, i + 1)
        {
            out.push(v);
            i += 3;
        } else {
            out.push(b[i]);
            i += 1;
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// Go's `path.Clean`.
fn clean_path(path: &str) -> String {
    if path.is_empty() {
        return ".".to_string();
    }
    let rooted = path.starts_with('/');
    let mut parts: Vec<&str> = Vec::new();
    for part in path.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                if parts.last().is_some_and(|p| *p != "..") {
                    parts.pop();
                } else if !rooted {
                    parts.push("..");
                }
            }
            p => parts.push(p),
        }
    }
    let joined = parts.join("/");
    if rooted {
        format!("/{joined}")
    } else if joined.is_empty() {
        ".".to_string()
    } else {
        joined
    }
}

/// Split `scheme://authority/path?query#fragment`.
fn split_url(url: &str) -> (&str, &str, &str, &str) {
    let (scheme_end, rest) = match url.find("://") {
        Some(i) => (i + 3, &url[i + 3..]),
        None => (0, url),
    };
    let authority_len = rest.find(['/', '?', '#']).unwrap_or(rest.len());
    let authority_end = scheme_end + authority_len;
    let after = &url[authority_end..];
    let (before_fragment, fragment) = match after.find('#') {
        Some(i) => (&after[..i], &after[i..]),
        None => (after, ""),
    };
    let (path, query) = match before_fragment.find('?') {
        Some(i) => (&before_fragment[..i], &before_fragment[i..]),
        None => (before_fragment, ""),
    };
    let _ = fragment;
    (&url[..authority_end], path, query, fragment)
}

fn validate_host(uri: &str, authority: &str) -> Result<()> {
    let host = authority.split("://").nth(1).unwrap_or(authority);
    let host = host.rsplit('@').next().unwrap_or(host);
    for c in host.chars() {
        let ok = c.is_ascii_alphanumeric() || "-_.~!$&'()*+,;=:[]<>\"%".contains(c) || !c.is_ascii();
        if !ok {
            return Err(Error(format!(
                "parse {}: invalid character {} in host name",
                gofmt::quote(uri),
                gofmt::quote(&c.to_string())
            )));
        }
    }
    if !host.starts_with('[')
        && let Some(i) = host.rfind(':')
        && !host[i + 1..].bytes().all(|b| b.is_ascii_digit())
    {
        return Err(Error(format!(
            "parse {}: invalid port {} after host",
            gofmt::quote(uri),
            gofmt::quote(&host[i..])
        )));
    }
    Ok(())
}

/// `URLHelper` returns a URL built from the endpoint, section and command.
/// The scheme defaults to `http://`, the path is cleaned like Go's
/// `path.Clean`, and `{placeholders}` stay untouched for path parameters.
pub fn url_helper(section: &str, command: &str) -> Result<String> {
    let base = config::endpoint();
    debug!("[GenerateURI]", "base" => base, "section" => section, "command" => command);
    let scheme = if base.contains("://") { "" } else { "http://" };
    let uri = format!("{scheme}{base}/{section}/{command}");
    let (authority, path, query, fragment) = split_url(&uri);
    validate_host(&uri, authority)?;
    let res = format!(
        "{}{}{}{}",
        authority,
        clean_path(&unescape(path)),
        unescape(query),
        unescape(fragment)
    );
    debug!("[GenerateURI] Result", "uri" => uri, "url(string)" => res);
    Ok(res)
}

/// True if the raw path only contains characters Go would leave unescaped.
fn valid_encoded_path(path: &str) -> bool {
    let b = path.as_bytes();
    let mut i = 0;
    while i < b.len() {
        let c = b[i];
        if c == b'%' {
            if hex2(b, i + 1).is_none() {
                return false;
            }
            i += 3;
            continue;
        }
        if !(b"!$&'()*+,;=:@[]".contains(&c) || !should_escape(c, Encode::Path)) {
            return false;
        }
        i += 1;
    }
    true
}

/// The URL as Go's `net/http` puts it on the wire (`URL.EscapedPath`).
fn wire_url(url: &str) -> String {
    let (authority, path, query, _fragment) = split_url(url);
    let path = if valid_encoded_path(path) {
        path.to_string()
    } else {
        escape(&unescape(path), Encode::Path)
    };
    let path = if path.is_empty() { "/".to_string() } else { path };
    format!("{authority}{path}{query}")
}

// ---------------------------------------------------------------------------
// Requests
// ---------------------------------------------------------------------------

/// resty's `IsJSONType`: `(?i:(application|text)/(.*json.*)(;|$))`
pub fn is_json_type(content_type: &str) -> bool {
    let ct = content_type.to_ascii_lowercase();
    ["application/", "text/"].iter().any(|prefix| {
        ct.match_indices(prefix)
            .any(|(i, _)| ct[i + prefix.len()..].contains("json"))
    })
}

/// A request to the Supervisor (the equivalent of a `*resty.Request`).
///
/// ```ignore
/// let url = client::url_helper("addons", "{slug}/update")?;
/// let resp = client::json_request_timeout(client::CONTAINER_DOWNLOAD_TIMEOUT)
///     .path_param("slug", slug)
///     .body(options)
///     .post(&url);
/// output::show(client::generic_json_error_handling(resp))
/// ```
#[derive(Debug, Clone)]
pub struct Request {
    /// HTTP method used by [`Request::send`].
    pub method: String,
    /// URL used by [`Request::send`]; may contain `{name}` path parameters.
    pub url: String,
    pub timeout: Duration,
    headers: Vec<(String, String)>,
    path_params: Vec<(String, String)>,
    query_params: Vec<(String, String)>,
    body: Option<Vec<u8>>,
    content_type: Option<String>,
    /// Parse the response body as the Supervisor JSON envelope.
    parse_json: bool,
}

/// `GetRequestTimeout`: a request with the `Accept: application/json` header
/// and bearer authentication; the response body is not interpreted.
pub fn request_timeout(timeout: Duration) -> Request {
    let mut request = Request {
        method: "GET".to_string(),
        url: String::new(),
        timeout,
        headers: Vec::new(),
        path_params: Vec::new(),
        query_params: Vec::new(),
        body: None,
        content_type: None,
        parse_json: false,
    };
    request.set_header("Accept", "application/json");
    let token = config::api_token();
    if !token.is_empty() {
        request.set_header("Authorization", &format!("Bearer {token}"));
    }
    request
}

/// `GetRequest`
pub fn request() -> Request {
    request_timeout(DEFAULT_TIMEOUT)
}

/// `GetJSONRequestTimeout`: like [`request_timeout`], and the response is
/// decoded into [`ApiResponse`] (unless `--raw-json` is active).
pub fn json_request_timeout(timeout: Duration) -> Request {
    let mut request = request_timeout(timeout);
    request.parse_json = !raw_json();
    request
}

/// `GetJSONRequest`
pub fn json_request() -> Request {
    json_request_timeout(DEFAULT_TIMEOUT)
}

impl Request {
    fn set_header(&mut self, name: &str, value: &str) {
        self.headers.retain(|(n, _)| !n.eq_ignore_ascii_case(name));
        self.headers.push((name.to_string(), value.to_string()));
    }

    /// `SetHeader` (replaces an existing header of the same name).
    pub fn header(mut self, name: &str, value: &str) -> Request {
        self.set_header(name, value);
        self
    }

    /// `SetPathParam`: replaces `{name}` in the URL with the path-escaped value.
    pub fn path_param(mut self, name: &str, value: &str) -> Request {
        self.path_params.retain(|(n, _)| n != name);
        self.path_params.push((name.to_string(), value.to_string()));
        self
    }

    /// `SetQueryParam`
    pub fn query_param(mut self, name: &str, value: &str) -> Request {
        self.query_params.retain(|(n, _)| n != name);
        self.query_params.push((name.to_string(), value.to_string()));
        self
    }

    /// `SetBody(map)`: JSON body (encoded like Go's `json.Marshal`) with
    /// `Content-Type: application/json`.
    pub fn body(self, options: Options) -> Request {
        self.body_json(&Value::Object(options))
    }

    /// JSON body from any JSON value.
    pub fn body_json(mut self, value: &Value) -> Request {
        self.body = Some(gofmt::json_marshal(value).into_bytes());
        self.content_type = Some("application/json".to_string());
        self
    }

    /// Raw body with an explicit content type.
    pub fn body_bytes(mut self, bytes: Vec<u8>, content_type: &str) -> Request {
        self.body = Some(bytes);
        self.content_type = Some(content_type.to_string());
        self
    }

    /// `SetDoNotParseResponse(true)`: leave the body as a stream
    /// (see [`Response::reader`]).
    pub fn do_not_parse(mut self) -> Request {
        self.parse_json = false;
        self
    }

    pub fn get(self, url: &str) -> Result<Response> {
        self.execute("GET", url)
    }

    pub fn post(self, url: &str) -> Result<Response> {
        self.execute("POST", url)
    }

    pub fn put(self, url: &str) -> Result<Response> {
        self.execute("PUT", url)
    }

    pub fn delete(self, url: &str) -> Result<Response> {
        self.execute("DELETE", url)
    }

    /// `request.Send()`: uses the `method` and `url` fields.
    pub fn send(self) -> Result<Response> {
        let (method, url) = (self.method.clone(), self.url.clone());
        self.execute(&method, &url)
    }

    /// The final URL: path parameters substituted, query string appended.
    fn final_url(&self, url: &str) -> String {
        let mut url = url.to_string();
        for (name, value) in &self.path_params {
            url = url.replace(&format!("{{{name}}}"), &path_escape(value));
        }
        if !self.query_params.is_empty() {
            let mut params = self.query_params.clone();
            params.sort();
            let query: Vec<String> = params
                .iter()
                .map(|(k, v)| format!("{}={}", query_escape(k), query_escape(v)))
                .collect();
            url.push(if url.contains('?') { '&' } else { '?' });
            url.push_str(&query.join("&"));
        }
        url
    }

    /// Perform the request with the given method.
    pub fn execute(self, method: &str, url: &str) -> Result<Response> {
        let url = self.final_url(url);
        let go_method = {
            let lower = method.to_ascii_lowercase();
            let mut chars = lower.chars();
            chars
                .next()
                .map(|c| c.to_ascii_uppercase().to_string() + chars.as_str())
                .unwrap_or_default()
        };
        let wrap = |detail: String| Error(format!("{} {}: {}", go_method, gofmt::quote(&url), detail));

        let mut builder = ureq::http::Request::builder().method(method).uri(wire_url(&url));
        for (name, value) in &self.headers {
            builder = builder.header(name.as_str(), value.as_str());
        }
        if let Some(content_type) = &self.content_type {
            builder = builder.header("Content-Type", content_type.as_str());
        }
        if let Some(body) = &self.body {
            debug!("Request body", "body" => String::from_utf8_lossy(body));
        }

        let config = ureq::Agent::config_builder()
            .http_status_as_error(false)
            .timeout_global(if self.timeout.is_zero() {
                None
            } else {
                Some(self.timeout)
            })
            .user_agent(concat!("ha-cli-rs/", env!("CARGO_PKG_VERSION")))
            .redirect_auth_headers(ureq::config::RedirectAuthHeaders::SameHost)
            .build();
        let agent = ureq::Agent::new_with_config(config);

        let result = match self.body {
            Some(body) => builder
                .body(body)
                .map_err(|e| wrap(e.to_string()))
                .and_then(|r| agent.run(r).map_err(|e| wrap(describe_error(&e, &url)))),
            None if matches!(method, "POST" | "PUT" | "PATCH") => builder
                .body(Vec::<u8>::new())
                .map_err(|e| wrap(e.to_string()))
                .and_then(|r| agent.run(r).map_err(|e| wrap(describe_error(&e, &url)))),
            None => builder
                .body(())
                .map_err(|e| wrap(e.to_string()))
                .and_then(|r| agent.run(r).map_err(|e| wrap(describe_error(&e, &url)))),
        };
        let http = result?;

        let status = http.status().as_u16();
        let content_type = http
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or_default()
            .to_string();
        debug!("Response", "statuscode" => status, "content-type" => content_type);
        let mut response = Response {
            status,
            content_type,
            data: ApiResponse::default(),
            bytes: Vec::new(),
            reader: None,
        };
        let mut reader = http.into_body().into_reader();
        if self.parse_json {
            reader
                .read_to_end(&mut response.bytes)
                .map_err(|e| wrap(e.to_string()))?;
            let parse = is_json_type(&response.content_type) && (response.is_success() || status > 399);
            if parse {
                response.data = parse_api_response(&response.bytes)?;
            }
        } else {
            response.reader = Some(Box::new(reader));
        }
        Ok(response)
    }
}

/// Go's errno text (`syscall.Errno.Error()`): lower-case, no error number.
pub fn errno_text(err: &std::io::Error) -> String {
    let text = err.to_string();
    let text = text.split(" (os error").next().unwrap_or(&text);
    let mut chars = text.chars();
    match chars.next() {
        Some(c) => c.to_lowercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

/// Best-effort rendering of transport errors in Go's wording.
fn describe_error(err: &ureq::Error, url: &str) -> String {
    let (authority, ..) = split_url(url);
    let https = authority.starts_with("https://");
    let host = authority.split("://").nth(1).unwrap_or(authority);
    let hostport = if host.contains(':') {
        host.to_string()
    } else {
        format!("{host}:{}", if https { 443 } else { 80 })
    };
    match err {
        ureq::Error::Io(e) if e.kind() == std::io::ErrorKind::ConnectionRefused => {
            format!("dial tcp {hostport}: connect: connection refused")
        }
        ureq::Error::ConnectionFailed => format!("dial tcp {hostport}: connect: connection refused"),
        ureq::Error::HostNotFound => {
            format!(
                "dial tcp: lookup {}: no such host",
                host.split(':').next().unwrap_or(host)
            )
        }
        ureq::Error::Timeout(_) => {
            "context deadline exceeded (Client.Timeout exceeded while awaiting headers)".to_string()
        }
        ureq::Error::Io(e) => errno_text(e),
        ureq::Error::TlsRequired => "https endpoints are not supported (built without TLS)".to_string(),
        other => other.to_string(),
    }
}

fn json_kind(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "bool",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

/// Decode the Supervisor JSON envelope with `encoding/json`-like strictness
/// and error messages.
pub fn parse_api_response(bytes: &[u8]) -> Result<ApiResponse> {
    // Like encoding/json, invalid UTF-8 is replaced instead of rejected.
    let text = String::from_utf8_lossy(bytes);
    let bytes = text.as_bytes();
    let value: Value = match serde_json::from_slice(bytes) {
        Ok(v) => v,
        Err(e) => {
            if e.is_eof() || bytes.iter().all(|b| b.is_ascii_whitespace()) {
                return Err(Error::new("unexpected end of JSON input"));
            }
            // Approximation of Go's syntax error text.
            let line = bytes
                .split(|b| *b == b'\n')
                .nth(e.line().saturating_sub(1))
                .unwrap_or_default();
            let text = String::from_utf8_lossy(line);
            let c = text.chars().nth(e.column().saturating_sub(1)).unwrap_or(' ');
            return Err(Error(format!("invalid character '{c}' looking for beginning of value")));
        }
    };
    let object = match value {
        Value::Object(map) => map,
        Value::Null => return Ok(ApiResponse::default()),
        other => {
            return Err(Error(format!(
                "json: cannot unmarshal {} into Go value of type client.Response",
                json_kind(&other)
            )));
        }
    };
    let mut response = ApiResponse::default();
    let field_error = |kind: &str, field: &str, ty: &str| {
        Error(format!(
            "json: cannot unmarshal {kind} into Go struct field Response.{field} of type {ty}"
        ))
    };
    for (key, value) in object {
        match key.to_ascii_lowercase().as_str() {
            "result" => match value {
                Value::String(s) => response.result = s,
                Value::Null => {}
                other => return Err(field_error(json_kind(&other), "result", "string")),
            },
            "message" => match value {
                Value::String(s) => response.message = s,
                Value::Null => {}
                other => return Err(field_error(json_kind(&other), "message", "string")),
            },
            "data" => match value {
                Value::Object(map) => response.data = map,
                Value::Null => {}
                other => return Err(field_error(json_kind(&other), "data", "map[string]interface {}")),
            },
            _ => {}
        }
    }
    Ok(response)
}

/// An API response (the equivalent of a `*resty.Response`).
pub struct Response {
    pub status: u16,
    pub content_type: String,
    /// The decoded envelope (`resp.Result().(*helper.Response)` /
    /// `resp.Error()`); all-default if the body was not parsed.
    pub data: ApiResponse,
    bytes: Vec<u8>,
    reader: Option<Box<dyn Read>>,
}

impl Response {
    /// `resp.StatusCode()`
    pub fn status_code(&self) -> u16 {
        self.status
    }

    /// `resp.IsSuccess()`: status 2xx.
    pub fn is_success(&self) -> bool {
        (200..300).contains(&self.status)
    }

    /// `resp.Body()`: the raw body of a parsed (non-streaming) response.
    pub fn body(&self) -> &[u8] {
        &self.bytes
    }

    /// `resp.RawBody()`: the body stream of a response that was not parsed
    /// (raw JSON mode, logs). Can be taken once.
    pub fn reader(&mut self) -> Box<dyn Read> {
        match self.reader.take() {
            Some(reader) => reader,
            None => Box::new(std::io::Cursor::new(std::mem::take(&mut self.bytes))),
        }
    }
}

/// `GenericJSONErrorHandling`: map HTTP status codes and non-JSON responses
/// of the Supervisor to errors.
pub fn generic_json_error_handling(result: Result<Response>) -> Result<Response> {
    let resp = result?;
    let json = is_json_type(&resp.content_type);
    match resp.status {
        200 | 400 | 404 | 503 | 429 => {
            // Success and these errors should have JSON responses
            if !json {
                return Err(Error(format!("unexpected non-JSON response (status: {})", resp.status)));
            }
        }
        500 => {
            if !json {
                return Err(Error::new(
                    "unknown error occurred, check supervisor logs with 'ha supervisor logs'",
                ));
            }
        }
        401 => {
            if !json {
                return Err(Error::new("unauthorized: missing or invalid API token"));
            }
        }
        403 => {
            if !json {
                return Err(Error::new("forbidden: insufficient permissions or invalid token"));
            }
        }
        502 => return Err(Error::new("bad gateway: core proxy or ingress service failure")),
        status => return Err(Error(format!("unexpected server response (status: {status})"))),
    }
    Ok(resp)
}

fn generic_json_method(
    get: bool,
    section: &str,
    command: &str,
    body: Option<Options>,
    timeout: Duration,
) -> Result<Response> {
    let url = url_helper(section, command)?;
    let request = json_request_timeout(timeout);
    let result = if get {
        request.get(&url)
    } else {
        match body {
            Some(body) if !body.is_empty() => request.body(body).post(&url),
            _ => request.post(&url),
        }
    };
    generic_json_error_handling(result)
}

/// `GenericJSONGet`
pub fn generic_json_get(section: &str, command: &str) -> Result<Response> {
    generic_json_method(true, section, command, None, DEFAULT_TIMEOUT)
}

/// `GenericJSONGetTimeout`
pub fn generic_json_get_timeout(section: &str, command: &str, timeout: Duration) -> Result<Response> {
    generic_json_method(true, section, command, None, timeout)
}

/// `GenericJSONPost` (an empty or missing body sends no body at all).
pub fn generic_json_post(section: &str, command: &str, body: Option<Options>) -> Result<Response> {
    generic_json_method(false, section, command, body, DEFAULT_TIMEOUT)
}

/// `GenericJSONPostTimeout`
pub fn generic_json_post_timeout(
    section: &str,
    command: &str,
    body: Option<Options>,
    timeout: Duration,
) -> Result<Response> {
    generic_json_method(false, section, command, body, timeout)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Mirrors `client/helper_test.go` of the Go CLI.
    #[test]
    fn url_helper_matches_go() {
        let cases = [
            ("supervisor", "section", "command", "http://supervisor/section/command"),
            (
                "supervisor:80",
                "section",
                "command",
                "http://supervisor:80/section/command",
            ),
            (
                "supervisor.example.org:8080",
                "section",
                "command",
                "http://supervisor.example.org:8080/section/command",
            ),
            (
                "https://supervisor",
                "section",
                "command",
                "https://supervisor/section/command",
            ),
            (
                "https://supervisor:8080",
                "section",
                "command",
                "https://supervisor:8080/section/command",
            ),
            (
                "https://supervisor",
                "section",
                "command",
                "https://supervisor/section/command",
            ),
            (
                "https://supervisor:8080",
                "section",
                "command",
                "https://supervisor:8080/section/command",
            ),
            ("supervisor", "section", "", "http://supervisor/section"),
            (
                "supervisor",
                "section/../othersection",
                "",
                "http://supervisor/othersection",
            ),
            (
                "supervisor/api/",
                "section",
                "command",
                "http://supervisor/api/section/command",
            ),
            (
                "supervisor/api/",
                "section",
                "{slug}/command",
                "http://supervisor/api/section/{slug}/command",
            ),
        ];
        for (base, section, command, want) in cases {
            config::set_endpoint_for_test(base);
            assert_eq!(
                url_helper(section, command).unwrap(),
                want,
                "[{base}][{section}][{command}]"
            );
        }
        config::set_endpoint_for_test("bad host");
        assert_eq!(
            url_helper("dns", "info").unwrap_err().0,
            "parse \"http://bad host/dns/info\": invalid character \" \" in host name"
        );
    }

    #[test]
    fn path_and_query_escaping() {
        assert_eq!(path_escape("core_ssh"), "core_ssh");
        assert_eq!(path_escape("a b/c?d"), "a%20b%2Fc%3Fd");
        assert_eq!(path_escape("a:b@c$d&e+f=g"), "a:b@c$d&e+f=g");
        assert_eq!(query_escape("a b&c=d/é"), "a+b%26c%3Dd%2F%C3%A9");
        assert_eq!(wire_url("http://h/a/{slug}/b c"), "http://h/a/%7Bslug%7D/b%20c");
        assert_eq!(wire_url("http://h/a%2Fb/c"), "http://h/a%2Fb/c");
        let request = request()
            .path_param("slug", "a/b")
            .query_param("z", "1 2")
            .query_param("a", "x");
        assert_eq!(
            request.final_url("http://h/addons/{slug}/info"),
            "http://h/addons/a%2Fb/info?a=x&z=1+2"
        );
    }

    #[test]
    fn json_content_types() {
        assert!(is_json_type("application/json"));
        assert!(is_json_type("application/json; charset=utf-8"));
        assert!(is_json_type("Application/Problem+JSON"));
        assert!(is_json_type("text/json"));
        assert!(!is_json_type("text/plain"));
        assert!(!is_json_type(""));
    }

    #[test]
    fn envelope_errors_match_go() {
        let err = |s: &str| parse_api_response(s.as_bytes()).unwrap_err().0;
        assert_eq!(
            err(r#"{"result":"ok","data":[1]}"#),
            "json: cannot unmarshal array into Go struct field Response.data of type map[string]interface {}"
        );
        assert_eq!(
            err("[1]"),
            "json: cannot unmarshal array into Go value of type client.Response"
        );
        assert_eq!(err(""), "unexpected end of JSON input");
        let ok = parse_api_response(br#"{"result":"error","message":"m","data":null}"#).unwrap();
        assert_eq!(
            (ok.result.as_str(), ok.message.as_str(), ok.data.len()),
            ("error", "m", 0)
        );
    }

    #[test]
    fn clean() {
        assert_eq!(clean_path("/a/b/../c/"), "/a/c");
        assert_eq!(clean_path("/a//b/./c"), "/a/b/c");
        assert_eq!(clean_path("/../a"), "/a");
        assert_eq!(clean_path("/"), "/");
        assert_eq!(clean_path(""), ".");
    }
}
