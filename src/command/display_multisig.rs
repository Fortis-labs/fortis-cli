use colored::*;
use fortis_sdk::{
    client::{get_multisig, get_proposal},
    pda::*,
};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

use clap::Args;

#[derive(Args)]
pub struct DisplayMultisig {
    /// RPC URL
    #[arg(long)]
    rpc_url: Option<String>,
    /// Multisig
    #[arg(long)]
    multisig_address: String,
}
impl DisplayMultisig {
    pub async fn execute(self) -> eyre::Result<()> {
        let Self {
            rpc_url,
            multisig_address,
        } = self;

        let rpc_url = rpc_url.unwrap_or_else(|| "https://api.mainnet-beta.solana.com".to_string());

        let rpc_client = &RpcClient::new(rpc_url);

        let multisig = Pubkey::from_str(&multisig_address).expect("Invalid multisig address");
        let multisig_data = get_multisig(rpc_client, &multisig).await?;
        let multisig_address =
            Pubkey::from_str(&multisig_address).expect("Invalid multisig address");
        let vault_address = get_vault_pda(&multisig_address, None);
        println!(
            "{} {}",
            "Multisig address:".dimmed().cyan(),
            multisig_address.to_string().bold()
        );

        println!(
            "{} {}",
            "Multisig creator:".dimmed().cyan(),
            multisig_data.create_key.to_string().bold()
        );

        println!(
            "{} {}",
            "Rent collector:".dimmed().cyan(),
            multisig_data.rent_collector.to_string().bold()
        );

        println!(
            "{} {}",
            "Proposals count:".dimmed().cyan(),
            multisig_data.transaction_index.to_string().bold()
        );

        println!(
            "{} {}",
            "Multisig threshold:".dimmed().cyan(),
            multisig_data.threshold.to_string().bold()
        );

        println!(
            "{}\n{:#?}",
            "Multisig members:".dimmed().cyan(),
            multisig_data.members
        );

        println!(
            "{} {}",
            "Multisig vault:".dimmed().cyan(),
            vault_address.0.to_string().bold()
        );
        //display all proposals and transactions
        let num_proposals = multisig_data.transaction_index;

        for proposal_index in 1..num_proposals + 1 {
            let proposal_key = get_proposal_pda(&multisig_address, proposal_index, None).0;

            let proposal = match get_proposal(rpc_client, &proposal_key).await {
                Ok(p) => p,
                Err(_) => {
                    println!(
                        "{} Proposal #{} does not exist\n",
                        "⚠".yellow(),
                        proposal_index
                    );
                    continue;
                }
            };

            println!(
                "\n{} Proposal #{}",
                "▶".cyan(),
                proposal_index.to_string().bold()
            );
            println!("  Address     : {}", proposal_key.to_string().dimmed());
            println!("  Created At  : {}", proposal.timestamp);
            println!("  Deadline    : {}", proposal.deadline);
            println!(
                "  Approvals   : {}/{}",
                proposal.approved.len().to_string().green(),
                multisig_data.threshold
            );

            if proposal.approved.is_empty() {
                println!("  Approvers   : {}", "None".dimmed());
            } else {
                for pk in &proposal.approved {
                    println!("    • {}", pk.to_string().bright_white());
                }
            }

            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();

            let status = match proposal.status {
                0 if now > proposal.deadline as u64 => "Expired".red(),
                0 => "Active".yellow(),
                1 => "Approved".green(),
                _ => "Executed".bright_green(),
            };

            println!("  Status      : {}\n", status.bold());
        }
        Ok(())
    }
}
