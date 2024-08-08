# ORE CLI - BOOST MODE

A command line interface for mining ORE cryptocurrency with the ability to use boost tx landing [98% success] with extended relayer <br /><br />


## New argument TIPS ##


**Tips** - The number of lamports you want to pay as a transaction boost fee (usually 40k lamports, less than **$0.01**)
<br /><br />

Depending on the network load, the number of tips can be increased so that the transaction is processed in the first priority.
<br /><br />

> If you specify tips, transactions are sent to the relay **https://rpc.ore.wtf**.
> 
> If tips are not used, the standard ore-cli logic works (sending to the rpc with multiple attempts).
<br />

Usage example:
```sh
ore --rpc http://<rpc_host> --keypair 'keypair.json' --priority-fee 10000 --tips 50000 mine --cores 1
```

<br />Commissions are sent to the account [EoXEM37CZpA4pPv2pet4befGQ93sw2ZRNUrEWVQRJQnK](https://solscan.io/account/EoXEM37CZpA4pPv2pet4befGQ93sw2ZRNUrEWVQRJQnK) associated with the tx relay. <br /><br />

## Latency (RPC&relay)

Make sure your client instance is close to the RPC and relay **https://rpc.ore.wtf** (NY region located).
Recommended latency value is 10-50ms

## Benchmark of tx landing
Benchmark of successful transactions over the past 24 hours.
Green indicators are successful transactions.

![image](https://github.com/user-attachments/assets/023201cf-7f22-4424-af40-da33668f2830)

Ore-cli logger:

![image](https://github.com/user-attachments/assets/34f72a6e-587c-4c96-8030-51e4f6599b53)


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
