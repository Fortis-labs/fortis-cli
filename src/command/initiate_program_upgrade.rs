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
use solana_loader_v3_interface::instruction::upgrade;
use solana_sdk::{
    message::{v0::Message, VersionedMessage},
    pubkey::Pubkey,
    signature::Signer,
    sysvar::instructions::Instructions,
    transaction::VersionedTransaction,
};
use solana_system_interface::program::ID as SYS_PROGRAM_ID;

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
pub struct InitiateProgramUpgrade {
    /// RPC URL
    #[arg(long)]
    rpc_url: Option<String>,

    /// account that holds new program code.
    #[arg(long)]
    buffer_address: String,

    /// Path to the Proposal Creator Keypair
    #[arg(long)]
    keypair: String,

    /// The multisig where the transaction has been proposed
    #[arg(long)]
    multisig_pubkey: String,

    #[arg(long)]
    voting_deadline: u64,

    /// The program to upgrade
    #[arg(long)]
    program_to_upgrade_id: String,

    /// The spill address(adress to send execessive sol from upgrade)
    #[arg(long)]
    spill_address: String,

    #[arg(long)]
    priority_fee_lamports: Option<u64>,
}

impl InitiateProgramUpgrade {
    pub async fn execute(self) -> eyre::Result<()> {
        let Self {
            rpc_url,
            keypair,
            multisig_pubkey,
            voting_deadline,
            priority_fee_lamports,
            buffer_address,
            program_to_upgrade_id,
            spill_address,
        } = self;

        let transaction_creator_keypair = create_signer_from_path(keypair).unwrap();

        let transaction_creator = transaction_creator_keypair.pubkey();

        let program_to_upgrade =
            Pubkey::from_str(&program_to_upgrade_id).expect("Invalid to upgrade program ID");
        let spill_address_id = Pubkey::from_str(&spill_address).expect("Invalid spill address");
        let buffer_address_id = Pubkey::from_str(&buffer_address).expect("Invalid buffer address");

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
            "👀 You're about to initiate a proposal and a vault transaction to upgrade your program from a buffer, please review the details:".yellow()
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
        println!("To upgrade program ID:       {}", program_to_upgrade_id);
        println!("Buffer Address:       {}", buffer_address);
        println!("Spill Address:       {}", spill_address);
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

        let buffer_auth_update_ix = solana_loader_v3_interface::instruction::set_buffer_authority(
            &buffer_address_id,
            &transaction_creator,
            &vault_pda.0,
        );
        let instruction = upgrade(
            &program_to_upgrade,
            &buffer_address_id,
            &vault_pda.0,
            &spill_address_id,
        );
        let upgrade_program_message = VaultTransactionMessage::try_compile(
            &vault_pda.0,
            &[buffer_auth_update_ix, instruction],
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
                    &upgrade_program_message,
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
