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
    signature::{Keypair, Signer},
    transaction::VersionedTransaction,
};
use solana_system_interface::program::ID as SYS_PROGRAM_ID;

// ─────────────────────────────
// Fortis SDK
// ─────────────────────────────
use fortis_sdk::{
    client::multisig_create,
    pda::{get_multisig_pda, FORTIS_PROGRAM_ID},
    state::{MultisigCreateAccounts, MultisigCreateArgs},
};

// ─────────────────────────────
// Local utilities
// ─────────────────────────────
use crate::utils::{create_signer_from_path, send_and_confirm_transaction};
//this command can only be used ,if a rent_collector has been set previously
#[derive(Args)]
pub struct MultisigCreate {
    /// RPC URL
    #[arg(long)]
    rpc_url: Option<String>,

    /// Path to the Multisig Creator
    #[arg(long)]
    keypair: String,

    #[arg(long)]
    rent_collector: Option<Pubkey>,

    #[arg(long, short, value_delimiter = ' ')]
    members: Vec<String>,

    #[arg(long)]
    threshold: u16,

    #[arg(long)]
    priority_fee_lamports: Option<u64>,
}

impl MultisigCreate {
    pub async fn execute(self) -> eyre::Result<()> {
        let Self {
            rpc_url,
            keypair,
            members,
            threshold,
            rent_collector,
            priority_fee_lamports,
        } = self;

        let transaction_creator_keypair = create_signer_from_path(keypair).unwrap();

        let transaction_creator = transaction_creator_keypair.pubkey();

        let rpc_url = rpc_url.unwrap_or_else(|| "https://api.mainnet-beta.solana.com".to_string());

        let members = parse_members(members).unwrap_or_else(|err| {
            eprintln!("Error parsing members: {}", err);
            std::process::exit(1);
        });

        println!();
        println!(
            "{}",
            "👀 You're about to create a multisig, please review the details:".yellow()
        );
        println!();
        println!("RPC Cluster URL:   {}", rpc_url);
        println!("Program ID:        {}", FORTIS_PROGRAM_ID.to_string());
        println!("Your Public Key:       {}", transaction_creator);
        println!();
        println!("⚙️ Config Parameters");
        println!();
        println!("Threshold:          {}", threshold);
        println!(
            "Rent Collector:     {}",
            rent_collector
                .map(|k| k.to_string())
                .unwrap_or_else(|| "None".to_string())
        );
        println!("Members amount:      {}", members.len());
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

        let random_keypair = Keypair::new();

        let multisig_key = get_multisig_pda(&random_keypair.pubkey(), None);
        let message = Message::try_compile(
            &transaction_creator,
            &[
                ComputeBudgetInstruction::set_compute_unit_price(
                    priority_fee_lamports.unwrap_or(5000),
                ),
                multisig_create(
                    MultisigCreateAccounts {
                        create_key: random_keypair.pubkey(),
                        creator: transaction_creator,
                        multisig: multisig_key.0,
                        system_program: SYS_PROGRAM_ID,
                        treasury: fortis_sdk::pda::TREASURY,
                    },
                    MultisigCreateArgs {
                        members,
                        threshold,
                        rent_collector,
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
            &[
                &*transaction_creator_keypair,
                &random_keypair as &dyn Signer,
            ],
        )
        .expect("Failed to create transaction");

        let signature = send_and_confirm_transaction(&transaction, &rpc_client).await?;

        println!(
            "✅ Created Multisig: {}. Signature: {}",
            multisig_key.0,
            signature.green()
        );
        Ok(())
    }
}

fn parse_members(member_strings: Vec<String>) -> Result<Vec<Pubkey>, String> {
    member_strings
        .into_iter()
        .map(|s| Pubkey::from_str(&s).map_err(|_| format!("Invalid public key: {}", s)))
        .collect()
}
