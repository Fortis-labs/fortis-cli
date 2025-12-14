// ─────────────────────────────
// CLI / UX
// ─────────────────────────────
use clap::Args;
use colored::Colorize;
use dialoguer::Confirm;
use indicatif::ProgressBar;

// ─────────────────────────────
// Standard library
// ─────────────────────────────
use std::str::FromStr;
use std::time::Duration;

// ─────────────────────────────
// Solana SDK
// ─────────────────────────────
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_compute_budget_interface::ComputeBudgetInstruction;
use solana_sdk::{
    message::{v0::Message, VersionedMessage},
    pubkey::Pubkey,
    signature::Signer,
    transaction::VersionedTransaction,
};
use solana_system_interface::program::ID as SYS_PROGRAM_ID;

// ─────────────────────────────
// Fortis SDK
// ─────────────────────────────
use fortis_sdk::{
    client::{get_multisig, proposal_create},
    pda::{get_proposal_pda, get_transaction_pda, FORTIS_PROGRAM_ID},
    state::{ProposalCreateAccounts, VaultTransactionMessage},
};

// ─────────────────────────────
// Local utilities
// ─────────────────────────────
use crate::utils::{create_signer_from_path, send_and_confirm_transaction};

#[derive(Args)]
pub struct ProposalCreate {
    /// RPC URL
    #[arg(long)]
    rpc_url: Option<String>,

    /// Path to the Proposal Creator Keypair
    #[arg(long)]
    keypair: String,

    /// The multisig where the transaction has been proposed
    #[arg(long)]
    multisig_pubkey: String,

    #[arg(long)]
    voting_deadline: u64,

    #[arg(long)]
    transaction_message: Vec<u8>,

    #[arg(long)]
    priority_fee_lamports: Option<u64>,
}

impl ProposalCreate {
    pub async fn execute(self) -> eyre::Result<()> {
        let Self {
            rpc_url,
            keypair,
            multisig_pubkey,
            transaction_message,
            voting_deadline,
            priority_fee_lamports,
        } = self;

        let transaction_creator_keypair = create_signer_from_path(keypair).unwrap();

        let transaction_creator = transaction_creator_keypair.pubkey();

        let rpc_url = rpc_url.unwrap_or_else(|| "https://api.mainnet-beta.solana.com".to_string());
        let rpc_url_clone = rpc_url.clone();
        let rpc_client = &RpcClient::new(rpc_url);

        let multisig = Pubkey::from_str(&multisig_pubkey).expect("Invalid multisig address");

        let multisig_data = get_multisig(rpc_client, &multisig).await?;

        let transaction_index = multisig_data.transaction_index + 1;

        let transaction_pda = get_transaction_pda(&multisig, transaction_index, None);
        let proposal_pda = get_proposal_pda(&multisig, transaction_index, None);
        println!();
        println!(
            "{}",
            "👀 You're about to create a vault transaction, please review the details:".yellow()
        );
        println!();
        println!("RPC Cluster URL:   {}", rpc_url_clone);
        println!("Program ID:        {}", FORTIS_PROGRAM_ID.to_string());
        println!("Your Public Key:       {}", transaction_creator);
        println!();
        println!("⚙️ Config Parameters");
        println!("Multisig Key:       {}", multisig_pubkey);
        println!("Transaction Index:       {}", transaction_index);
        println!("Voting Deadline:       {}", voting_deadline);
        println!();

        let proceed = Confirm::new()
            .with_prompt("Do you want to proceed?")
            .default(false)
            .interact()?;
        if !proceed {
            println!("OK, aborting.");
            return Ok(());
        }
        println!();

        let progress = ProgressBar::new_spinner().with_message("Sending transaction...");
        progress.enable_steady_tick(Duration::from_millis(100));

        let blockhash = rpc_client
            .get_latest_blockhash()
            .await
            .expect("Failed to get blockhash");
        let transaction_message: VaultTransactionMessage =
            borsh::from_slice(&transaction_message).unwrap();

        let message = Message::try_compile(
            &transaction_creator,
            &[
                ComputeBudgetInstruction::set_compute_unit_price(
                    priority_fee_lamports.unwrap_or(5000),
                ),
                proposal_create(
                    ProposalCreateAccounts {
                        multisig,
                        trasaction: transaction_pda.0,
                        creator: transaction_creator,
                        proposal: proposal_pda.0,
                        system_program: SYS_PROGRAM_ID,
                    },
                    0,
                    &transaction_message,
                    voting_deadline as i64,
                    None,
                ),
            ],
            &[],
            blockhash,
        )
        .unwrap();

        let transaction = VersionedTransaction::try_new(
            VersionedMessage::V0(message),
            &[&*transaction_creator_keypair],
        )
        .expect("Failed to create transaction");

        let signature = send_and_confirm_transaction(&transaction, &rpc_client).await?;

        println!(
            "✅ Transaction created successfully. Signature: {}",
            signature.green()
        );
        Ok(())
    }
}
