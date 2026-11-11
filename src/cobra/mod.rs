//! A small command framework that behaves like spf13/cobra + spf13/pflag.
//!
//! The Go CLI is built on cobra; to be a drop-in replacement the Rust port
//! needs the same command lookup rules, flag syntax, help/usage text, error
//! messages and shell-completion protocol (`__complete`). Rather than bending
//! a Rust CLI library into shape, this module implements exactly the subset
//! of cobra the Go CLI uses, with a builder API:
//!
//! ```ignore
//! Command::new("update")
//!     .aliases(&["upgrade", "downgrade", "up", "down"])
//!     .short("Updates the internal Home Assistant DNS server")
//!     .long("\nUsing this command ...\n")
//!     .example("\n  ha dns update --version 5\n")
//!     .args(Args::None)
//!     .valid_args(comp::no_file())
//!     .flag(Flag::string("version", "", "Version to update to").completion(comp::no_file()))
//!     .run(|ctx| { ...; true })
//! ```

pub mod comp;
pub mod complete;
mod exec;
pub mod flags;
mod help;

use std::rc::Rc;

#[allow(unused_imports)]
pub use complete::{Completer, Completion, Directive};
pub use exec::{execute, with_builtins};
pub use flags::{Flag, FlagSet};

/// Positional argument validation (cobra's `Args:` field).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Args {
    /// `Args` not set in Go ("legacy" behaviour): group commands accept
    /// anything, the root command rejects unknown sub commands.
    Legacy,
    /// `cobra.NoArgs`
    None,
    /// `cobra.ExactArgs(n)`
    Exact(usize),
    /// `cobra.MinimumNArgs(n)`
    Minimum(usize),
    /// `cobra.MaximumNArgs(n)`
    Maximum(usize),
    /// `cobra.RangeArgs(min, max)`
    Range(usize, usize),
    /// `cobra.ArbitraryArgs`
    Arbitrary,
}

/// `Run` function. Returns `true` on success; `false` makes the process exit
/// with status 1 (the Go code sets `ExitWithError = true`).
pub type RunFn = Rc<dyn Fn(&Ctx) -> bool>;
/// `PersistentPreRun` / `PersistentPostRun` hook.
pub type HookFn = Rc<dyn Fn(&Ctx)>;

/// A command definition (the equivalent of `cobra.Command`).
pub struct Command {
    /// Cobra's `Use`: the command name, optionally followed by an argument
    /// synopsis (`"update [slug]"`).
    pub use_line: String,
    pub aliases: Vec<String>,
    pub short: String,
    pub long: String,
    pub example: String,
    pub hidden: bool,
    pub args: Args,
    pub run: Option<RunFn>,
    pub persistent_pre_run: Option<HookFn>,
    pub persistent_post_run: Option<HookFn>,
    pub valid_args_fn: Option<Completer>,
    pub flags: Vec<Flag>,
    pub persistent_flags: Vec<Flag>,
    pub subcommands: Vec<Command>,
    pub(crate) mutually_exclusive: Vec<Vec<String>>,
    pub(crate) one_required: Vec<Vec<String>>,
    pub(crate) normalize: Option<flags::NormalizeFn>,
    pub(crate) disable_flag_parsing: bool,
    pub(crate) disable_flags_in_use_line: bool,
    pub(crate) is_help: bool,
}

impl Command {
    /// Create a command. `use_line` is cobra's `Use` string.
    pub fn new(use_line: &str) -> Command {
        Command {
            use_line: use_line.to_string(),
            aliases: Vec::new(),
            short: String::new(),
            long: String::new(),
            example: String::new(),
            hidden: false,
            args: Args::Legacy,
            run: None,
            persistent_pre_run: None,
            persistent_post_run: None,
            valid_args_fn: None,
            flags: Vec::new(),
            persistent_flags: Vec::new(),
            subcommands: Vec::new(),
            mutually_exclusive: Vec::new(),
            one_required: Vec::new(),
            normalize: None,
            disable_flag_parsing: false,
            disable_flags_in_use_line: false,
            is_help: false,
        }
    }

    /// `Aliases: []string{...}`
    pub fn aliases(mut self, aliases: &[&str]) -> Command {
        self.aliases = aliases.iter().map(|s| s.to_string()).collect();
        self
    }

    /// `Short: "..."`
    pub fn short(mut self, text: &str) -> Command {
        self.short = text.to_string();
        self
    }

    /// `Long: "..."`. Copy the Go raw string verbatim, including the leading
    /// and trailing newlines.
    pub fn long(mut self, text: &str) -> Command {
        self.long = text.to_string();
        self
    }

    /// `Example: "..."`. Copy the Go raw string verbatim.
    pub fn example(mut self, text: &str) -> Command {
        self.example = text.to_string();
        self
    }

    /// `Hidden: true`
    pub fn hidden(mut self) -> Command {
        self.hidden = true;
        self
    }

    /// `Args: cobra.NoArgs` etc. Leave unset when the Go command has no
    /// `Args` field.
    pub fn args(mut self, args: Args) -> Command {
        self.args = args;
        self
    }

    /// `Run: func(cmd, args) {...}`. Return `true` for success, `false` where
    /// Go sets `ExitWithError = true`.
    pub fn run(mut self, f: impl Fn(&Ctx) -> bool + 'static) -> Command {
        self.run = Some(Rc::new(f));
        self
    }

    /// `PersistentPreRun`. Like in cobra only the hook nearest to the executed
    /// command runs; call `crate::cmd::root_pre_run(ctx)` from an override.
    pub fn persistent_pre_run(mut self, f: impl Fn(&Ctx) + 'static) -> Command {
        self.persistent_pre_run = Some(Rc::new(f));
        self
    }

    /// `PersistentPostRun`
    pub fn persistent_post_run(mut self, f: impl Fn(&Ctx) + 'static) -> Command {
        self.persistent_post_run = Some(Rc::new(f));
        self
    }

    /// `ValidArgsFunction` (completion of positional arguments).
    pub fn valid_args(mut self, completer: Completer) -> Command {
        self.valid_args_fn = Some(completer);
        self
    }

    /// Add a local flag (`cmd.Flags()...`).
    pub fn flag(mut self, flag: Flag) -> Command {
        self.flags.push(flag);
        self
    }

    /// Add a persistent flag (`cmd.PersistentFlags()...`).
    pub fn persistent_flag(mut self, flag: Flag) -> Command {
        self.persistent_flags.push(flag);
        self
    }

    /// `AddCommand`
    pub fn subcommand(mut self, cmd: Command) -> Command {
        self.subcommands.push(cmd);
        self
    }

    /// `MarkFlagsMutuallyExclusive(names...)`
    pub fn mutually_exclusive(mut self, names: &[&str]) -> Command {
        self.mutually_exclusive
            .push(names.iter().map(|s| s.to_string()).collect());
        self
    }

    /// `MarkFlagsOneRequired(names...)`
    pub fn one_required(mut self, names: &[&str]) -> Command {
        self.one_required.push(names.iter().map(|s| s.to_string()).collect());
        self
    }

    /// `Flags().SetNormalizeFunc(...)`, e.g. to accept `--foo_bar` for
    /// `--foo-bar` (see [`normalize_underscores`]).
    pub fn normalize_flags(mut self, f: flags::NormalizeFn) -> Command {
        self.normalize = Some(f);
        self
    }

    /// `DisableFlagsInUseLine: true`
    pub fn disable_flags_in_use_line(mut self) -> Command {
        self.disable_flags_in_use_line = true;
        self
    }

    /// `DisableFlagParsing: true`
    pub fn disable_flag_parsing(mut self) -> Command {
        self.disable_flag_parsing = true;
        self
    }

    /// The command name: first word of the use line.
    pub fn name(&self) -> &str {
        self.use_line.split(' ').next().unwrap_or("")
    }

    pub(crate) fn has_alias(&self, s: &str) -> bool {
        self.aliases.iter().any(|a| a == s)
    }

    pub(crate) fn runnable(&self) -> bool {
        self.run.is_some()
    }

    pub(crate) fn is_help_command(&self) -> bool {
        self.is_help
    }

    /// cobra's `IsAvailableCommand`
    pub(crate) fn is_available(&self) -> bool {
        if self.hidden || self.is_help_command() {
            return false;
        }
        self.runnable() || self.subcommands.iter().any(|c| c.is_available())
    }

    pub(crate) fn has_available_subcommands(&self) -> bool {
        self.subcommands.iter().any(|c| c.is_available())
    }

    /// Sub commands sorted by name (cobra's `Commands()`).
    pub(crate) fn sorted_subcommands(&self) -> Vec<&Command> {
        let mut v: Vec<&Command> = self.subcommands.iter().collect();
        v.sort_by(|a, b| a.name().cmp(b.name()));
        v
    }

    pub(crate) fn find_subcommand(&self, name: &str) -> Option<&Command> {
        self.subcommands.iter().find(|c| c.name() == name || c.has_alias(name))
    }
}

/// The usual `SetNormalizeFunc` of the Go CLI: `_` and `-` are equivalent.
pub fn normalize_underscores(name: &str) -> String {
    name.replace('_', "-")
}

/// Execution context handed to `run` functions, hooks and completers: the
/// command being executed, its positional args and parsed flags (cobra's
/// `cmd *cobra.Command, args []string`).
pub struct Ctx<'a> {
    /// Command chain from the root command to the executed command.
    pub chain: Vec<&'a Command>,
    /// Positional arguments.
    pub args: Vec<String>,
    /// Parsed flags (local, persistent and inherited).
    pub flags: FlagSet,
    pub(crate) called_as: String,
}

impl<'a> Ctx<'a> {
    /// The executed command.
    pub fn cmd(&self) -> &'a Command {
        self.chain[self.chain.len() - 1]
    }

    /// The root command.
    pub fn root(&self) -> &'a Command {
        self.chain[0]
    }

    /// `cmd.Name()`
    pub fn name(&self) -> &'a str {
        self.cmd().name()
    }

    /// `cmd.CommandPath()`, e.g. `"ha dns info"`.
    pub fn command_path(&self) -> String {
        help::command_path(&self.chain)
    }

    /// `cmd.CalledAs()`: the name or alias used on the command line.
    pub fn called_as(&self) -> &str {
        &self.called_as
    }

    /// Positional argument `i` (empty string if missing).
    pub fn arg(&self, i: usize) -> &str {
        self.args.get(i).map_or("", |s| s.as_str())
    }

    /// `cmd.Flags().Changed(name)`
    pub fn changed(&self, name: &str) -> bool {
        self.flags.changed(name)
    }

    /// `cmd.Flags().GetString(name)`
    pub fn string(&self, name: &str) -> String {
        self.flags.get_string(name)
    }

    /// `cmd.Flags().GetBool(name)`
    pub fn bool(&self, name: &str) -> bool {
        self.flags.get_bool(name)
    }

    /// `cmd.Flags().GetInt(name)`
    pub fn int(&self, name: &str) -> i64 {
        self.flags.get_int(name)
    }

    /// `cmd.Flags().GetUint32(name)`
    pub fn uint32(&self, name: &str) -> u32 {
        self.flags.get_uint32(name)
    }

    /// `cmd.Flags().GetStringArray(name)`
    pub fn string_array(&self, name: &str) -> Vec<String> {
        self.flags.get_string_array(name)
    }
}
