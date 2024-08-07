# ORE CLI [BOOST MODE]

A command line interface for mining ORE cryptocurrency with the ability to use boost tx landing [98% success] with extended relayer 


## New argument *TIPS* ##


**Tips** - The number of lamports you want to pay as a transaction boost fee (usually 40k lamports, less than **$0.01**)

Depending on the network load, the number of tips can be increased so that the transaction is processed in the first priority.


> If you specify tips, transactions are sent to the relay **https://rpc.ore.wtf**.
> 
> If tips are not used, the standard ore-cli logic works (sending to the rpc with multiple attempts).


Usage example:
```sh
ore --rpc http://<rpc_host> --keypair 'keypair.json' --priority-fee 10000 --tips 50000 mine --threads 1
```

Commissions are sent to the account [EoXEM37CZpA4pPv2pet4befGQ93sw2ZRNUrEWVQRJQnK](https://solscan.io/account/EoXEM37CZpA4pPv2pet4befGQ93sw2ZRNUrEWVQRJQnK) associated with the tx relay.

# BENCHMARK OF TX LANDING
Benchmark of successful transactions over the past 24 hours.
Green indicators are successful transactions.

![image](https://github.com/user-attachments/assets/023201cf-7f22-4424-af40-da33668f2830)


## Install

To install the CLI, use [cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html):

```sh
cargo install --git https://github.com/0xpfapi/ore-cli-boost
```

## Build

To build the codebase from scratch, checkout the repo and use cargo to build:

```sh
cargo build --release
```

## Help

You can use the `-h` flag on any command to pull up a help menu with documentation:

```sh
ore -h
```
