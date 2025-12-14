use crate::command::display_vault::DisplayVault;
use crate::command::initiate_program_upgrade::InitiateProgramUpgrade;
use crate::command::initiate_transfer::InitiateTransfer;
use crate::command::multisig_create::MultisigCreate;
use crate::command::proposal_accounts_close::ProposalAccountsClose;
use crate::command::proposal_approve::ProposalApprove;
use crate::command::proposal_create::ProposalCreate;
use crate::command::vault_transaction_execute::ProposalExecute;

use clap::Subcommand;

pub mod display_vault;
pub mod initiate_program_upgrade;
pub mod initiate_transfer;
pub mod multisig_create;
pub mod proposal_accounts_close;
pub mod proposal_approve;
pub mod proposal_create;
pub mod vault_transaction_execute;

#[derive(Subcommand)]
pub enum Command {
    MultisigCreate(MultisigCreate),
    ProposalApprove(ProposalApprove),
    ProposalExecute(ProposalExecute),
    ProposalCreate(ProposalCreate),
    ProposalAccountsClose(ProposalAccountsClose),
    InitiateTransfer(InitiateTransfer),
    InitiateProgramUpgrade(InitiateProgramUpgrade),
    DisplayVault(DisplayVault),
}
