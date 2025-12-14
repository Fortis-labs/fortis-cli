use fortis_sdk::pda::get_vault_pda;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

use clap::Args;
#[derive(Args)]
pub struct DisplayVault {
    /// Multisig
    #[arg(long)]
    multisig_address: String,
}

impl DisplayVault {
    pub async fn execute(self) -> eyre::Result<()> {
        let Self { multisig_address } = self;
        let multisig_address =
            Pubkey::from_str(&multisig_address).expect("Invalid multisig address");

        let vault_address = get_vault_pda(&multisig_address, None);

        println!("Vault: {:?}", vault_address);

        Ok(())
    }
}
