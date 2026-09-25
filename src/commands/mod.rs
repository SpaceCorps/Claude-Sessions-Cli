mod list;
mod sessions;
mod transfer;

use crate::cli::Command;
use crate::error::Result;
use crate::readme;

pub fn run(command: Command) -> Result<()> {
    match command {
        Command::List => list::run(),
        Command::Sessions { profile } => sessions::run(profile.as_deref()),
        Command::Transfer(args) => transfer::run(&args),
        Command::AgentReadme => {
            readme::print();
            Ok(())
        }
    }
}
