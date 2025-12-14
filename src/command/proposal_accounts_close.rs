// ─────────────────────────────
// Standard library
// ─────────────────────────────
use std::str::FromStr;
use std::time::Duration;

// ─────────────────────────────
// CLI / UX
// ─────────────────────────────
use clap::Args;
use colored::Colorize;
use dialoguer::Confirm;
use indicatif::ProgressBar;

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
    client::proposal_accounts_close,
    pda::{get_proposal_pda, get_transaction_pda, FORTIS_PROGRAM_ID},
    state::ProposalAccountsCloseAccounts,
};

// ─────────────────────────────
// Local utilities
// ─────────────────────────────
use crate::utils::{create_signer_from_path, send_and_confirm_transaction};

#[derive(Args)]
pub struct ProposalAccountsClose {
    /// RPC URL
    #[arg(long)]
    rpc_url: Option<String>,

    /// Path to the Keypair
    #[arg(long)]
    keypair: String,

    /// The multisig key
    #[arg(long)]
    multisig_pubkey: String,

    /// Index of the transaction to vote on
    #[arg(long)]
    transaction_index: u64,

    /// The proposal account key
    #[arg(long)]
    rent_collector: String,

    #[arg(long)]
    priority_fee_lamports: Option<u64>,
}

impl ProposalAccountsClose {
    pub async fn execute(self) -> eyre::Result<()> {
        let Self {
            rpc_url,
            keypair,
            multisig_pubkey,
            transaction_index,
            rent_collector,
            priority_fee_lamports,
        } = self;

        let multisig = Pubkey::from_str(&multisig_pubkey).expect("Invalid multisig key");

        let proposal_pda = get_proposal_pda(&multisig, transaction_index, None);

        let transaction_pda = get_transaction_pda(&multisig, transaction_index, None);

        let rent_collector_key =
            Pubkey::from_str(&rent_collector).expect("Invalid rent collector key");

        let transaction_creator_keypair = create_signer_from_path(keypair).unwrap();

        let transaction_creator = transaction_creator_keypair.pubkey();

        let rpc_url = rpc_url.unwrap_or_else(|| "https://api.mainnet-beta.solana.com".to_string());

        println!();
        println!(
            "{}",
            "👀 You're about to initialize ProgramConfig, please review the details:".yellow()
        );
        println!();
        println!("RPC Cluster URL:   {}", rpc_url);
        println!("Program ID:        {}", FORTIS_PROGRAM_ID.to_string());
        println!("Initializer:       {}", transaction_creator);
        println!();
        println!("⚙️ Config Parameters");
        println!();
        println!("Multisig Key:          {}", multisig_pubkey);
        println!("Transaction Index:      {}", transaction_index);
        println!("Rent reclamimer:      {}", rent_collector);
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

        let rpc_client = RpcClient::new(rpc_url);

        let progress = ProgressBar::new_spinner().with_message("Sending transaction...");
        progress.enable_steady_tick(Duration::from_millis(100));

        let blockhash = rpc_client
            .get_latest_blockhash()
            .await
            .expect("Failed to get blockhash");

        let message = Message::try_compile(
            &transaction_creator,
            &[
                ComputeBudgetInstruction::set_compute_unit_price(
                    priority_fee_lamports.unwrap_or(5000),
                ),
                proposal_accounts_close(
                    ProposalAccountsCloseAccounts {
                        multisig,
                        proposal: proposal_pda.0,
                        transaction: transaction_pda.0,
                        rent_collector: rent_collector_key,
                        system_program: SYS_PROGRAM_ID,
                    },
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
            "✅ Collected rent for transaction. Signature: {}",
            signature.green()
        );
        Ok(())
    }
}
