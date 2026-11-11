//! Command lookup and execution with cobra's semantics (`Find`, `ExecuteC`,
//! `execute`).

use super::complete;
use super::flags::FlagSet;
use super::help::{build_flagset, command_path, help_string, usage_string, usage_string_with};
use super::{Args, Command, Ctx, comp};
use crate::gofmt::quote;

fn has_no_opt_def_val(name: &str, fs: &FlagSet) -> bool {
    fs.lookup(name).is_some_and(|f| !f.def.no_opt_def_val.is_empty())
}

fn short_has_no_opt_def_val(name: &str, fs: &FlagSet) -> bool {
    name.chars()
        .next()
        .and_then(|c| fs.shorthand_lookup(c))
        .is_some_and(|f| !f.def.no_opt_def_val.is_empty())
}

/// True if `s` is a flag that consumes the following argument as its value.
fn takes_next_arg(s: &str, fs: &FlagSet) -> bool {
    if s.contains('=') {
        return false;
    }
    (s.starts_with("--") && !has_no_opt_def_val(&s[2..], fs))
        || (s.starts_with('-') && s.len() == 2 && !short_has_no_opt_def_val(&s[1..], fs))
}

/// cobra's `stripFlags`: the non-flag arguments (candidate sub commands).
fn strip_flags(args: &[String], chain: &[&Command]) -> Vec<String> {
    if args.is_empty() {
        return Vec::new();
    }
    // Like cobra, the automatic help flag is not known at this point.
    let fs = build_flagset(chain, false);
    let mut commands = Vec::new();
    let mut i = 0;
    while i < args.len() {
        let s = &args[i];
        i += 1;
        if s == "--" {
            break;
        }
        if takes_next_arg(s, &fs) {
            // If '--flag arg' then delete arg from args.
            if args.len() - i <= 1 {
                break;
            }
            i += 1;
            continue;
        }
        if !s.is_empty() && !s.starts_with('-') {
            commands.push(s.clone());
        }
    }
    commands
}

/// cobra's `argsMinusFirstX`: remove the first non-flag occurrence of `x`.
fn args_minus_first_x(args: &[String], x: &str, chain: &[&Command]) -> Vec<String> {
    let fs = build_flagset(chain, false);
    let mut pos = 0;
    while pos < args.len() {
        let s = &args[pos];
        if s == "--" {
            break;
        }
        if takes_next_arg(s, &fs) {
            pos += 2;
            continue;
        }
        if !s.starts_with('-') && s == x {
            let mut ret = args[..pos].to_vec();
            ret.extend_from_slice(&args[pos + 1..]);
            return ret;
        }
        pos += 1;
    }
    args.to_vec()
}

/// Levenshtein distance (cobra's `ld`, case-insensitive).
fn ld(s: &str, t: &str) -> usize {
    let s = s.to_lowercase().into_bytes();
    let t = t.to_lowercase().into_bytes();
    let mut d = vec![vec![0usize; t.len() + 1]; s.len() + 1];
    for (i, row) in d.iter_mut().enumerate() {
        row[0] = i;
    }
    for (j, cell) in d[0].iter_mut().enumerate() {
        *cell = j;
    }
    for j in 1..=t.len() {
        for i in 1..=s.len() {
            d[i][j] = if s[i - 1] == t[j - 1] {
                d[i - 1][j - 1]
            } else {
                d[i - 1][j].min(d[i][j - 1]).min(d[i - 1][j - 1]) + 1
            };
        }
    }
    d[s.len()][t.len()]
}

fn find_suggestions(cmd: &Command, arg: &str) -> String {
    let mut suggestions = Vec::new();
    for sub in &cmd.subcommands {
        if sub.is_available() {
            let by_distance = ld(arg, sub.name()) <= 2;
            let by_prefix = sub.name().to_lowercase().starts_with(&arg.to_lowercase());
            if by_distance || by_prefix {
                suggestions.push(sub.name());
            }
        }
    }
    if suggestions.is_empty() {
        return String::new();
    }
    let mut out = String::from("\n\nDid you mean this?\n");
    for s in suggestions {
        out.push_str(&format!("\t{s}\n"));
    }
    out
}

pub(crate) struct Found<'a> {
    pub chain: Vec<&'a Command>,
    pub args: Vec<String>,
    pub called_as: String,
}

/// cobra's `Find`: walk down the command tree following the non-flag args.
pub(crate) fn find<'a>(root: &'a Command, args: &[String]) -> Result<Found<'a>, (Found<'a>, String)> {
    let mut chain = vec![root];
    let mut args = args.to_vec();
    let mut called_as = root.name().to_string();
    loop {
        let stripped = strip_flags(&args, &chain);
        let Some(next) = stripped.first() else { break };
        let cmd = chain[chain.len() - 1];
        let Some(sub) = cmd.find_subcommand(next) else { break };
        args = args_minus_first_x(&args, next, &chain);
        called_as = next.clone();
        chain.push(sub);
    }
    let cmd = chain[chain.len() - 1];
    let mut error = None;
    if cmd.args == Args::Legacy && !cmd.subcommands.is_empty() && chain.len() == 1 {
        // legacyArgs: root command with subcommands, do subcommand validity checking
        let stripped = strip_flags(&args, &chain);
        if let Some(first) = stripped.first() {
            error = Some(format!(
                "unknown command {} for {}{}",
                quote(first),
                quote(&command_path(&chain)),
                find_suggestions(cmd, first)
            ));
        }
    }
    let found = Found { chain, args, called_as };
    match error {
        Some(e) => Err((found, e)),
        None => Ok(found),
    }
}

fn validate_args(cmd: &Command, path: &str, args: &[String]) -> Result<(), String> {
    let n = args.len();
    match cmd.args {
        Args::Legacy | Args::Arbitrary => Ok(()),
        Args::None if n > 0 => Err(format!("unknown command {} for {}", quote(&args[0]), quote(path))),
        Args::Exact(want) if n != want => Err(format!("accepts {want} arg(s), received {n}")),
        Args::Minimum(min) if n < min => Err(format!("requires at least {min} arg(s), only received {n}")),
        Args::Maximum(max) if n > max => Err(format!("accepts at most {max} arg(s), received {n}")),
        Args::Range(min, max) if n < min || n > max => {
            Err(format!("accepts between {min} and {max} arg(s), received {n}"))
        }
        _ => Ok(()),
    }
}

fn validate_required_flags(fs: &FlagSet) -> Result<(), String> {
    let missing: Vec<&str> = fs
        .sorted()
        .into_iter()
        .filter(|f| f.def.required && !f.changed)
        .map(|f| f.def.name.as_str())
        .collect();
    if missing.is_empty() {
        Ok(())
    } else {
        Err(format!("required flag(s) \"{}\" not set", missing.join("\", \"")))
    }
}

fn validate_flag_groups(cmd: &Command, fs: &FlagSet) -> Result<(), String> {
    let set_flags = |group: &[String]| -> Option<Vec<String>> {
        // Groups naming unknown flags are ignored, like in cobra.
        if !group.iter().all(|n| fs.lookup(n).is_some()) {
            return None;
        }
        let mut set: Vec<String> = group.iter().filter(|n| fs.changed(n)).cloned().collect();
        set.sort();
        Some(set)
    };
    let sorted = |groups: &[Vec<String>]| {
        let mut g = groups.to_vec();
        g.sort_by_key(|names| names.join(" "));
        g
    };
    for group in sorted(&cmd.one_required) {
        if let Some(set) = set_flags(&group)
            && set.is_empty()
        {
            return Err(format!(
                "at least one of the flags in the group [{}] is required",
                group.join(" ")
            ));
        }
    }
    for group in sorted(&cmd.mutually_exclusive) {
        if let Some(set) = set_flags(&group)
            && set.len() > 1
        {
            return Err(format!(
                "if any flags in the group [{}] are set none of the others can be; [{}] were all set",
                group.join(" "),
                set.join(" ")
            ));
        }
    }
    Ok(())
}

/// Print `Error: <msg>` followed by the usage of the command (cobra's
/// behaviour for errors raised while executing a command).
fn usage_error(chain: &[&Command], message: String) -> Result<bool, String> {
    eprintln!("Error: {message}");
    eprintln!("{}", usage_string(chain));
    Err(message)
}

/// Run the command line. Returns `Ok(success)` where `success` is the `Run`
/// function's result (`!ExitWithError`), or `Err(message)` for usage errors
/// (cobra's `Execute()` returning an error; already reported cobra-style on
/// stderr, the caller prints it once more like the Go `Execute()` does).
///
/// `initialize` is cobra's `OnInitialize` hook: it runs after flag parsing
/// and before argument validation of a runnable command.
pub fn execute(root: &Command, args: &[String], initialize: &dyn Fn(&FlagSet)) -> Result<bool, String> {
    let found = match find(root, args) {
        Ok(found) => found,
        Err((found, message)) => {
            eprintln!("Error: {message}");
            eprintln!("Run '{} --help' for usage.", command_path(&found.chain));
            return Err(message);
        }
    };
    let chain = found.chain;
    let cmd = chain[chain.len() - 1];

    let mut flags = build_flagset(&chain, true);
    let args = if cmd.disable_flag_parsing {
        found.args
    } else {
        if let Err(message) = flags.parse(&found.args) {
            return usage_error(&chain, message);
        }
        // Warnings collected while parsing (deprecated flags).
        eprint!("{}", flags.output);
        if flags.get_bool("help") {
            print!("{}", help_string(&chain));
            return Ok(true);
        }
        flags.args().to_vec()
    };

    if !cmd.runnable() {
        print!("{}", help_string(&chain));
        return Ok(true);
    }

    initialize(&flags);

    if let Err(message) = validate_args(cmd, &command_path(&chain), &args) {
        return usage_error(&chain, message);
    }

    let ctx = Ctx {
        chain: chain.clone(),
        args,
        flags,
        called_as: found.called_as,
    };

    // Only the PersistentPreRun nearest to the command runs.
    if let Some(hook) = chain.iter().rev().find_map(|c| c.persistent_pre_run.as_ref()) {
        hook(&ctx);
    }

    if !cmd.disable_flag_parsing {
        if let Err(message) = validate_required_flags(&ctx.flags) {
            return usage_error(&chain, message);
        }
        if let Err(message) = validate_flag_groups(cmd, &ctx.flags) {
            return usage_error(&chain, message);
        }
    }

    let success = match &cmd.run {
        Some(run) => run(&ctx),
        None => true,
    };

    if let Some(hook) = chain.iter().rev().find_map(|c| c.persistent_post_run.as_ref()) {
        hook(&ctx);
    }
    Ok(success)
}

/// cobra's default `help` command.
fn help_command(root_name: &str) -> Command {
    let mut cmd = Command::new("help [command]")
        .short("Help about any command")
        .long(&format!(
            "Help provides help for any command in the application.\nSimply type {root_name} help [path to command] for full details."
        ))
        .valid_args(comp::from_fn(|ctx, to_complete| {
            let Ok(found) = find(ctx.root(), &ctx.args) else {
                return (Vec::new(), super::Directive::NO_FILE_COMP);
            };
            let cmd = found.chain[found.chain.len() - 1];
            let completions = cmd
                .sorted_subcommands()
                .into_iter()
                .filter(|sub| (sub.is_available() || sub.is_help_command()) && sub.name().starts_with(to_complete))
                .map(|sub| format!("{}\t{}", sub.name(), sub.short))
                .collect();
            (completions, super::Directive::NO_FILE_COMP)
        }))
        .run(|ctx| {
            match find(ctx.root(), &ctx.args) {
                Ok(found) => print!("{}", help_string(&found.chain)),
                Err(_) => {
                    let topics: Vec<String> = ctx.args.iter().map(|a| format!("`{a}`")).collect();
                    eprintln!("Unknown help topic [{}]", topics.join(" "));
                    eprint!("{}", usage_string_with(&[ctx.root()], false));
                }
            }
            true
        });
    cmd.is_help = true;
    cmd
}

/// Add cobra's automatic commands to the root command: `help`, `completion`
/// and the hidden `__complete` used by the shell completion scripts.
///
/// `on_completion_flags` is called during `__complete` once the flags of the
/// command line being completed are parsed, right before a dynamic completer
/// runs (so that e.g. `--endpoint` given on that command line is honoured).
pub fn with_builtins(root: Command, on_completion_flags: impl Fn(&FlagSet) + 'static) -> Command {
    let name = root.name().to_string();
    root.subcommand(help_command(&name))
        .subcommand(complete::completion_command(&name))
        .subcommand(complete::complete_command(on_completion_flags))
}
