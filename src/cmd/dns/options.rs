use crate::cmd::helpers::{Call, bool_if_changed, leaf, string_array_if_set};
use crate::cobra::{Command, Flag, comp};
use crate::{debug, gofmt};

pub fn command() -> Command {
    leaf("options")
        .aliases(&["option", "opt", "opts", "op"])
        .short("Allow to set options for the internal Home Assistant DNS server")
        .long(
            "
This command allows you to set configuration options for the internally
running Home Assistant DNS server.
",
        )
        .example(
            "
  ha dns options --servers dns://8.8.8.8 --servers dns://1.1.1.1
",
        )
        .flag(
            Flag::string_array(
                "servers",
                "Upstream DNS servers to use. Use multiple times for multiple servers.",
            )
            .short('r')
            .completion(comp::no_file()),
        )
        .flag(
            Flag::bool("fallback", true, "Enable/Disable fallback DNS (Cloudflare DoT)")
                .no_opt_default("true")
                .completion(comp::bools()),
        )
        .run(Call::post("dns", "options").run_with(|ctx, options| {
            debug!("servers", "servers" => gofmt::list(&ctx.string_array("servers")));
            string_array_if_set(ctx, options, "servers", "servers");
            bool_if_changed(ctx, options, "fallback", "fallback");
        }))
}
