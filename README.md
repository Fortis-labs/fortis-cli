<h1 align="center">
  Fortis sdk
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
   - [Vote on proposals](#proposal-vote)
   - [Reclaim Vault Transaction rent](#vault-transaction-accounts-close)
   - [Create new Vault Transaction](#vault-transaction-create)
   - [Execute Vault Transaction](#vault-transaction-execute)

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
multisig-create --rpc-url <RPC_URL> --keypair <KEYPAIR_PATH> --rent-collector <RENT_COLLECTOR> --members <MEMBER_1> <MEMBER_2> ... --threshold <THRESHOLD>
```

### Parameters

- `--rpc-url <RPC_URL>`: (Optional) The URL of the Solana RPC endpoint. Defaults to mainnet if not specified.
- `--keypair <KEYPAIR_PATH>`: Path to your keypair file.
- `--members <MEMBER_...>`: List of members' public keys, separated by spaces.
- `--threshold <THRESHOLD>`: The threshold number of signatures required for executing multisig transactions.
- `--rent-collector <RENT_COLLECTOR>` : The Public key that will be able to reclaim rent from canceled and executed transactions.

### Example Usage


   ```bash
   multisig-create --keypair /path/to/keypair.json --members "Member1PubKey" "Member2PubKey" --threshold 2 --rent-collector <RENT_COLLECTOR>
   ```

   Creates a new multisig account with two members and a threshold of 2.

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
- `--transaction-index <TRANSACTION_INDEX>`: The index of the transaction to vote on.

### Example Usage

   ```bash
   proposal-approve --keypair /path/to/keypair.json --multisig-pubkey <MULTISIG_PUBLIC_KEY> --transaction_index 1 --action Approve
   ```

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
- `--transaction-index <TRANSACTION_INDEX>`: The index of the transaction whose accounts are to be closed.
- `--rent-collector <RENT_COLLECTOR_PUBKEY>`: The public key of the account responsible for collecting rent.

### Example Usage

```bash
proposal-accounts-close --keypair /path/to/keypair.json --multisig-pubkey <MULTISIG_PUBLIC_KEY> --transaction-index 1 --rent-collector <RENT_COLLECTOR_PUBKEY>
```

In this example, the command closes the proposal accounts for the transaction at index 1 in the specified multisig account and collects rent using the provided rent collector public key.

## Proposal Create

### Description

Creates a new proposal with a custom transaction message.

### Syntax

```bash
proposal-create --rpc-url <RPC_URL>  --keypair <KEYPAIR_PATH> --multisig-pubkey <MULTISIG_PUBLIC_KEY> --transaction-message <TRANSACTION_MESSAGE>
```

### Parameters

- `--rpc-url <RPC_URL>`: (Optional) The URL of the Solana RPC endpoint. Defaults to mainnet if not specified.
- `--keypair <KEYPAIR_PATH>`: Path to your keypair file.
- `--multisig-pubkey <MULTISIG_PUBLIC_KEY>`: The public key of the multisig account.
- `--transaction-message <TRANSACTION_MESSAGE>`: The message or payload of the transaction.

### Example Usage

```bash
proposal-create --keypair /path/to/keypair.json --multisig-pubkey <MULTISIG_PUBLIC_KEY> --vault-index 1 --transaction-message [1, 2, 3, 5, 5, 6, 7, 8]
```

In this example, a new transaction with the specified message is proposed in the multisig vault.

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
- `--transaction-index <TRANSACTION_INDEX>`: The index of the transaction to be executed.

### Example Usage

```bash
vault-transaction-execute --keypair /path/to/keypair.json --multisig-pubkey <MULTISIG_PUBLIC_KEY> --transaction-index 1
```

This example executes the proposal at index 1 in the specified multisig.
