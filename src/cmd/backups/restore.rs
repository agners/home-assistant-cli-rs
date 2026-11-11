use serde_json::Value;

use super::{backups_apps_completions, backups_completions, backups_folders_completions, string_array};
use crate::client::{self, Options};
use crate::cmd::completions;
use crate::cmd::helpers::{Call, bool_if_changed, debug_args, string_if_changed};
use crate::cobra::{Args, Command, Flag, comp};
use crate::{debug, gofmt};

pub fn command() -> Command {
    Command::new("restore [slug]")
        .short("Restores a Home Assistant backup")
        .long(
            "
When something goes wrong, this command allows you to restore a previously
take Home Assistant backup on your system.",
        )
        .example(
            "
  ha backups restore c1a07617
  ha backups restore c1a07617 --app core_ssh --app core_mosquitto
  ha backups restore c1a07617 --folders homeassistant",
        )
        .valid_args(backups_completions())
        .args(Args::Exact(1))
        .flag(Flag::string("password", "", "Password").completion(comp::no_file()))
        .flag(
            Flag::bool(
                "homeassistant",
                true,
                "Restore homeassistant (default true), performs a partial restore when set to false",
            )
            .completion(comp::bools()),
        )
        .flag(
            Flag::string_array(
                "app",
                "App to restore, performs a partial restore. Use multiple times for multiple apps.",
            )
            .short('a')
            .completion(backups_apps_completions()),
        )
        .flag(
            Flag::string_array("addons", "")
                .hidden()
                .deprecated("use --app instead"),
        )
        .flag(
            Flag::string_array("folders", "Folders to restore, performs a partial restore")
                .short('f')
                .completion(backups_folders_completions()),
        )
        .flag(
            Flag::string("location", "", "Where to put the backup file (backup mount or local)")
                .short('l')
                .no_opt_default(".local")
                .completion(completions::backups_locations()),
        )
        .run(|ctx| {
            debug_args(ctx);

            let mut command = "restore/full";

            let mut options = Options::new();

            let password = ctx.string("password");
            if !password.is_empty() && ctx.changed("password") {
                options.insert("password".to_string(), Value::String(password));
            }

            if ctx.changed("homeassistant") {
                bool_if_changed(ctx, &mut options, "homeassistant", "homeassistant");
                command = "restore/partial";
            }

            let mut apps = string_array(ctx, "app");
            apps.extend(string_array(ctx, "addons"));
            debug!("apps", "apps" => gofmt::list(&apps));

            if !apps.is_empty() {
                options.insert("addons".to_string(), Value::from(apps));
                command = "restore/partial";
            }

            let folders = string_array(ctx, "folders");
            debug!("folders", "folders" => gofmt::list(&folders));

            if !folders.is_empty() {
                options.insert("folders".to_string(), Value::from(folders));
                command = "restore/partial";
            }

            string_if_changed(ctx, &mut options, "location", "location");

            Call::post("backups/{slug}", command)
                .arg_param("slug", 0)
                .timeout(client::BACKUP_TIMEOUT)
                .spinner()
                .execute(ctx, options)
        })
}
