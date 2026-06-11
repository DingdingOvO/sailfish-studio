pub mod check;
pub mod new_project;
pub mod pack;
pub mod run;

use crate::error::Result;

/// Dispatch a command based on CLI arguments.
pub fn dispatch(cmd: crate::CliCommand) -> Result<()> {
    match cmd {
        crate::CliCommand::Run { file, headed, fps, width, height } => {
            run::execute(&file, headed, fps, width, height)
        }
        crate::CliCommand::Pack {
            file,
            output,
            embed_runtime,
        } => pack::execute(&file, output.as_deref(), embed_runtime),
        crate::CliCommand::New {
            name,
            template,
            dir,
        } => new_project::execute(&name, template.as_deref(), dir.as_deref()),
        crate::CliCommand::Check { file, strict } => check::execute(&file, strict),
    }
}
