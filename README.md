# Home Assistant CLI (Rust port)

A Rust reimplementation of the [Home Assistant CLI](https://github.com/home-assistant/cli)
(`ha`), meant as a drop-in replacement for the Go binary: same commands,
aliases, flags, HTTP requests, output and exit codes.

```sh
cargo build --release          # target/release/ha (small, stripped)
cargo test                     # unit tests
python3 tests/parity.py        # Go vs Rust behaviour diff (see below)
python3 tests/tree_diff.py     # Go vs Rust command tree / help diff
```

Status: all command groups of the Go CLI are ported.

## Contributing

### Module layout

```
src/
  main.rs            entry point, exit code (0 / 1 like the Go main.go)
  cobra/             cobra + pflag compatible command framework (see below)
    mod.rs             Command, Args, Ctx (builder API)
    flags.rs           Flag, FlagSet: pflag parsing, errors, usage text
    exec.rs            command lookup + execution (cobra Find/ExecuteC), `help` command
    help.rs            help/usage rendering (cobra's default templates)
    complete.rs        `__complete` protocol, `completion <shell>` command
    comp.rs            ready-made completers: no_file(), bools(), fixed(), from_fn()
    completions/*.tmpl cobra's shell scripts (bash, zsh, fish, powershell)
  config.rs          viper equivalent: flag > env > config file > default
  client.rs          URL helper, request builder, GenericJSON*, error handling, timeouts
  output.rs          ShowJSONResponse(+Transform), StreamTextResponse, PrintError
  goyaml.rs          YAML emitter identical to ghodss/yaml + go-yaml v2
  gofmt.rs           Go formatting helpers (%q, float formatting, json.Marshal)
  prompt.rs          AskForConfirmation, ReadInteger, ReadPassword
  spinner.rs         progress spinner (stderr, only on a TTY)
  log.rs             slog-like logging, `debug!` / `info!` macros
  cmd/
    mod.rs             root command, global flags, registration of ALL groups
    helpers.rs         leaf(), Call, body helpers, run_update(), run_restart(), ...
    logs.rs            add_logs_flags(), process_logs_flags(), run_logs()
    completions.rs     completers shared by several groups (apps, store apps, ...)
    dns/               REFERENCE GROUP: mod.rs + one file per sub command
    info/              reference for a top-level command without sub commands
    <group>/mod.rs     one directory per top-level group
tests/
  mock_supervisor.py mock Supervisor API (records requests, canned responses)
  parity.py          parity runner
  tree_diff.py       command tree / help diff
  parity/cases/<group>.txt   parity cases, one file per group
  parity/fixtures/           response bodies for the `file:` preset
```

Rules for working on a group in parallel with others:

* Only touch `src/cmd/<your group>/` and `tests/parity/cases/<your group>.txt`
  (plus fixtures named `<group>_*.json`). Every group is already declared and
  registered in `src/cmd/mod.rs`; do not edit it or any other shared file.
* A group module exposes `pub fn command() -> Command`. Put each sub command
  in its own file with its own `pub fn command() -> Command` (nested groups
  can be sub directories).
* If something in the shared core is wrong or missing, keep the workaround
  local to your group and report it; do not change the core.
* Things other groups need from yours must be `pub` (e.g.
  `crate::cmd::os::...` for `factory-reset`, which runs `os datadisk wipe`).
  Completers used by several groups are already in `src/cmd/completions.rs`:
  `apps()` (Go `appsCompletions`), `store_apps()` (`storeAppCompletions`),
  `backups_locations()` (`backupsLocationsCompletions`), `host_boot()`
  (`hostBootCompletions`).

### How to add a command

Use `src/cmd/dns/` as the template; each file there is the port of the Go
file of the same name (`cmd/dns_update.go` -> `src/cmd/dns/update.rs`).

```rust
// src/cmd/dns/options.rs  (Go: cmd/dns_options.go)
pub fn command() -> Command {
    leaf("options")                                   // Use + Args: NoArgs + NoFileCompletions
        .aliases(&["option", "opt", "opts", "op"])    // Aliases
        .short("Allow to set options for ...")        // Short
        .long("\nThis command allows ...\n")          // Long   (verbatim!)
        .example("\n  ha dns options --servers ...\n")// Example (verbatim!)
        .flag(Flag::string_array("servers", "Upstream DNS servers ...").short('r').completion(comp::no_file()))
        .flag(Flag::bool("fallback", true, "Enable/Disable fallback DNS (Cloudflare DoT)").completion(comp::bools()))
        .run(Call::post("dns", "options").run_with(|ctx, options| {
            string_array_if_set(ctx, options, "servers", "servers");
            bool_if_changed(ctx, options, "fallback", "fallback");
        }))
}
```

Mapping from the Go/cobra source:

| Go | Rust |
| --- | --- |
| `Use: "update [slug]"` | `Command::new("update [slug]")` (or `leaf(..)`, see helpers) |
| `Aliases`, `Short`, `Long`, `Example` | `.aliases(&[..])`, `.short(..)`, `.long(..)`, `.example(..)` |
| no `Args:` field | nothing (`Args::Legacy`, the default) |
| `Args: cobra.NoArgs` / `ExactArgs(n)` / `MaximumNArgs(n)` | `.args(Args::None)` / `Args::Exact(n)` / `Args::Maximum(n)` (also `Minimum`, `Range`, `Arbitrary`) |
| `ValidArgsFunction: cobra.NoFileCompletions` | `.valid_args(comp::no_file())` |
| `ValidArgsFunction: appsCompletions` | `.valid_args(completions::apps())` |
| `Run: func(cmd, args) {..}` | `.run(\|ctx\| { ..; success })`: return `false` where Go sets `ExitWithError = true` |
| `PersistentPreRun` (deprecated alias warning) | `.persistent_pre_run(\|ctx\| { helpers::warn_deprecated_aliases(&["addons", ..], "apps"); crate::cmd::root_pre_run(ctx); })` |
| `Flags().String("name", "", "usage")` / `StringP(.., "n", ..)` | `.flag(Flag::string("name", "", "usage"))` / `...short('n')` |
| `Bool`, `Int`, `Uint32`, `StringArray` | `Flag::bool(name, default, usage)`, `Flag::int`, `Flag::uint32`, `Flag::string_array(name, usage)` |
| `Lookup("x").NoOptDefVal = ".local"` | `.no_opt_default(".local")` (bool flags have `"true"` implicitly, as in pflag) |
| `RegisterFlagCompletionFunc("x", boolCompletions)` | `.completion(comp::bools())` on the flag |
| `MarkFlagRequired("x")` | `.required()` on the flag |
| `Flags().MarkHidden("x")` / `MarkDeprecated("x", "msg")` | `.hidden()` / `.deprecated("msg")` on the flag |
| `MarkFlagsMutuallyExclusive("a", "b")` / `MarkFlagsOneRequired(..)` | `.mutually_exclusive(&["a", "b"])` / `.one_required(&[..])` on the command |
| `Flags().SetNormalizeFunc(.. "_" -> "-")` | `.normalize_flags(cobra::normalize_underscores)` |
| `cmd.Flags().GetString("x")`, `GetBool`, `GetInt`, `GetUint32`, `GetStringArray` | `ctx.string("x")`, `ctx.bool`, `ctx.int`, `ctx.uint32`, `ctx.string_array` |
| `cmd.Flags().Changed("x")` | `ctx.changed("x")` |
| `args[0]`, `len(args)` | `ctx.arg(0)`, `ctx.args.len()` |
| `cmd.Name()`, `cmd.CalledAs()` | `ctx.name()`, `ctx.called_as()` |
| `cmd.PrintErrln("Aborted:", err)` | `eprintln!("Aborted: {err}")` |
| `slog.Debug("dns info", "args", args)` | `helpers::debug_args(ctx)` (done by `Call::run*`), `debug!("msg", "key" => value)` |
| `helper.PrintError(err)` | `output::print_error(&err)` |
| `ProgressSpinner.Start()` / `.Stop()` | `spinner::start()` / `spinner::stop()` |

Things to watch:

* `Long` and `Example` must be copied byte for byte (leading/trailing newlines,
  tabs, trailing spaces); write a trailing space as `\x20` and a tab as `\t`
  so editors and `cargo fmt` cannot eat them. `tests/tree_diff.py <group>`
  compares the rendered help byte for byte and will tell you.
* Flag usage strings appear in help *and* in shell completion; copy them
  verbatim too.
* Replicate the Go logic literally, including quirks (which flags are sent
  when "changed" vs. "non-empty", when the spinner starts/stops, which
  timeout is used, messages printed to stdout vs. stderr).
* Go iterates maps in random order; iterate in sorted order instead.

### Helper API (`src/cmd/helpers.rs`, `src/cmd/logs.rs`, `src/client.rs`, `src/output.rs`)

Command shapes:

```rust
leaf("info")                       // Command::new + Args::None + NoFileCompletions

// GenericJSONGet + ShowJSONResponse (info, stats, ...)
.run(Call::get("dns", "info").run())
// GenericJSONPost, no body
.run(Call::post("supervisor", "reload").run())
// spinner + GenericJSONPostTimeout (restart, reset, ...)
.run(Call::post("dns", "reset").spinner().run())
.run(Call::post("dns", "restart").timeout(client::CONTAINER_OPERATION_TIMEOUT).spinner().run())
.run(run_restart("dns"))                          // the line above, as a shortcut
// POST with an options body built from flags (no body is sent if it stays empty)
.run(Call::post("dns", "options").run_with(|ctx, options| { ... }))
// plug-in update: --version flag, spinner, CONTAINER_DOWNLOAD_TIMEOUT
.flag(version_flag()).run(run_update("dns"))
// "{slug}" path parameter from positional arg 0 (SetPathParams)
.run(Call::post("addons", "{slug}/restart").arg_param("slug", 0).run())
// Go code that calls request.SetBody(options) unconditionally (sends "{}")
Call::post(..).always_body()
// logs: flags + Run
add_logs_flags(leaf("logs")...).run(run_logs("dns"))
```

Inside a hand-written `run` closure:

```rust
Call::post("backups", command).timeout(client::BACKUP_TIMEOUT).spinner().execute(ctx, options) // -> bool
let resp = Call::get("os", "info").request(ctx, Options::new());  // -> client::Result<Response>
output::show(result)                       // print error or ShowJSONResponse; -> bool
output::show_json_response(&mut resp)      // ShowJSONResponse
output::show_json_response_transform(&mut resp, Some(&|data| { ... }))
output::stream_text_response(&mut resp)    // StreamTextResponse
output::print_error(&err) / output::print_error_string("...")
```

Body helpers (`options` is `client::Options`, a `serde_json::Map`):
`bool_if_changed`, `string_if_changed`, `string_if_set` (non-empty),
`int_if_changed`, `string_array_if_set` (non-empty), each
`(ctx, options, "flag-name", "json_key")`. Anything else:
`options.insert("key".into(), Value::from(..))`.

Low-level client, mirroring the Go `client` package one to one:

| Go | Rust |
| --- | --- |
| `helper.URLHelper(section, command)` | `client::url_helper(section, command)?` |
| `helper.GetJSONRequest()` / `GetJSONRequestTimeout(t)` | `client::json_request()` / `client::json_request_timeout(t)` |
| `helper.GetRequest()` / `GetRequestTimeout(t)` | `client::request()` / `client::request_timeout(t)` |
| `request.SetPathParam(s)`, `SetQueryParam`, `SetHeader`, `SetBody(map)` | `.path_param(k, v)`, `.query_param(k, v)`, `.header(k, v)`, `.body(options)` (also `.body_json(&value)`, `.body_bytes(bytes, content_type)`) |
| `SetDoNotParseResponse(true)` | `.do_not_parse()` |
| `request.Get(url)` / `Post` / `Put` / `Delete` | `.get(&url)` / `.post(&url)` / `.put(&url)` / `.delete(&url)` (or `.execute("PATCH", &url)`) |
| `request.URL = ..; request.Send()` | `request.url = ..; request.method = ..; request.send()` |
| `helper.GenericJSONErrorHandling(resp, err)` | `client::generic_json_error_handling(result)` |
| `helper.GenericJSONGet/Post[Timeout]` | `client::generic_json_get/post[_timeout]` (`body: Option<Options>`) |
| `resp.IsSuccess()`, `resp.StatusCode()`, `resp.Body()` | `resp.is_success()`, `resp.status_code()`, `resp.body()` |
| `resp.Result().(*helper.Response)` | `resp.data` (`.result`, `.message`, `.data`) |
| `helper.RawJSON` | `client::raw_json()` |
| `helper.DefaultTimeout`, `ContainerOperationTimeout`, ... | `client::DEFAULT_TIMEOUT`, `CONTAINER_OPERATION_TIMEOUT`, `CONTAINER_DOWNLOAD_TIMEOUT`, `OS_DOWNLOAD_TIMEOUT`, `BACKUP_TIMEOUT`, `REBOOT_TIMEOUT`, `MOUNT_TIMEOUT`, `NO_TIMEOUT` |
| `helper.AskForConfirmation(prompt, tries)` | `prompt::ask_for_confirmation(prompt, tries)` |
| `helper.ReadInteger(..)` / `ReadPassword(repeat)` | `prompt::read_integer(..)` / `prompt::read_password(repeat)` |

Dynamic completion: a completer is `comp::from_fn(|ctx, to_complete| (candidates, Directive::NO_FILE_COMP))`
where candidates are `"value"` or `"value\tdescription"`. `ctx.args` are the
positional args typed so far (Go: `if len(args) != 0 { return nil, NoFileComp }`),
`ctx.name()` is the command being completed. `completions::api_data(section, command)`
fetches the `data` object of a successful response, `completions::describe(..)`
builds `value\tdesc, desc`. Attach with `.valid_args(completer)` on a command
or `.completion(completer)` on a flag. See `src/cmd/completions.rs`.

### Parity runner

`tests/parity.py` runs the Go binary and the Rust binary with the same
command line and environment against a mock Supervisor and diffs stdout,
stderr, exit code and the recorded HTTP requests (method, path, query,
`Accept` / `Authorization` / `Content-Type` / `Range` / `Content-Length`
headers, body).

```sh
python3 tests/parity.py dns            # one group  (tests/parity/cases/dns.txt)
python3 tests/parity.py dns -k logs    # only cases whose name contains "logs"
python3 tests/parity.py dns --show     # also print what the Go CLI did (handy while writing cases)
python3 tests/parity.py                # all case files
python3 tests/parity.py -x dns         # stop at first failure
```

It runs `cargo build` first (skip with `--no-build`) and builds the Go
reference binary from `../cli` into `target/go/ha` when `go` is installed
(nothing is written into the Go repository); without `go` it falls back to
the prebuilt `../cli/ha` and says so. Override with `HA_GO_REPO`,
`HA_GO_BIN`, `HA_RUST_BIN`.

Case file format (`tests/parity/cases/<group>.txt`; see `dns.txt`, and
`_core.txt` for config/env/completion examples):

```
== dns info                        # case name; also the command line unless "args:" is given
response: data                     # default response preset (default: ok)
== dns update fails
args: dns update --version 1       # shell quoting; "" is an empty argument
response: error:400:Version 1 not found
route: GET /dns/info = data:{"a":1}   # per-route preset; "p1 ;; p2" serves a sequence
stdin: YES\n                       # piped stdin (default: empty)
env: SUPERVISOR_RAW_JSON=true      # extra environment variable
unset: SUPERVISOR_TOKEN            # drop a default variable
file: cfg.yaml = raw-json: true\n  # create {tmp}/cfg.yaml
compare: stdout exit requests      # default: stdout stderr exit requests
```

Placeholders: `{endpoint}`, `{tmp}`, `{home}`. Response presets (`ok`,
`data`, `data:<json>`, `json:<body>`, `file:<fixture>`, `error[:status[:msg]]`,
`text:<status>[:body]`, `empty:<status>`, `logs`, `raw:<status>:<ctype>:<body>`)
are documented at the top of `tests/mock_supervisor.py`, which can also be
started standalone for manual testing
(`python3 tests/mock_supervisor.py --port 8123 --preset data`).

Write cases for every sub command: plain call, every alias once, every flag
(set / unset / `=false` forms), wrong args, an API error, `--raw-json`, and
`__complete ...` lines for arg and flag completion.

### Command tree diff

`tests/tree_diff.py` walks `--help` of both binaries recursively and compares
per command: sub commands, aliases, usage line, flags (name, shorthand, type,
default, usage), descriptions, examples, and the flag-name completion
(`__complete <path> -`).

```sh
python3 tests/tree_diff.py dns             # sub tree `ha dns ...`
python3 tests/tree_diff.py os boards       # sub tree `ha os boards ...`
python3 tests/tree_diff.py                 # whole tree
python3 tests/tree_diff.py --loose dns     # structure only, ignore help texts
```

Hidden commands and hidden/deprecated flags are invisible in help; cover them
with parity cases.

### Before handing in a group

```sh
cargo fmt && cargo clippy --all-targets && cargo test
python3 tests/parity.py <group>
python3 tests/tree_diff.py <group>
```

## Known deviations from the Go CLI

* HTTP framing details: `User-Agent` is `ha-cli-rs/<version>` instead of
  `go-resty/...`; body-less POSTs are sent with `Content-Length: 0` instead of
  an empty chunked body; no `Accept-Encoding: gzip` is sent (Go's transport
  adds it and decompresses transparently); header names are lower-case.
* No TLS: the binary is built without a TLS stack, so an `https://` endpoint
  fails with `https endpoints are not supported (built without TLS)`. The
  Supervisor API is plain HTTP; re-add ureq's `rustls` feature if needed.
* Transport error texts are reproduced for connection refused, unknown host
  (without Go's `on <resolver>` detail) and timeouts; other network/TLS
  errors carry the Rust library's wording after the Go-style
  `Get "<url>": ` prefix. JSON *syntax* error texts are approximations
  (`unexpected end of JSON input` and the `json: cannot unmarshal ...` type
  errors are exact).
* Debug/info log lines (`--log-level debug`) have the slog format but not the
  same set of messages.
* Config file: only flat `key: value` YAML and JSON are read
  (`$HOME/.homeassistant.{json,yaml,yml}` or `--config`); viper's TOML/INI/...
  formats and nested YAML are not supported.
* Boot IDs in `--boot` completion are sorted (Go: random map order).
* JSON responses with lone UTF-16 surrogate escapes or numbers beyond float64
  range are rejected with a different message.
