//! Ready-made completers (`ValidArgsFunction` / `RegisterFlagCompletionFunc`
//! values) and a constructor for custom ones.

use std::rc::Rc;

use super::Ctx;
use super::complete::{Completer, Completion, Directive};

/// Wrap a closure as a completer. The closure gets the context of the command
/// line being completed (`ctx.args` are the positional args typed so far,
/// `ctx.name()` the command name, flags are parsed) and the word to complete.
/// Candidates may carry a description after a tab: `"core_ssh\tTerminal & SSH"`.
pub fn from_fn(f: impl Fn(&Ctx, &str) -> Completion + 'static) -> Completer {
    Rc::new(f)
}

/// `cobra.NoFileCompletions`
pub fn no_file() -> Completer {
    from_fn(|_, _| (Vec::new(), Directive::NO_FILE_COMP))
}

/// The Go CLI's `boolCompletions`: `true` / `false`.
pub fn bools() -> Completer {
    fixed(&["true", "false"])
}

/// A fixed list of candidates, without file completion.
pub fn fixed(values: &[&str]) -> Completer {
    let values: Vec<String> = values.iter().map(|s| s.to_string()).collect();
    from_fn(move |_, _| (values.clone(), Directive::NO_FILE_COMP))
}
