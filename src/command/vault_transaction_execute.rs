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
use solana_compute_budget_interface::ComputeBudgetInstruction;
use solana_message::AddressLookupTableAccount;
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
    client::proposal_execute,
    pda::{get_proposal_pda, get_transaction_pda, FORTIS_PROGRAM_ID},
    state::{ProposalExecuteAccounts, VaultTransaction, VaultTransactionMessage},
};
use solana_client::nonblocking::rpc_client::RpcClient;

use crate::utils::{create_signer_from_path, send_and_confirm_transaction};

#[derive(Args)]
pub struct ProposalExecute {
    /// RPC URL
    #[arg(long)]
    rpc_url: Option<String>,

    /// Path to the Executor Keypair
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
    compute_unit_limit: Option<u32>,

    #[arg(long)]
    extra_keypair: Option<String>,

    #[arg(long)]
    fee_payer_keypair: Option<String>,
}

impl ProposalExecute {
    pub async fn execute(self) -> eyre::Result<()> {
        let Self {
            rpc_url,
            keypair,
            multisig_pubkey,
            transaction_index,
            priority_fee_lamports,
            compute_unit_limit,
            extra_keypair,
            fee_payer_keypair,
        } = self;

        let transaction_creator_keypair = create_signer_from_path(keypair).unwrap();

        let transaction_creator = transaction_creator_keypair.pubkey();

        let multisig = Pubkey::from_str(&multisig_pubkey).expect("Invalid multisig address");

        let proposal_pda = get_proposal_pda(&multisig, transaction_index, None);

        let transaction_pda = get_transaction_pda(&multisig, transaction_index, None);

        let rpc_url = rpc_url.unwrap_or_else(|| "https://api.mainnet-beta.solana.com".to_string());

        let transaction_extra_signer_keypair =
            extra_keypair.map(|path| create_signer_from_path(path).unwrap());

        let transaction_fee_payer_keypair =
            fee_payer_keypair.map(|path| create_signer_from_path(path).unwrap());

        println!();
        println!(
            "{}",
            "👀 You're about to execute a vault transaction, please review the details:".yellow()
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

        let transaction_account_data = rpc_client
            .get_account(&transaction_pda.0)
            .await
            .expect("Failed to get transaction account")
            .data;

        let deserialized_account_data: VaultTransaction =
            borsh::from_slice(transaction_account_data.as_slice()).unwrap();

        let transaction_message = deserialized_account_data.message;
        let address_lkup_tables: Vec<AddressLookupTableAccount> =
            get_necessary_alt(&rpc_client, transaction_message).await;

        let proposal_execute_ix = proposal_execute(
            transaction_account_data.as_slice(),
            ProposalExecuteAccounts {
                member: transaction_creator,
                multisig,
                proposal: proposal_pda.0,
                transaction: transaction_pda.0,
            },
            &address_lkup_tables,
            None,
        )
        .await
        .unwrap();

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
                ComputeBudgetInstruction::set_compute_unit_limit(
                    compute_unit_limit.unwrap_or(200_000),
                ),
                ComputeBudgetInstruction::set_compute_unit_price(
                    priority_fee_lamports.unwrap_or(5000),
                ),
                proposal_execute_ix,
            ],
            &address_lkup_tables,
            blockhash,
        )
        .unwrap();

        let mut signers = vec![&*transaction_creator_keypair];
        if let Some(ref fee_payer_kp) = transaction_fee_payer_keypair {
            if fee_payer_kp.pubkey() != transaction_creator {
                signers.push(&**fee_payer_kp);
            }
        }
        if let Some(ref extra_signer) = transaction_extra_signer_keypair {
            signers.push(&**extra_signer);
        }

        let transaction = VersionedTransaction::try_new(VersionedMessage::V0(message), &signers)
            .expect("Failed to create transaction");

        let signature = send_and_confirm_transaction(&transaction, &rpc_client).await?;

        println!(
            "✅ Executed Vault Transaction. Signature: {}",
            signature.green()
        );
        Ok(())
    }
}

pub async fn get_necessary_alt(
    rpc_client: &RpcClient,
    message: VaultTransactionMessage,
) -> Vec<AddressLookupTableAccount> {
    let mut address_lookup_table_accounts: Vec<AddressLookupTableAccount> = Vec::new();
    let address_lookup_table_keys = message
        .address_table_lookups
        .iter()
        .map(|lookup| lookup.account_key)
        .collect::<Vec<_>>();
    for key in address_lookup_table_keys {
        let account_data = rpc_client.get_account(&key).await.unwrap().data;
        let lookup_table =
            solana_address_lookup_table_interface::state::AddressLookupTable::deserialize(
                &account_data,
            )
            .unwrap();

        let address_lookup_table_account = AddressLookupTableAccount {
            addresses: lookup_table
                .addresses
                .iter()
                .map(|pk| solana_message::Address::new_from_array(pk.to_bytes()))
                .collect(),

            key,
        };

        address_lookup_table_accounts.push(address_lookup_table_account);
    }

    address_lookup_table_accounts
}
