use crate::cmd::helpers::{Call, leaf};
use crate::cobra::Command;

pub fn command() -> Command {
    leaf("reload")
        .aliases(&["refresh", "re"])
        .short("Reload the files on disk to check for new or removed backups")
        .long(
            "
If a backup has been manually placed inside the backup folder, or has been
removed manually, this command can trigger Home Assistant to re-read the files
on disk",
        )
        .example(
            "
  ha backups reload",
        )
        .run(Call::post("backups", "reload").run())
}
