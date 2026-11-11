use super::{run_volume, volume_flags};
use crate::cmd::helpers::leaf;
use crate::cobra::Command;

pub fn command() -> Command {
    volume_flags(
        leaf("input")
            .aliases(&["in"])
            .short("Set volume of a Home Assistant Audio input channel")
            .long(
                "
This command allows you to set the volume of a Home Assistant Audio
input channel or application on your Home Assistant system.",
            )
            .example(
                "
\tha audio volume input --index 1 --mute
\tha audio volume input --index 1 --unmute
\tha audio volume input --index 1 --volume 75
\tha audio volume input --index 1 --mute --application
\tha audio volume input --index 1 --unmute --application
\tha audio volume input --index 2 --volume 50 --application
",
            ),
    )
    .run(run_volume("input"))
}
