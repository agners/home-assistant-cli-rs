//! Helpers for the common command shapes of the Go CLI, so that a ported
//! command is mostly declarative. See `src/cmd/dns/` for usage.

use std::time::Duration;

use serde_json::Value;

use crate::client::{self, Options};
use crate::cobra::{Args, Command, Ctx, Flag, comp};
use crate::{debug, gofmt, output, spinner};

/// A leaf command with the settings nearly every Go leaf command has:
/// `Args: cobra.NoArgs` and `ValidArgsFunction: cobra.NoFileCompletions`.
pub fn leaf(use_line: &str) -> Command {
    Command::new(use_line).args(Args::None).valid_args(comp::no_file())
}

/// The `slog.Debug("<command>", "args", args)` line most Go commands start
/// with, e.g. `dns info args=[]`.
pub fn debug_args(ctx: &Ctx) {
    let path = ctx.command_path();
    let path = path.split_once(' ').map_or("", |(_, rest)| rest);
    debug!(path, "args" => gofmt::list(&ctx.args));
}

#[derive(Clone, Copy, PartialEq)]
enum Method {
    Get,
    Post,
}

/// A declarative Supervisor API call + response rendering, covering the
/// bodies of most Go `Run` functions:
///
/// ```ignore
/// // GenericJSONGet + ShowJSONResponse
/// .run(Call::get("dns", "info").run())
/// // spinner + GenericJSONPostTimeout + ShowJSONResponse
/// .run(Call::post("dns", "restart").timeout(client::CONTAINER_OPERATION_TIMEOUT).spinner().run())
/// // POST with a body built from flags
/// .run(Call::post("dns", "options").run_with(|ctx, options| { ... }))
/// // "{slug}" in the command taken from positional arg 0
/// .run(Call::post("addons", "{slug}/restart").arg_param("slug", 0).run())
/// ```
#[derive(Clone)]
pub struct Call {
    method: Method,
    section: String,
    command: String,
    timeout: Duration,
    spinner: bool,
    always_body: bool,
    arg_params: Vec<(String, usize)>,
}

impl Call {
    fn new(method: Method, section: &str, command: &str) -> Call {
        Call {
            method,
            section: section.to_string(),
            command: command.to_string(),
            timeout: client::DEFAULT_TIMEOUT,
            spinner: false,
            always_body: false,
            arg_params: Vec::new(),
        }
    }

    /// `helper.GenericJSONGet(section, command)`
    pub fn get(section: &str, command: &str) -> Call {
        Call::new(Method::Get, section, command)
    }

    /// `helper.GenericJSONPost(section, command, options)`
    pub fn post(section: &str, command: &str) -> Call {
        Call::new(Method::Post, section, command)
    }

    /// Use another timeout (`GenericJSONPostTimeout`, `GetJSONRequestTimeout`).
    pub fn timeout(mut self, timeout: Duration) -> Call {
        self.timeout = timeout;
        self
    }

    /// Wrap the request in `ProgressSpinner.Start()` / `.Stop()`.
    pub fn spinner(mut self) -> Call {
        self.spinner = true;
        self
    }

    /// Send the JSON body even when the options are empty (`{}`), for Go code
    /// that calls `request.SetBody(options)` unconditionally. By default an
    /// empty options map sends no body, like `GenericJSONPost`.
    pub fn always_body(mut self) -> Call {
        self.always_body = true;
        self
    }

    /// Fill the `{name}` path parameter from positional argument `index`
    /// (Go: `request.SetPathParams(map[string]string{"slug": args[0]})`).
    pub fn arg_param(mut self, name: &str, index: usize) -> Call {
        self.arg_params.push((name.to_string(), index));
        self
    }

    /// Perform the call and return the checked response
    /// (`GenericJSONErrorHandling` applied).
    pub fn request(&self, ctx: &Ctx, options: Options) -> client::Result<client::Response> {
        let url = client::url_helper(&self.section, &self.command)?;
        let mut request = client::json_request_timeout(self.timeout);
        for (name, index) in &self.arg_params {
            request = request.path_param(name, ctx.arg(*index));
        }
        if self.method == Method::Post && (self.always_body || !options.is_empty()) {
            request = request.body(options);
        }
        if self.spinner {
            spinner::start();
        }
        let result = match self.method {
            Method::Get => request.get(&url),
            Method::Post => request.post(&url),
        };
        if self.spinner {
            spinner::stop();
        }
        client::generic_json_error_handling(result)
    }

    /// Perform the call and print the outcome; `true` on success.
    pub fn execute(&self, ctx: &Ctx, options: Options) -> bool {
        output::show(self.request(ctx, options))
    }

    /// A `Run` function without request body.
    pub fn run(self) -> impl Fn(&Ctx) -> bool {
        move |ctx| {
            debug_args(ctx);
            self.execute(ctx, Options::new())
        }
    }

    /// A `Run` function whose request body is filled by `build` (from flags
    /// and args).
    pub fn run_with(self, build: impl Fn(&Ctx, &mut Options)) -> impl Fn(&Ctx) -> bool {
        move |ctx| {
            debug_args(ctx);
            let mut options = Options::new();
            build(ctx, &mut options);
            self.execute(ctx, options)
        }
    }
}

// --- request body helpers: copy flag values into the options map ----------

/// `if cmd.Flags().Changed(flag) { options[key] = GetBool(flag) }`
pub fn bool_if_changed(ctx: &Ctx, options: &mut Options, flag: &str, key: &str) {
    if ctx.changed(flag) {
        options.insert(key.to_string(), Value::Bool(ctx.bool(flag)));
    }
}

/// `if cmd.Flags().Changed(flag) { options[key] = GetString(flag) }`
pub fn string_if_changed(ctx: &Ctx, options: &mut Options, flag: &str, key: &str) {
    if ctx.changed(flag) {
        options.insert(key.to_string(), Value::String(ctx.string(flag)));
    }
}

/// `if v, _ := GetString(flag); v != "" { options[key] = v }`
pub fn string_if_set(ctx: &Ctx, options: &mut Options, flag: &str, key: &str) {
    let value = ctx.string(flag);
    if !value.is_empty() {
        options.insert(key.to_string(), Value::String(value));
    }
}

/// `if cmd.Flags().Changed(flag) { options[key] = GetInt(flag) }`
pub fn int_if_changed(ctx: &Ctx, options: &mut Options, flag: &str, key: &str) {
    if ctx.changed(flag) {
        options.insert(key.to_string(), Value::from(ctx.int(flag)));
    }
}

/// `if v, _ := GetStringArray(flag); len(v) >= 1 { options[key] = v }`
pub fn string_array_if_set(ctx: &Ctx, options: &mut Options, flag: &str, key: &str) {
    let values = ctx.string_array(flag);
    if !values.is_empty() {
        options.insert(key.to_string(), Value::from(values));
    }
}

// --- recurring command shapes ----------------------------------------------

/// The `--version` flag of the `update` commands.
pub fn version_flag() -> Flag {
    Flag::string("version", "", "Version to update to").completion(comp::no_file())
}

/// `Run` of the plug-in `update` commands: POST `<section>/update` with
/// `{"version": ...}` if `--version` is given, behind the spinner, with the
/// container download timeout.
pub fn run_update(section: &str) -> impl Fn(&Ctx) -> bool {
    Call::post(section, "update")
        .timeout(client::CONTAINER_DOWNLOAD_TIMEOUT)
        .spinner()
        .run_with(|ctx, options| string_if_set(ctx, options, "version", "version"))
}

/// `Run` of the plug-in `restart` commands: POST `<section>/restart` behind
/// the spinner with the container operation timeout.
pub fn run_restart(section: &str) -> impl Fn(&Ctx) -> bool {
    Call::post(section, "restart")
        .timeout(client::CONTAINER_OPERATION_TIMEOUT)
        .spinner()
        .run()
}

/// The Go CLI's deprecated-alias warning (see `cmd/apps.go`): prints
/// `The use of '<alias>' is deprecated, please use '<replacement>' instead!`
/// to stderr for every command line argument that is a deprecated alias.
/// Call it from a `persistent_pre_run` override, followed by
/// `crate::cmd::root_pre_run(ctx)`.
pub fn warn_deprecated_aliases(deprecated: &[&str], replacement: &str) {
    for arg in std::env::args().skip(1) {
        if deprecated.contains(&arg.as_str()) {
            eprintln!("The use of '{arg}' is deprecated, please use '{replacement}' instead!");
        }
    }
}
