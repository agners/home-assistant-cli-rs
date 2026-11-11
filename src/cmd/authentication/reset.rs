use serde_json::Value;

use super::debug_args;
use crate::client::{self, Options};
use crate::cmd::helpers::leaf;
use crate::cobra::{Command, Flag, comp};
use crate::{output, prompt};

struct User {
    username: String,
    name: String,
    owner: bool,
    active: bool,
    local_only: bool,
}

const MSG_LIMITED_ACCESS: &str = "For security reasons, this command works only from the operating system terminal.";

/// Go's `getUsers` uses unchecked type assertions, which panic on a
/// malformed user list; here that is reported as an error instead.
fn malformed(what: &str) -> client::Error {
    client::Error::new(format!("unexpected response from Supervisor: {what}"))
}

fn get_users() -> client::Result<Vec<User>> {
    let resp = client::generic_json_get("auth", "list")?;

    let data = &resp.data;
    if !resp.is_success() || data.result != "ok" {
        return Err(client::Error::new(format!(
            "error returned from Supervisor: {}",
            data.message
        )));
    }

    let users = data
        .data
        .get("users")
        .and_then(Value::as_array)
        .ok_or_else(|| malformed("users is not a list"))?;
    let mut result = Vec::new();
    for user in users {
        let user = user.as_object().ok_or_else(|| malformed("user is not an object"))?;
        let string = |key: &str| {
            user.get(key)
                .and_then(Value::as_str)
                .map(str::to_string)
                .ok_or_else(|| malformed(&format!("{key} is not a string")))
        };
        let boolean = |key: &str| {
            user.get(key)
                .and_then(Value::as_bool)
                .ok_or_else(|| malformed(&format!("{key} is not a boolean")))
        };
        result.push(User {
            username: string("username")?,
            name: string("name")?,
            owner: boolean("is_owner")?,
            active: boolean("is_active")?,
            local_only: boolean("local_only")?,
        });
    }
    Ok(result)
}

fn list_users(users: &[User]) {
    for (i, user) in users.iter().enumerate() {
        println!("- {}: {} ({})", i + 1, user.username, user.name);
        println!(
            "     owner: {}, active: {}, local only: {}",
            user.owner, user.active, user.local_only
        );
    }
}

pub fn command() -> Command {
    leaf("reset")
        .aliases(&["rst", "change"])
        .short("Reset the password of a Home Assistant user.")
        .long(
            "
This command allows you to change a password of a Home Assistant user.
Please note, this command is limited due to security reasons, and will
only work on some locations. For example, the Operating System CLI.
",
        )
        .example(
            "
  ha authentication reset --username \"JohnDoe\" --password \"123SuperSecret!\"
  ha authentication reset --interactive
",
        )
        .flag(Flag::string("username", "", "Username to reset the password for").completion(comp::no_file()))
        .flag(
            Flag::string("password", "", "New password to assign. Ignored in interactive mode")
                .completion(comp::no_file()),
        )
        .flag(Flag::bool(
            "interactive",
            false,
            "Use interactive prompt for entering username and/or password",
        ))
        .one_required(&["username", "interactive"])
        .one_required(&["password", "interactive"])
        .run(|ctx| {
            debug_args(ctx);

            let mut options = Options::new();

            for value in ["username", "password"] {
                let val = ctx.string(value);
                if !val.is_empty() && ctx.changed(value) {
                    options.insert(value.to_string(), Value::String(val));
                }
            }

            if ctx.bool("interactive") {
                // prompt for user if not given
                if let Some(username) = options.get("username").and_then(Value::as_str) {
                    println!("Changing password for user '{username}'");
                } else {
                    let users = match get_users() {
                        Ok(users) => users,
                        Err(err) => {
                            eprintln!("{MSG_LIMITED_ACCESS}");
                            output::print_error(&err);
                            return false;
                        }
                    };

                    if users.is_empty() {
                        eprintln!("No users configured.");
                        return false;
                    }

                    println!("List of users:");
                    list_users(&users);

                    let idx =
                        match prompt::read_integer("Select a user to reset the password for", 3, 1, users.len() as i64)
                        {
                            Ok(idx) => idx,
                            Err(err) => {
                                eprintln!("Aborted: {err}");
                                return false;
                            }
                        };

                    let user = &users[(idx - 1) as usize];
                    println!("Changing password for user {idx}: {} ({})", user.username, user.name);
                    options.insert("username".to_string(), Value::String(user.username.clone()));
                }

                // prompt for password
                match prompt::read_password(true) {
                    Ok(password) => {
                        options.insert("password".to_string(), Value::String(password));
                    }
                    Err(err) => {
                        output::print_error_string(&format!("Failed to set password: {err}\n"));
                        return false;
                    }
                }
            }

            // change the password
            match client::generic_json_post("auth", "reset", Some(options)) {
                Ok(mut resp) => output::show_json_response(&mut resp),
                Err(err) => {
                    eprintln!("{MSG_LIMITED_ACCESS}");
                    output::print_error(&err);
                    false
                }
            }
        })
}
