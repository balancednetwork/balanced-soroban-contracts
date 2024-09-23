[![codecov](https://codecov.io/gh/balancednetwork/balanced-soroban-contracts/graph/badge.svg?token=6Epcv9Uek5)](https://codecov.io/gh/balancednetwork/balanced-soroban-contracts) ![build](https://github.com/balancednetwork/balanced-soroban-contracts/actions/workflows/build-test-soroban-contracts.yml/badge.svg)

# Balanced Package Structure

## Overview

The Balanced contracts within the Stellar blockchain ecosystem is designed to manage various aspects of the decentralized application (dApp) including asset management, cross-chain communication, and stable coin operations. This structure ensures efficient handling of these operations through well-defined smart contracts and unique contract addresses. The smart contracts are written using Soroban framework of Rust programming language

## Contracts

The Balanced Stellar Spoke have three main smart contracts, each responsible for specific functionalities:

### 1. Asset Manager: asset_manager
- **Purpose**: Manages assets within the Balanced ecosystem.
  
### 2. xCall Manager: xcall_manager
- **Purpose**: Facilitates cross-chain administration from the icon side.

### 3. Balanced Dollar: balanced_dollar
- **Purpose**: Includes the bnUSD token contract and Manages the crosschain bnUSD operations within the Balanced ecosystem.

## Identifiers

### Contract Addresses
- **Definition**:  Addresses of each each smart contract on the stellar blockchain
- **Usage**: Used while called using RPC and cross contract calls, also used in cross-chain configuration

```shell
soroban contract  invoke  --id $contract_address \
--source-account $ACCOUNT --rpc-url $ENDPOINT \
--network-passphrase  "$PASSPHRASE"  -- \
#method_name  --arg1 $arg1 --arg2 $arg2
```

## Frontend Integration Interfaces

This guide provides an overview of the key functions for interacting with the Stellar blockchain within your frontend application. These functions are part of the Asset Manager, Balanced Dollar, and XCall smart contracts, which allow for token deposits, cross-chain transfers, and cross-chain calls.

### Important Note: Renting mechanism in Stellar Blockchain and TTL of storage

Stellar has unique renting mechanism implemented on its smart contracts..

---

### Asset Manager Smart contract

The Asset Manager Contract handles depositing Stellar tokens in balanced.

#### `deposit`

Deposits a specified amount of a token into the Sui blockchain.
``` typescript
	deposit(
		from: Address, //Address from which the transaction is initiated
		token: Address, // Address of the token being deposited
		amount: u128, // Amount of the token being deposited
		to: Option<String>,// (Optional) The recipient's address if needed.
		data: Option<Bytes>, // (Optional) Any additional data you want to attach to the deposit.
	);
```
The above method can be RPC called using the shell script with the following code: 
```shell 
	soroban contract  invoke  --id $assetManagerContractAddress \
	--source-account $ACCOUNT --rpc-url $ENDPOINT \
	--network-passphrase  "$PASSPHRASE"  -- \
	deposit  --from $fromAddress --token $tokenAddress \
	--amount  $amount  --to  $toNetworkAddress
```
Both native token and Other fungible tokens can be deposited using the `deposit` method

### Balanced Dollar Module

The Balanced Dollar Contract facilitates the transfer of `BALANCED_DOLLAR` tokens across chains.

#### `cross_transfer`

Transfers `BALANCED_DOLLAR` tokens across chains.
```typescript
	function cross_transfer(
		from: Address, //Address from which the transaction is initiated
		amount: u128, // Amount of the balanced dollar being deposited
		to: String,  // The recipient's address on the destination chain.
		data: Option<Bytes> // (Optional) Any additional data to attach to the transfer.
	)
```
The above method can be called using following shell script: 
```shell
	soroban contract  invoke  --id $bnUSDContractAddress \
	--rpc-url $ENDPOINT --network-passphrase  "$PASSPHRASE" \
	--source-account $ACCOUNT --  cross_transfer  --from $fromAddress --  amount $amount  --to  $toNetworkAddress
```

### XCallManager Contract

#### get_protocols
The `get_protocols` function retrieves the sources and destinations associated with a given configuration. 

```typescript
function get_protocols(
) -> Result<( // Returns a tuple containing two arrays: sources and destinations or error if occurred 
	Vec<String>, 
	Vec<String>
), ContractError>       
```
