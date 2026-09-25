use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "claude-sessions",
    version,
    about = "Transfer Claude desktop Code-tab sessions between accounts and organizations",
    long_about = "The Claude desktop app keeps a separate Code-tab session list for every account and \
organization. Sessions started under one login vanish from the sidebar after you sign in with another. \
`claude-sessions transfer` copies them into the list the app is showing now."
)]
pub struct Cli {
    /// Emit JSON instead of YAML (stdout and the stderr error envelope).
    #[arg(long, global = true)]
    pub json: bool,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// List the account/organization profiles that have sessions on this machine.
    List,
    /// List sessions in every profile, or in the ones matching ACCOUNT[/ORG].
    Sessions {
        /// Profile as ACCOUNT[/ORG]; prefixes of either UUID work.
        #[arg(value_name = "ACCOUNT[/ORG]")]
        profile: Option<String>,
    },
    /// Copy sessions from other profiles into the signed-in one.
    Transfer(TransferArgs),
    /// Print the operating manual for AI agents.
    AgentReadme,
}

#[derive(Args)]
pub struct TransferArgs {
    /// Source profile as ACCOUNT[/ORG]; repeatable, prefixes work. Default: every other profile.
    #[arg(long, value_name = "ACCOUNT[/ORG]")]
    pub from: Vec<String>,

    /// Destination profile as ACCOUNT/ORG; prefixes work. Default: the signed-in account's active organization.
    #[arg(long, value_name = "ACCOUNT/ORG")]
    pub to: Option<String>,

    /// Only transfer this session (id or id prefix, with or without `local_`); repeatable.
    #[arg(long = "session", value_name = "ID")]
    pub sessions: Vec<String>,

    /// Leave archived sessions behind.
    #[arg(long)]
    pub skip_archived: bool,

    /// Report what would be copied without writing anything.
    #[arg(long)]
    pub dry_run: bool,
}
