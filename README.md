<h1 align="center">
  Fortis cli
</h1>
<p align="center">
<img width="500" height="394" alt="logo1" src="https://github.com/user-attachments/assets/6e48acf5-e9c7-4435-9ef5-88b1710f848c" />
</p>
<p align="center">
cli for fortis multisig.
</p>
Fortis multisig cli is an extesive toolkit to support multisig proposal workflows on SVM

# Fortis CLI

The following is an overview of commands available to interact Fortis Multisig Cli.

Overview

1. [Installation](#1-installation)
2. [Supported wallets](#2-supported-wallets)
3. [Commands](#3-commands)
   - [Create multisig](#multisig-create)
   - [Create proposal](#proposal-create)
   - [Proposal approve](#proposal-approve)
   - [Proposal execute](#proposal-execute)
   - [Proposal accounts close](#proposal-accounts-close)
   - [Display Vault](#display-vault)
   - [Display Multisig](#display-multisig)
   - [Initiate Native transfer](#initiate-native-transfer)
   - [Initiate Program Upgrade](#initiate-program-upgrade)

# 1. Installation

You can install the CLI with Cargo.
For this an installation of Rust will be needed. You can find installation steps [here](https://www.rust-lang.org/tools/install).

To install

```bash
cargo install fortis-multisig-cli
```

# 2. Supported wallets

We Provide the  same wallet support as the Solana CLI, meaning it supports file system wallets as well as Ledger hardware wallets.

### File system wallets

You can easily use your local filesystem wallet by using it as the "keypair" argument in commands.

```bash
fortis-multisig-cli example-command --keypair /path/to/keypair.json
```

This specifies the path of the Keypair that you want to use to sign a CLI transaction.

### Ledger support

To use a Ledger with the Fortis CLI, just specify the Ledger device URL in the "keypair" argument.

```bash
fortis-multisig-cli example-command --keypair usb://ledger
```

This will use the default derivation path of your Ledger.

```bash
fortis-multisig-cli example-command --keypair usb://ledger/5BvrQfDzwjFFjpaAys2KA1a7GuuhLXKJoCWykhsoyHet?key=0/0
```

This specifies a custom derivation path. You can read more about it [here](https://docs.solana.com/wallet-guide/hardware-wallets/ledger).

# 3. Commands

## Multisig Create

### Description

Creates a new multisig with members and threshold configuration.

### Syntax

```bash
multisig-create --rpc-url <RPC_URL> --keypair <KEYPAIR_PATH> --rent-collector <RENT_COLLECTOR> --members " <MEMBER_1_PUBKEY> <MEMBER_2_PUBKEY> ..." --threshold <THRESHOLD>
```

### Parameters

- `--rpc-url <RPC_URL>`: (Optional) The URL of the Solana RPC endpoint. Defaults to mainnet if not specified.
- `--keypair <KEYPAIR_PATH>`: Path to your keypair file.
- `--members <MEMBER_...>`: List of members' public keys, separated by spaces.
- `--threshold <THRESHOLD>`: The threshold number of signatures required for executing multisig transactions.
- `--rent-collector <RENT_COLLECTOR>` : (Optional)The Public key that will be able to reclaim rent from canceled and executed transactions.if not provided will default to Fortis treasury

### Example Usage


   ```bash
   multisig-create --keypair /path/to/keypair.json --members" Member1PubKey Member2PubKey" --threshold 2 --rent-collector <RENT_COLLECTOR>
   ```

   Creates a new multisig account with two members and a threshold of 2.

## Proposal Create

### Description

Create a new  proposal. This command allows any member of a multisig to propose a transaction.

### Syntax

```bash
proposal-create --rpc_url <RPC_URL>  --keypair <KEYPAIR_PATH> --multisig-pubkey <MULTISIG_PUBLIC_KEY> --voting-deadline <VOTING_DEADLINE> --transaction-meesage <TRANSACTION_PAYLOAD> 
```

### Parameters

- `--rpc-url <RPC_URL>`: (Optional) The URL of the Solana RPC endpoint. Defaults to mainnet if not specified.
- `--keypair <KEYPAIR_PATH>`: Path to your keypair file.
- `--multisig-pubkey <MULTISIG_PUBLIC_KEY>`: The public key of the multisig account.
- `--voting-deadline <VOTING_DEADLINE>`: voting deadline for proposal ,should be i64 ,same as unix time format.
- `--transaction-message <TRANSACTION_PAYLOAD>`: transaction message proposal encoded as a base58 string.Serialize your VaultTransactionMessage into Vec<u8>, then encode the bytes as a base58 string.

### Example Usage

   ```bash
   proposal-create --keypair /path/to/keypair.json --multisig-pubkey <MULTISIG_PUBLIC_KEY> --transaction-message abc... --voting-deadline 175978512685
   ```


## Proposal Approve

### Description

Approve a proposed transaction proposal. This command allows a member of a multisig to approve, a transaction proposal.

### Syntax

```bash
proposal-approve --rpc_url <RPC_URL>  --keypair <KEYPAIR_PATH> --multisig-pubkey <MULTISIG_PUBLIC_KEY> --transaction-index <TRANSACTION_INDEX> 
```

### Parameters

- `--rpc-url <RPC_URL>`: (Optional) The URL of the Solana RPC endpoint. Defaults to mainnet if not specified.
- `--keypair <KEYPAIR_PATH>`: Path to your keypair file.
- `--multisig-pubkey <MULTISIG_PUBLIC_KEY>`: The public key of the multisig account.
- `--transaction-index <TRANSACTION_INDEX>`: The index of the proposal (technically transaction) to vote on.

### Example Usage

   ```bash
   proposal-approve --keypair /path/to/keypair.json --multisig-pubkey <MULTISIG_PUBLIC_KEY> --transaction_index 1 
   ```

## Proposal Execute

### Description

Executes a  proposal ,once it has reachen threshold.

### Syntax

```bash
proposal-execute --rpc-url <RPC_URL> --keypair <KEYPAIR_PATH> --multisig-pubkey <MULTISIG_PUBLIC_KEY> --transaction-index <TRANSACTION_INDEX>
```

### Parameters

- `--rpc-url <RPC_URL>`: (Optional) The URL of the Solana RPC endpoint. Defaults to mainnet if not specified.
- `--keypair <KEYPAIR_PATH>`: Path to your keypair file.
- `--multisig-pubkey <MULTISIG_PUBLIC_KEY>`: The public key of the multisig account.
- `--transaction-index <TRANSACTION_INDEX>`: The index of the proposal (technically transaction) to execute.

### Example Usage

```bash
proposal-execute --keypair /path/to/keypair.json --multisig-pubkey <MULTISIG_PUBLIC_KEY> --transaction-index 1
```

This example executes the proposal at index 1 in the specified multisig.


## Proposal Accounts Close

### Description

Closes the proposal and transaction accounts associated with a specific Proposal. The rent will be returned to the multisigs "rent_collector".

### Syntax

```bash
proposal-accounts-close --rpc_url <RPC_URL> --keypair <KEYPAIR_PATH> --multisig-pubkey <MULTISIG_PUBLIC_KEY> --transaction-index <TRANSACTION_INDEX> --rent-collector <RENT_COLLECTOR_PUBKEY>
```

### Parameters

- `--rpc-url <RPC_URL>`: (Optional) The URL of the Solana RPC endpoint. Defaults to mainnet if not specified.
- `--keypair <KEYPAIR_PATH>`: Path to your keypair file.
- `--multisig-pubkey <MULTISIG_PUBLIC_KEY>`: The public key of the multisig account.
- `--transaction-index <TRANSACTION_INDEX>`: The index of the proposal (technically transaction) whose accounts are to be closed.
- `--rent-collector <RENT_COLLECTOR_PUBKEY>`: The public key of the account responsible for collecting rent.

### Example Usage

```bash
proposal-accounts-close --keypair /path/to/keypair.json --multisig-pubkey <MULTISIG_PUBLIC_KEY> --transaction-index 1 --rent-collector <RENT_COLLECTOR_PUBKEY>
```

In this example, the command closes the proposal accounts for the transaction at index 1 in the specified multisig account and collects rent using the provided rent collector public key.
## Display Vault

### Description

View vault associated to your multisig

### Syntax

```bash
display-vault --multisig-address <MULTISIG_PUBLIC_KEY>
```

### Parameters

- `--multisig-address <MULTISIG_PUBLIC_KEY>`: The public key of the multisig account.

## Display Multisig

### Description

View all info on your multisig

### Syntax

```bash
display-multisig --rpc_url <RPC_URL> --multisig-address <MULTISIG_PUBLIC_KEY>
```

### Parameters
- `--rpc-url <RPC_URL>`: (Optional) The URL of the Solana RPC endpoint. Defaults to mainnet if not specified.
- `--multisig-address <MULTISIG_PUBLIC_KEY>`: The public key of the multisig account.

### Example Usage

```bash
display-multisig --multisig-address <MULTISIG_PUBLIC_KEY>
```
```console
# Example Output
Multisig address: 2C5tyVLDnJ4QhL5xkFb8dvrAKZDakFNmUHM8T9jWoSNf
Multisig creator: 4YbfMWzXz29tAs9W8y5h98c3PW3ruGZbDg1E1zB73DVJ
Rent collector: ap5oPFPVSnxtc8bbvcCeKwy9Xnu5NePhMGzX2hexDVh
Proposals count: 1
Multisig threshold: 1
Multisig members:
[
    ap5oPFPVSnxtc8bbvcCeKwy9Xnu5NePhMGzX2hexDVh,
    AgZ9okAAA7sHz6ddJnuq6RFHXuEQZt3CgBZsNGHByjq5,
]
Multisig vault: 7npNuK6LjWehkTbg768yd5vXwstPF9V4i6Gu6rULWSWV

▶ Proposal #1
  Address     : 69sJ9DLxTL76cGnGSrVpMHRRsrdW3ZFs28YmbekGi4g6
  Created At  : 1765812709
  Deadline    : 1766664125
  Approvals   : 0/1
  Approvers   : None
  Status      : Active

```
## Initiate Native transfer

### Description

Create a new  sol transfer proposal. This command allows any member of a multisig to propose a transaction.

### Syntax

```bash
initiate-native-transfer --rpc_url <RPC_URL>  --keypair <KEYPAIR_PATH> --multisig-pubkey <MULTISIG_PUBLIC_KEY> --voting-deadline <VOTING_DEADLINE> --recipient <RECIPIENT_PUBLIC_KEY> --token-amount-u64 <LAMPORTS_TO_TRANSFER>
```

### Parameters

- `--rpc-url <RPC_URL>`: (Optional) The URL of the Solana RPC endpoint. Defaults to mainnet if not specified.
- `--keypair <KEYPAIR_PATH>`: Path to your keypair file.
- `--multisig-pubkey <MULTISIG_PUBLIC_KEY>`: The public key of the multisig account.
- `--voting-deadline <VOTING_DEADLINE>`: voting deadline for proposal ,should be i64 ,same as unix time format.
- `--recipient <RECIPIENT_PUBLIC_KEY>`: The public key of the recipient account.
- `--token-amount-u64 <LAMPORTS_TO_TRANSFER>`: lamports to transfer.

## Initiate transfer

### Description

Create a new  spl-token transfer proposal. This command allows any member of a multisig to propose a transaction.

### Syntax

```bash
initiate-transfer --rpc-url <RPC_URL>  --token-mint-address <TOKEN_MINT> --token-amount-u64 <AMOUNT_IN_SMALLEST_UNITS> --recipient <RECIPIENT_PUBKEY> --keypair <KEYPAIR_PATH> --multisig-pubkey <MULTISIG_PUBKEY> --voting-deadline <VOTING_DEADLINE> 
```

### Parameters

- `--rpc-url <RPC_URL>`: (Optional) The URL of the Solana RPC endpoint. Defaults to mainnet if not specified.
- `--keypair <KEYPAIR_PATH>`: Path to your keypair file.
- `--multisig-pubkey <MULTISIG_PUBLIC_KEY>`: The public key of the multisig account.
- `--token-mint-address <TOKEN_MINT>`: Token Mint
- `--voting-deadline <VOTING_DEADLINE>`: voting deadline for proposal ,should be i64 ,same as unix time format.
- `--recipient <RECIPIENT_PUBLIC_KEY>`: The public key of the recipient account.
- `--token-amount-u64 <LAMPORTS_TO_TRANSFER>`: amount to transfer.

## Initiate Program Upgrade

### Description

Create a proposal to uprade program. This command allows any member of a multisig to propose a transaction.

### Syntax

```bash
initiate-program-upgrade --rpc-url <RPC_URL> --keypair <KEYPAIR_PATH>  --multisig-pubkey <MULTISIG_PUBKEY> --voting-deadline <VOTING_DEADLINE> --spill-address <SPILL_ADDRESS> --program-to-upgrade-id <PROGRAM_ID> --buffer-address <BUFFER_ADDRESS>
```
Fortis allows teams to control program upgrades using a multisig.
- Deploy your program
  
``` solana program deploy PATH_TO_DOT_SO```
- Change the upgrade authority to the Fortis multisig vault
  
```solana program set-upgrade-authority PROGRAM_ID --new-upgrade-authority FORTIS_MULTISIG_VAULT --skip-new-upgrade-authority-signer-check```
- Write the updated program to a buffer

```solana program write-buffer PATH_TO_DOT_SO```
- Initiate the program upgrade via Fortis

Once the program’s upgrade authority is set to the Fortis multisig, you can initiate and approve the upgrade through the multisig workflow

### Parameters

- `--rpc-url <RPC_URL>`: (Optional) The URL of the Solana RPC endpoint. Defaults to mainnet if not specified.
- `--keypair <KEYPAIR_PATH>`: Path to your keypair file.
- `--multisig-pubkey <MULTISIG_PUBLIC_KEY>`: The public key of the multisig account.
- `--voting-deadline <VOTING_DEADLINE>`: voting deadline for proposal ,should be i64 ,same as unix time format.
- `--program-to-upgrade-id <PROGRAM_ID`: id of the program to be updated.
- `--spill-address <SPILL_ADDRESS>` :adress to send execessive sol from upgrade
- `buffer-address <BUFFER ADDRESS>`:account that holds new program code

