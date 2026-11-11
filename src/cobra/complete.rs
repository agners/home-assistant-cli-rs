//! Shell completion: cobra's `__complete` protocol and the `completion`
//! command. The generated shell scripts are cobra's own scripts (v1.10.2), so
//! existing completion setups keep working unchanged.

use std::rc::Rc;

use super::exec::find;
use super::flags::{FlagSet, FlagState, Kind};
use super::help::build_flagset;
use super::{Args, Command, Ctx, Flag, comp};

/// cobra's `ShellCompDirective` bit set.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Directive(pub u32);

impl Directive {
    pub const DEFAULT: Directive = Directive(0);
    pub const ERROR: Directive = Directive(1);
    pub const NO_SPACE: Directive = Directive(2);
    pub const NO_FILE_COMP: Directive = Directive(4);
    pub const FILTER_FILE_EXT: Directive = Directive(8);
    pub const FILTER_DIRS: Directive = Directive(16);
    pub const KEEP_ORDER: Directive = Directive(32);

    fn describe(self) -> String {
        let names = [
            (1, "ShellCompDirectiveError"),
            (2, "ShellCompDirectiveNoSpace"),
            (4, "ShellCompDirectiveNoFileComp"),
            (8, "ShellCompDirectiveFilterFileExt"),
            (16, "ShellCompDirectiveFilterDirs"),
            (32, "ShellCompDirectiveKeepOrder"),
        ];
        let set: Vec<&str> = names
            .iter()
            .filter(|(bit, _)| self.0 & bit != 0)
            .map(|(_, n)| *n)
            .collect();
        if set.is_empty() {
            return "ShellCompDirectiveDefault".to_string();
        }
        if self.0 >= 64 {
            return format!("ERROR: unexpected ShellCompDirective value: {}", self.0);
        }
        set.join(", ")
    }
}

impl std::ops::BitOr for Directive {
    type Output = Directive;
    fn bitor(self, rhs: Directive) -> Directive {
        Directive(self.0 | rhs.0)
    }
}

/// Completion candidates (optionally `"value\tdescription"`) and a directive.
pub type Completion = (Vec<String>, Directive);
/// A dynamic completion function, see [`comp::from_fn`].
pub type Completer = Rc<dyn Fn(&Ctx, &str) -> Completion>;

fn is_flag_arg(arg: &str) -> bool {
    (arg.len() >= 3 && arg.starts_with("--")) || (arg.len() >= 2 && arg.starts_with('-') && !arg[1..].starts_with('-'))
}

fn flag_name_completions(f: &FlagState, to_complete: &str) -> Vec<String> {
    let mut out = Vec::new();
    if f.def.hidden || !f.def.deprecated.is_empty() {
        return out;
    }
    let long = format!("--{}", f.def.name);
    if long.starts_with(to_complete) {
        out.push(format!("{long}\t{}", f.def.usage));
    }
    if let Some(c) = f.def.shorthand {
        let short = format!("-{c}");
        if short.starts_with(to_complete) {
            out.push(format!("{short}\t{}", f.def.usage));
        }
    }
    out
}

/// Flags in cobra's visiting order: inherited first, then the command's own.
fn ordered_flags(fs: &FlagSet) -> Vec<&FlagState> {
    let sorted = fs.sorted();
    let mut out: Vec<&FlagState> = sorted.iter().copied().filter(|f| f.inherited).collect();
    out.extend(sorted.iter().copied().filter(|f| !f.inherited));
    out
}

fn complete_required_flags(fs: &FlagSet, to_complete: &str) -> Vec<String> {
    ordered_flags(fs)
        .into_iter()
        .filter(|f| f.def.required && !f.changed)
        .flat_map(|f| flag_name_completions(f, to_complete))
        .collect()
}

/// cobra's `enforceFlagGroupsForCompletion`.
fn enforce_flag_groups(cmd: &Command, fs: &mut FlagSet) {
    let known = |fs: &FlagSet, group: &[String]| group.iter().all(|n| fs.lookup(n).is_some());
    for group in &cmd.one_required {
        if known(fs, group) && !group.iter().any(|n| fs.changed(n)) {
            for f in fs.flags.iter_mut().filter(|f| group.contains(&f.def.name)) {
                f.def.required = true;
            }
        }
    }
    for group in &cmd.mutually_exclusive {
        if !known(fs, group) {
            continue;
        }
        let set: Vec<String> = group.iter().filter(|n| fs.changed(n)).cloned().collect();
        for name in &set {
            for f in fs
                .flags
                .iter_mut()
                .filter(|f| group.contains(&f.def.name) && &f.def.name != name)
            {
                f.def.hidden = true;
            }
        }
    }
}

struct Outcome {
    completions: Vec<String>,
    directive: Directive,
    error: Option<String>,
}

fn outcome(completions: Vec<String>, directive: Directive) -> Outcome {
    Outcome {
        completions,
        directive,
        error: None,
    }
}

fn failure(message: String) -> Outcome {
    Outcome {
        completions: Vec::new(),
        directive: Directive::DEFAULT,
        error: Some(message),
    }
}

/// cobra's `getCompletions`.
fn get_completions(root: &Command, args: &[String], on_flags: &dyn Fn(&FlagSet)) -> Outcome {
    let to_complete = args[args.len() - 1].clone();
    let trimmed = &args[..args.len() - 1];

    let Ok(found) = find(root, trimmed) else {
        return failure(format!(
            "unable to find a command for arguments: [{}]",
            trimmed.join(" ")
        ));
    };
    let chain = found.chain;
    let cmd = chain[chain.len() - 1];
    let mut final_args = found.args;
    let mut to_complete = to_complete;
    let template = build_flagset(&chain, !cmd.disable_flag_parsing);

    // checkIfFlagCompletion
    let mut flag_name: Option<String> = None;
    let mut flag_error: Option<String> = None;
    if !cmd.disable_flag_parsing {
        let org_last_arg = to_complete.clone();
        let mut name = String::new();
        let mut trimmed_args = final_args.clone();
        let mut flag_with_equal = false;
        let mut is_flag_word = false;
        if to_complete.starts_with('-') {
            if let Some(index) = to_complete.find('=') {
                name = if to_complete[..index].starts_with("--") {
                    to_complete[2..index].to_string()
                } else {
                    to_complete[index.saturating_sub(1)..index].to_string()
                };
                to_complete = to_complete[index + 1..].to_string();
                flag_with_equal = true;
            } else {
                // Normal flag name completion.
                is_flag_word = true;
            }
        }
        if !is_flag_word {
            if name.is_empty()
                && let Some(prev) = final_args.last()
                && is_flag_arg(prev)
                && !prev.contains('=')
            {
                name = if let Some(long) = prev.strip_prefix("--") {
                    long.to_string()
                } else {
                    prev[prev.len() - 1..].to_string()
                };
                trimmed_args.pop();
            }
            if !name.is_empty() {
                // findFlag
                let state = if name.chars().count() == 1 {
                    name.chars().next().and_then(|c| template.shorthand_lookup(c))
                } else {
                    template.lookup(&name)
                };
                match state {
                    None => {
                        to_complete = org_last_arg;
                        flag_error = Some(format!("Subcommand '{}' does not support flag '{}'", cmd.name(), name));
                    }
                    Some(state) => {
                        if !flag_with_equal && !state.def.no_opt_def_val.is_empty() {
                            // The flag does not take the next word as its value.
                        } else {
                            flag_name = Some(state.def.name.clone());
                            final_args = trimmed_args;
                        }
                    }
                }
            } else {
                final_args = trimmed_args;
            }
        }
    }

    // Parse the flags of the command line typed so far.
    let mut flag_completion = true;
    let mut flags = template.clone();
    if !cmd.disable_flag_parsing {
        let mut probe = template.clone();
        let mut with_dash = final_args.clone();
        with_dash.push("--".to_string());
        let _ = probe.parse(&with_dash);
        let new_arg_count = probe.args().len();
        if let Err(e) = flags.parse(&final_args) {
            return failure(format!(
                "Error while parsing flags from args [{}]: {}",
                final_args.join(" "),
                e
            ));
        }
        if new_arg_count > flags.args().len() {
            // A "--" was typed before: everything after is a positional arg.
            flag_completion = false;
        }
    }
    if let Some(e) = flag_error
        && flag_completion
    {
        return failure(e);
    }

    if flags.lookup("help").is_some_and(|f| f.def.builtin_help && f.changed) {
        return outcome(Vec::new(), Directive::NO_FILE_COMP);
    }
    if !cmd.disable_flag_parsing {
        final_args = flags.args().to_vec();
    }

    let flag = if flag_completion {
        flag_name.and_then(|n| flags.lookup(&n).cloned())
    } else {
        None
    };
    if let Some(f) = &flag
        && !f.def.filename_ext.is_empty()
    {
        return outcome(f.def.filename_ext.clone(), Directive::FILTER_FILE_EXT);
    }

    enforce_flag_groups(cmd, &mut flags);

    let mut completions: Vec<String> = Vec::new();
    let mut directive;
    if flag.is_none() && to_complete.starts_with('-') && !to_complete.contains('=') && flag_completion {
        // Complete flag names.
        completions = complete_required_flags(&flags, &to_complete);
        if completions.is_empty() {
            for f in ordered_flags(&flags) {
                if !f.changed || f.def.kind == Kind::StringArray {
                    completions.extend(flag_name_completions(f, &to_complete));
                }
            }
        }
        directive = Directive::NO_FILE_COMP;
        if completions.len() == 1 && completions[0].ends_with('=') {
            directive = Directive::NO_SPACE;
        }
        if !cmd.disable_flag_parsing {
            return outcome(completions, directive);
        }
    } else {
        directive = Directive::DEFAULT;
        if flag.is_none() {
            let found_local_non_persistent = flags.flags.iter().any(|f| !f.inherited && !f.persistent && f.changed);
            if final_args.is_empty() && !found_local_non_persistent {
                for sub in cmd.sorted_subcommands() {
                    if sub.is_available() || sub.is_help_command() {
                        if sub.name().starts_with(&to_complete) {
                            completions.push(format!("{}\t{}", sub.name(), sub.short));
                        }
                        directive = Directive::NO_FILE_COMP;
                    }
                }
            }
            completions.extend(complete_required_flags(&flags, &to_complete));
        }
    }

    on_flags(&flags);
    let completer = match &flag {
        Some(f) => f.def.completer.clone(),
        None => cmd.valid_args_fn.clone(),
    };
    if let Some(completer) = completer {
        let ctx = Ctx {
            chain: chain.clone(),
            args: final_args,
            flags,
            called_as: found.called_as,
        };
        let (comps, d) = completer(&ctx, &to_complete);
        completions.extend(comps);
        directive = d;
    }
    outcome(completions, directive)
}

/// The hidden `__complete` command (alias `__completeNoDesc`).
pub(crate) fn complete_command(on_flags: impl Fn(&FlagSet) + 'static) -> Command {
    Command::new("__complete [command-line]")
        .aliases(&["__completeNoDesc"])
        .short("Request shell completion choices for the specified command-line")
        .long(
            "__complete is a special command that is used by the shell completion logic\n\
             to request completion choices for the specified command-line.",
        )
        .hidden()
        .disable_flags_in_use_line()
        .disable_flag_parsing()
        .args(Args::Minimum(1))
        .run(move |ctx| {
            let result = get_completions(ctx.root(), &ctx.args, &on_flags);
            if let Some(e) = &result.error {
                // cobra's CompErrorln
                let message = format!("[Debug] [Error] {e}\n");
                if let Some(path) = std::env::var_os("BASH_COMP_DEBUG_FILE").filter(|p| !p.is_empty())
                    && let Ok(mut file) = std::fs::OpenOptions::new().append(true).create(true).open(path)
                {
                    use std::io::Write;
                    let _ = file.write_all(message.as_bytes());
                }
                eprint!("{message}");
            }
            let mut no_descriptions = ctx.called_as() == "__completeNoDesc";
            if !no_descriptions {
                let var = |name: &str| {
                    let name: String = format!("{name}_COMPLETION_DESCRIPTIONS")
                        .to_uppercase()
                        .chars()
                        .map(|c| {
                            if c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_' {
                                c
                            } else {
                                '_'
                            }
                        })
                        .collect();
                    std::env::var(name).unwrap_or_default()
                };
                let mut value = var(ctx.root().name());
                if value.is_empty() {
                    value = var("COBRA");
                }
                if let Ok(descriptions) = super::flags::parse_bool(&value) {
                    no_descriptions = !descriptions;
                }
            }
            let mut out = String::new();
            for comp in &result.completions {
                let mut comp = comp.as_str();
                if no_descriptions {
                    comp = comp.split('\t').next().unwrap_or("");
                }
                let comp = comp.split('\n').next().unwrap_or("").trim();
                out.push_str(comp);
                out.push('\n');
            }
            out.push_str(&format!(":{}\n", result.directive.0));
            print!("{out}");
            eprintln!("Completion ended with directive: {}", result.directive.describe());
            true
        })
}

fn script(template: &str, name: &str, descriptions: bool) -> String {
    let var = name.replace(['-', ':'], "_");
    let env: String = name
        .to_uppercase()
        .chars()
        .map(|c| {
            if c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();
    template
        .replace(
            "@@COMPCMD@@",
            if descriptions { "__complete" } else { "__completeNoDesc" },
        )
        .replace("@@NAME36@@", &format!("{name:<36}"))
        .replace("@@ENV@@", &env)
        .replace("@@VAR@@", &var)
        .replace("@@NAME@@", name)
}

/// cobra's default `completion` command with its four shell sub commands.
pub(crate) fn completion_command(root_name: &str) -> Command {
    let n = root_name;
    let shell = |shell: &'static str, long: String, template: &'static str| {
        let name = root_name.to_string();
        Command::new(shell)
            .short(&format!("Generate the autocompletion script for {shell}"))
            .long(&long)
            .args(Args::None)
            .valid_args(comp::no_file())
            .flag(Flag::bool("no-descriptions", false, "disable completion descriptions"))
            .run(move |ctx| {
                print!("{}", script(template, &name, !ctx.bool("no-descriptions")));
                true
            })
    };
    let bash = shell(
        "bash",
        format!(
            "Generate the autocompletion script for the bash shell.\n\n\
This script depends on the 'bash-completion' package.\n\
If it is not installed already, you can install it via your OS's package manager.\n\n\
To load completions in your current shell session:\n\n\
\tsource <({n} completion bash)\n\n\
To load completions for every new session, execute once:\n\n\
#### Linux:\n\n\
\t{n} completion bash > /etc/bash_completion.d/{n}\n\n\
#### macOS:\n\n\
\t{n} completion bash > $(brew --prefix)/etc/bash_completion.d/{n}\n\n\
You will need to start a new shell for this setup to take effect.\n"
        ),
        include_str!("completions/bash.tmpl"),
    )
    .disable_flags_in_use_line();
    let zsh = shell(
        "zsh",
        format!(
            "Generate the autocompletion script for the zsh shell.\n\n\
If shell completion is not already enabled in your environment you will need\n\
to enable it.  You can execute the following once:\n\n\
\techo \"autoload -U compinit; compinit\" >> ~/.zshrc\n\n\
To load completions in your current shell session:\n\n\
\tsource <({n} completion zsh)\n\n\
To load completions for every new session, execute once:\n\n\
#### Linux:\n\n\
\t{n} completion zsh > \"${{fpath[1]}}/_{n}\"\n\n\
#### macOS:\n\n\
\t{n} completion zsh > $(brew --prefix)/share/zsh/site-functions/_{n}\n\n\
You will need to start a new shell for this setup to take effect.\n"
        ),
        include_str!("completions/zsh.tmpl"),
    );
    let fish = shell(
        "fish",
        format!(
            "Generate the autocompletion script for the fish shell.\n\n\
To load completions in your current shell session:\n\n\
\t{n} completion fish | source\n\n\
To load completions for every new session, execute once:\n\n\
\t{n} completion fish > ~/.config/fish/completions/{n}.fish\n\n\
You will need to start a new shell for this setup to take effect.\n"
        ),
        include_str!("completions/fish.tmpl"),
    );
    let powershell = shell(
        "powershell",
        format!(
            "Generate the autocompletion script for powershell.\n\n\
To load completions in your current shell session:\n\n\
\t{n} completion powershell | Out-String | Invoke-Expression\n\n\
To load completions for every new session, add the output of the above command\n\
to your powershell profile.\n"
        ),
        include_str!("completions/powershell.tmpl"),
    );
    Command::new("completion")
        .short("Generate the autocompletion script for the specified shell")
        .long(&format!(
            "Generate the autocompletion script for {n} for the specified shell.\n\
See each sub-command's help for details on how to use the generated script.\n"
        ))
        .args(Args::None)
        .valid_args(comp::no_file())
        .subcommand(bash)
        .subcommand(zsh)
        .subcommand(fish)
        .subcommand(powershell)
}
