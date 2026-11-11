use serde_json::Value;

use super::{backups_apps_completions, backups_folders_completions, string_array};
use crate::client::{self, Options};
use crate::cmd::completions;
use crate::cmd::helpers::{Call, bool_if_changed, debug_args, leaf};
use crate::cobra::{Command, Flag, comp};
use crate::{debug, gofmt};

pub fn command() -> Command {
    leaf("new")
        .aliases(&["create", "backup"])
        .short("Create a new Home Assistant backup")
        .long(
            "
This command can be used to trigger the creation of a new Home Assistant
backup.",
        )
        .example(
            "
  ha backups new
  ha backups new --app core_ssh --app core_mosquitto
  ha backups new --folders homeassistant
  ha backups new --uncompressed
",
        )
        .flag(Flag::string("name", "", "Name of the backup").completion(comp::no_file()))
        .flag(Flag::string("password", "", "Password").completion(comp::no_file()))
        .flag(
            Flag::bool("uncompressed", false, "Use uncompressed archives")
                .no_opt_default("false")
                .completion(comp::bools()),
        )
        .flag(
            Flag::string_array(
                "app",
                "App to backup, triggers a partial backup. Use multiple times for multiple apps.",
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
            Flag::string_array("folders", "Folders to backup, triggers a partial backup")
                .short('f')
                .completion(backups_folders_completions()),
        )
        .flag(
            Flag::string_array(
                "location",
                "Where to put backup file (backup mount or local). Use multiple times for multiple locations.",
            )
            .short('l')
            .no_opt_default(".local")
            .completion(completions::backups_locations()),
        )
        .flag(
            Flag::bool(
                "homeassistant-exclude-database",
                false,
                "Exclude the Home Assistant database file from backup",
            )
            .no_opt_default("false")
            .completion(comp::bools()),
        )
        .flag(Flag::string("filename", "", "Name to use for the backup file").completion(comp::no_file()))
        .run(|ctx| {
            debug_args(ctx);

            let mut command = "new/full";

            let mut options = Options::new();

            let name = ctx.string("name");
            if !name.is_empty() && ctx.changed("name") {
                options.insert("name".to_string(), Value::String(name));
            }

            let password = ctx.string("password");
            if !password.is_empty() && ctx.changed("password") {
                options.insert("password".to_string(), Value::String(password));
            }

            let mut apps = string_array(ctx, "app");
            apps.extend(string_array(ctx, "addons"));
            debug!("apps", "apps" => gofmt::list(&apps));

            if !apps.is_empty() && (ctx.changed("app") || ctx.changed("addons")) {
                options.insert("addons".to_string(), Value::from(apps));
                command = "new/partial";
            }

            let folders = string_array(ctx, "folders");
            debug!("folders", "folders" => gofmt::list(&folders));

            if !folders.is_empty() && ctx.changed("folders") {
                options.insert("folders".to_string(), Value::from(folders));
                command = "new/partial";
            }

            if ctx.changed("uncompressed") {
                options.insert("compressed".to_string(), Value::Bool(false));
            }

            let location = string_array(ctx, "location");
            debug!("location", "location" => gofmt::list(&location));
            if !location.is_empty() && ctx.changed("location") {
                options.insert("location".to_string(), Value::from(location));
            }

            let filename = ctx.string("filename");
            debug!("filename", "filename" => filename);
            if !filename.is_empty() && ctx.changed("filename") {
                options.insert("filename".to_string(), Value::String(filename));
            }

            bool_if_changed(
                ctx,
                &mut options,
                "homeassistant-exclude-database",
                "homeassistant_exclude_database",
            );

            Call::post("backups", command)
                .timeout(client::BACKUP_TIMEOUT)
                .spinner()
                .execute(ctx, options)
        })
}
