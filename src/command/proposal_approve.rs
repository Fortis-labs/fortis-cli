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

// ─────────────────────────────
// Fortis SDK
// ─────────────────────────────
use fortis_sdk::{
    client::proposal_approve,
    pda::{get_proposal_pda, FORTIS_PROGRAM_ID},
    state::{ProposalApproveAccounts, ProposalApproveArgs},
};

// ─────────────────────────────
// Local utilities
// ─────────────────────────────
use crate::utils::{create_signer_from_path, send_and_confirm_transaction};

#[derive(Args)]
pub struct ProposalApprove {
    /// RPC URL
    #[arg(long)]
    rpc_url: Option<String>,

    /// Path to the Member Keypair
    #[arg(long)]
    keypair: String,

    /// Index of the transaction to vote on
    #[arg(long)]
    transaction_index: u64,

    /// The multisig where the transaction has been proposed
    #[arg(long)]
    multisig_pubkey: String,

    #[arg(long)]
    priority_fee_lamports: Option<u64>,

    #[arg(long)]
    fee_payer_keypair: Option<String>,
}

impl ProposalApprove {
    pub async fn execute(self) -> eyre::Result<()> {
        let Self {
            rpc_url,
            keypair,
            multisig_pubkey,
            transaction_index,
            priority_fee_lamports,
            fee_payer_keypair,
        } = self;

        let transaction_creator_keypair = create_signer_from_path(keypair).unwrap();

        let transaction_creator = transaction_creator_keypair.pubkey();

        let multisig = Pubkey::from_str(&multisig_pubkey).expect("Invalid multisig address");

        let proposal_pda = get_proposal_pda(&multisig, transaction_index, None);

        let rpc_url = rpc_url.unwrap_or_else(|| "https://api.mainnet-beta.solana.com".to_string());

        let transaction_fee_payer_keypair =
            fee_payer_keypair.map(|path| create_signer_from_path(path).unwrap());

        println!();
        println!(
            "{}",
            "👀 You're about to approve a proposal, please review the details:".yellow()
        );
        println!();
        println!("RPC Cluster URL:   {}", rpc_url);
        println!("Program ID:        {}", FORTIS_PROGRAM_ID.to_string());
        println!("Your Public Key:       {}", transaction_creator);
        println!();
        println!("⚙️ Config Parameters");
        println!("Multisig Key:       {}", multisig_pubkey);
        println!("Transaction Index:       {}", transaction_index);
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

        let fee_payer = transaction_fee_payer_keypair
            .as_ref()
            .map(|kp| kp.pubkey())
            .unwrap_or(transaction_creator);
        let message = Message::try_compile(
            &fee_payer,
            &[
                ComputeBudgetInstruction::set_compute_unit_price(
                    priority_fee_lamports.unwrap_or(5000),
                ),
                proposal_approve(
                    ProposalApproveAccounts {
                        multisig,
                        proposal: proposal_pda.0,
                        member: transaction_creator,
                    },
                    ProposalApproveArgs {},
                    None,
                ),
            ],
            &[],
            blockhash,
        )
        .unwrap();

        let mut signers = vec![&*transaction_creator_keypair];
        if let Some(ref fee_payer_kp) = transaction_fee_payer_keypair {
            if fee_payer_kp.pubkey() != transaction_creator {
                signers.push(&**fee_payer_kp);
            }
        }

        let transaction = VersionedTransaction::try_new(VersionedMessage::V0(message), &signers)
            .expect("Failed to create transaction");

        let signature = send_and_confirm_transaction(&transaction, &rpc_client).await?;

        println!("✅ Casted Approved vote. Signature: {}", signature.green());
        Ok(())
    }
}
