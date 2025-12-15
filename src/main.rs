use clap::Parser;
use command::Command;
mod command;
pub mod utils;

#[derive(Parser)]
struct App {
    #[command(subcommand)]
    command: Command,
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let app = App::parse();

    match app.command {
        Command::MultisigCreate(command) => command.execute().await,
        Command::ProposalApprove(command) => command.execute().await,
        Command::ProposalExecute(command) => command.execute().await,
        Command::ProposalCreate(command) => command.execute().await,
        Command::ProposalAccountsClose(command) => command.execute().await,
        Command::InitiateTransfer(command) => command.execute().await,
        Command::InitiateNativeTransfer(command) => command.execute().await,
        Command::InitiateProgramUpgrade(command) => command.execute().await,
        Command::DisplayVault(command) => command.execute().await,
        Command::DisplayMultisig(command) => command.execute().await,
    }
}
