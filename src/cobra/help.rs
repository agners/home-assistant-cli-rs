//! Help and usage rendering, byte-compatible with cobra's default templates.

use super::Command;
use super::flags::{Flag, FlagSet};

/// `CommandPath()`: names of the chain joined by spaces.
pub(crate) fn command_path(chain: &[&Command]) -> String {
    chain.iter().map(|c| c.name()).collect::<Vec<_>>().join(" ")
}

/// Build the merged flag set of the last command in `chain`: its local
/// flags, its persistent flags, the persistent flags of all parents and
/// (optionally) the automatic `--help` flag.
pub(crate) fn build_flagset(chain: &[&Command], with_help: bool) -> FlagSet {
    let cmd = chain[chain.len() - 1];
    let mut fs = FlagSet {
        normalize: cmd.normalize,
        ..FlagSet::default()
    };
    for f in &cmd.flags {
        fs.add(f, false, false);
    }
    for f in &cmd.persistent_flags {
        fs.add(f, false, true);
    }
    for parent in chain[..chain.len() - 1].iter().rev() {
        for f in &parent.persistent_flags {
            fs.add(f, true, true);
        }
    }
    if with_help && fs.lookup("help").is_none() {
        let name = cmd.name();
        let usage = format!("help for {}", if name.is_empty() { "this command" } else { name });
        let mut help = Flag::bool("help", false, &usage).short('h');
        help.builtin_help = true;
        fs.add(&help, false, false);
    }
    fs
}

fn rpad(s: &str, padding: usize) -> String {
    format!("{s:<padding$}")
}

/// `UseLine()`
fn use_line(chain: &[&Command], fs: &FlagSet) -> String {
    let cmd = chain[chain.len() - 1];
    let mut line = if chain.len() > 1 {
        format!("{} {}", command_path(&chain[..chain.len() - 1]), cmd.use_line)
    } else {
        cmd.use_line.clone()
    };
    if cmd.disable_flags_in_use_line {
        return line;
    }
    if fs.flags.iter().any(|f| !f.def.hidden) && !line.contains("[flags]") {
        line.push_str(" [flags]");
    }
    line
}

/// cobra's `IsAdditionalHelpTopicCommand`
fn is_additional_help_topic(cmd: &Command) -> bool {
    if cmd.runnable() || cmd.hidden {
        return false;
    }
    cmd.subcommands.iter().all(is_additional_help_topic)
}

/// `UsageString()` (cobra's default usage template).
pub(crate) fn usage_string(chain: &[&Command]) -> String {
    usage_string_with(chain, true)
}

/// Usage text; `with_help` tells whether the automatic `--help` flag has
/// been initialised (cobra adds it lazily, so a few code paths print the
/// usage without it).
pub(crate) fn usage_string_with(chain: &[&Command], with_help: bool) -> String {
    let cmd = chain[chain.len() - 1];
    let fs = build_flagset(chain, with_help);
    let path = command_path(chain);
    let mut w = String::from("Usage:");
    if cmd.runnable() {
        w.push_str(&format!("\n  {}", use_line(chain, &fs)));
    }
    if cmd.has_available_subcommands() {
        w.push_str(&format!("\n  {path} [command]"));
    }
    if !cmd.aliases.is_empty() {
        w.push_str("\n\nAliases:\n");
        w.push_str(&format!(
            "  {}",
            std::iter::once(cmd.name())
                .chain(cmd.aliases.iter().map(|s| s.as_str()))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    if !cmd.example.is_empty() {
        w.push_str("\n\nExamples:\n");
        w.push_str(&cmd.example);
    }
    if cmd.has_available_subcommands() {
        let padding = cmd
            .subcommands
            .iter()
            .map(|c| c.name().len())
            .max()
            .unwrap_or(0)
            .max(11);
        w.push_str("\n\nAvailable Commands:");
        for sub in cmd.sorted_subcommands() {
            if sub.is_available() || sub.is_help_command() {
                w.push_str(&format!("\n  {} {}", rpad(sub.name(), padding), sub.short));
            }
        }
    }
    if fs.flags.iter().any(|f| !f.inherited && !f.def.hidden) {
        w.push_str("\n\nFlags:\n");
        w.push_str(fs.usages(|f| !f.inherited).trim_end());
    }
    if fs.flags.iter().any(|f| f.inherited && !f.def.hidden) {
        w.push_str("\n\nGlobal Flags:\n");
        w.push_str(fs.usages(|f| f.inherited).trim_end());
    }
    if cmd.subcommands.iter().any(is_additional_help_topic) {
        let padding = cmd
            .subcommands
            .iter()
            .map(|c| path.len() + 1 + c.name().len())
            .max()
            .unwrap_or(0)
            .max(11);
        w.push_str("\n\nAdditional help topics:");
        for sub in cmd.sorted_subcommands() {
            if is_additional_help_topic(sub) {
                w.push_str(&format!(
                    "\n  {} {}",
                    rpad(&format!("{path} {}", sub.name()), padding),
                    sub.short
                ));
            }
        }
    }
    if cmd.has_available_subcommands() {
        w.push_str(&format!(
            "\n\nUse \"{path} [command] --help\" for more information about a command."
        ));
    }
    w.push('\n');
    w
}

/// The help text (cobra's default help template).
pub(crate) fn help_string(chain: &[&Command]) -> String {
    let cmd = chain[chain.len() - 1];
    let mut out = String::new();
    let text = if cmd.long.is_empty() { &cmd.short } else { &cmd.long };
    let text = text.trim_end();
    if !text.is_empty() {
        out.push_str(text);
        out.push_str("\n\n");
    }
    if cmd.runnable() || !cmd.subcommands.is_empty() {
        out.push_str(&usage_string(chain));
    }
    out
}
