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
// Solana SDK & Programs
// ─────────────────────────────
use solana_compute_budget_interface::ComputeBudgetInstruction;
use solana_sdk::{
    message::{v0::Message, VersionedMessage},
    pubkey::Pubkey,
    signature::Signer,
    transaction::VersionedTransaction,
};
use solana_system_interface::program::ID as SYS_PROGRAM_ID;

// SPL
use spl_associated_token_account::get_associated_token_address_with_program_id;
use spl_token::instruction::transfer;

// ─────────────────────────────
// Fortis SDK
// ─────────────────────────────
use fortis_sdk::{
    client::{get_multisig, proposal_create},
    pda::{get_proposal_pda, get_transaction_pda, get_vault_pda, FORTIS_PROGRAM_ID},
    state::{ProposalCreateAccounts, VaultTransactionMessage},
};

// ─────────────────────────────
// RPC
// ─────────────────────────────
use solana_client::nonblocking::rpc_client::RpcClient;

// ─────────────────────────────
// Local utils
// ─────────────────────────────
use crate::utils::{create_signer_from_path, send_and_confirm_transaction};

#[derive(Args)]
pub struct InitiateTransfer {
    /// RPC URL
    #[arg(long)]
    rpc_url: Option<String>,

    /// Token program ID. Defaults to regular SPL.
    #[arg(long)]
    token_program_id: Option<String>,

    /// Token Mint Address.
    #[arg(long)]
    token_mint_address: String,

    #[arg(long)]
    token_amount_u64: u64,

    /// The recipient of the Token(s)
    #[arg(long)]
    recipient: String,

    /// Path to the Proposal Creator Keypair
    #[arg(long)]
    keypair: String,

    /// The multisig where the transaction has been proposed
    #[arg(long)]
    multisig_pubkey: String,

    #[arg(long)]
    voting_deadline: u64,

    #[arg(long)]
    priority_fee_lamports: Option<u64>,
}

impl InitiateTransfer {
    pub async fn execute(self) -> eyre::Result<()> {
        let Self {
            rpc_url,
            token_program_id,
            keypair,
            multisig_pubkey,
            voting_deadline,
            priority_fee_lamports,
            token_amount_u64,
            token_mint_address,
            recipient,
        } = self;

        let token_program_id = token_program_id
            .unwrap_or_else(|| "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA".to_string());

        let token_program_id = Pubkey::from_str(&token_program_id).expect("Invalid program ID");

        let transaction_creator_keypair = create_signer_from_path(keypair).unwrap();

        let transaction_creator = transaction_creator_keypair.pubkey();

        let rpc_url = rpc_url.unwrap_or_else(|| "https://api.mainnet-beta.solana.com".to_string());
        let rpc_url_clone = rpc_url.clone();
        let rpc_client = &RpcClient::new(rpc_url);

        let multisig = Pubkey::from_str(&multisig_pubkey).expect("Invalid multisig address");

        let recipient_pubkey = Pubkey::from_str(&recipient).expect("Invalid recipient address");

        let token_mint = Pubkey::from_str(&token_mint_address).expect("Invalid Token Mint Address");

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
        println!("Voting deadline:       {}", voting_deadline);
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

        let vault_pda = get_vault_pda(&multisig, None);

        let sender_ata = get_associated_token_address_with_program_id(
            &vault_pda.0,
            &token_mint,
            &token_program_id,
        );

        let recipient_ata = get_associated_token_address_with_program_id(
            &recipient_pubkey,
            &token_mint,
            &token_program_id,
        );

        let transfer_message = VaultTransactionMessage::try_compile(
            &vault_pda.0,
            &[transfer(
                &token_program_id,
                &sender_ata,
                &recipient_ata,
                &vault_pda.0,
                &[&vault_pda.0],
                token_amount_u64,
            )
            .unwrap()],
            &[],
        )
        .unwrap();

        let message = Message::try_compile(
            &transaction_creator,
            &[
                ComputeBudgetInstruction::set_compute_unit_price(
                    priority_fee_lamports.unwrap_or(200_000),
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
                    &transfer_message,
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
