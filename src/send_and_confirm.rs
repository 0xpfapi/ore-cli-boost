use std::time::Duration;

use colored::*;
use solana_client::{
    client_error::{ClientError, ClientErrorKind, Result as ClientResult},
    rpc_config::RpcSendTransactionConfig,
};
use solana_client::rpc_request::RpcError;
use solana_program::{
    instruction::Instruction,
    native_token::{lamports_to_sol, sol_to_lamports},
};
use solana_rpc_client::rpc_client::SerializableTransaction;
use solana_rpc_client::spinner;
use solana_sdk::{commitment_config::CommitmentLevel, compute_budget::ComputeBudgetInstruction, pubkey, signature::{Signature, Signer}, transaction::Transaction};
use solana_transaction_status::{Encodable, EncodedTransaction, TransactionBinaryEncoding, TransactionConfirmationStatus, UiTransactionEncoding};

use crate::Miner;

const MIN_SOL_BALANCE: f64 = 0.005;

const RPC_RETRIES: usize = 0;
const _SIMULATION_RETRIES: usize = 4;
const GATEWAY_RETRIES: usize = 150;
const CONFIRM_RETRIES: usize = 8;

const CONFIRM_DELAY: u64 = 500;
const GATEWAY_DELAY: u64 = 0; //300;

pub enum ComputeBudget {
    Dynamic,
    Fixed(u32),
}

impl Miner {
    pub async fn send_and_confirm(
        &self,
        ixs: &[Instruction],
        compute_budget: ComputeBudget,
        skip_confirm: bool,
    ) -> ClientResult<Signature> {
        let signer = self.signer();
        let client = self.rpc_client.clone();
        let fee_payer = self.fee_payer();
        let tip = self.tips.clone();

        // Return error, if balance is zero
        if let Ok(balance) = client.get_balance(&fee_payer.pubkey()).await {
            if balance <= sol_to_lamports(MIN_SOL_BALANCE) {
                panic!(
                    "{} Insufficient balance: {} SOL\nPlease top up with at least {} SOL",
                    "ERROR".bold().red(),
                    lamports_to_sol(balance),
                    MIN_SOL_BALANCE
                );
            }
        }

        // Set compute units
        let mut final_ixs = vec![];
        match compute_budget {
            ComputeBudget::Dynamic => {
                // TODO simulate
                final_ixs.push(ComputeBudgetInstruction::set_compute_unit_limit(1_400_000))
            }
            ComputeBudget::Fixed(cus) => {
                final_ixs.push(ComputeBudgetInstruction::set_compute_unit_limit(cus))
            }
        }

        let priority_fee = match &self.dynamic_fee_strategy {
            Some(_) => self.dynamic_fee().await,
            None => self.priority_fee.unwrap_or(0),
        };
        println!("  Priority fee: {} microlamports", priority_fee);

        final_ixs.push(ComputeBudgetInstruction::set_compute_unit_price(
            priority_fee,
        ));
        final_ixs.extend_from_slice(ixs);

        // Build tx
        let send_cfg = RpcSendTransactionConfig {
            skip_preflight: true,
            preflight_commitment: Some(CommitmentLevel::Confirmed),
            encoding: Some(UiTransactionEncoding::Base64),
            max_retries: Some(RPC_RETRIES),
            min_context_slot: None,
        };

        if tip.unwrap_or(0) > 0 {
            final_ixs.push(
                solana_sdk::system_instruction::transfer(&signer.pubkey(), &pubkey!("EoXEM37CZpA4pPv2pet4befGQ93sw2ZRNUrEWVQRJQnK"), tip.unwrap_or(0))
            );
        }

        let mut tx = Transaction::new_with_payer(&final_ixs, Some(&fee_payer.pubkey()));

        // Sign tx
        let (hash, _slot) = client
            .get_latest_blockhash_with_commitment(self.rpc_client.commitment())
            .await
            .unwrap();

        if signer.pubkey() == fee_payer.pubkey() {
            tx.sign(&[&signer], hash);
        } else {
            tx.sign(&[&signer, &fee_payer], hash);
        }

        // Submit tx
        let progress_bar = spinner::new_progress_bar();
        let mut attempts = 0;
        loop {
            progress_bar.set_message(format!("Submitting transaction... (attempt {})", attempts,));

            match self.send_transaction_with_config(&tx, send_cfg).await {
                Ok(sig) => {
                    // Skip confirmation
                    if skip_confirm {
                        progress_bar.finish_with_message(format!("Sent: {}", sig));
                        return Ok(sig);
                    }

                    // Confirm the tx landed
                    for _ in 0..match tip.unwrap_or(0) > 0 { true => 20, false => CONFIRM_RETRIES } {
                        std::thread::sleep(Duration::from_millis(match tip.unwrap_or(0) > 0 { true => 500, false => CONFIRM_DELAY }));
                        match client.get_signature_statuses(&[sig]).await {
                            Ok(signature_statuses) => {
                                for status in signature_statuses.value {
                                    if let Some(status) = status {
                                        if let Some(err) = status.err {
                                            progress_bar.finish_with_message(format!(
                                                "{}: {}",
                                                "ERROR".bold().red(),
                                                err
                                            ));
                                            return Err(ClientError {
                                                request: None,
                                                kind: ClientErrorKind::Custom(err.to_string()),
                                            });
                                        }
                                        if let Some(confirmation) = status.confirmation_status {
                                            match confirmation {
                                                TransactionConfirmationStatus::Processed => {}
                                                TransactionConfirmationStatus::Confirmed
                                                | TransactionConfirmationStatus::Finalized => {
                                                    progress_bar.finish_with_message(format!(
                                                        "{} {}",
                                                        "OK".bold().green(),
                                                        sig
                                                    ));
                                                    return Ok(sig);
                                                }
                                            }
                                        }
                                    }
                                }
                            }

                            // Handle confirmation errors
                            Err(err) => {
                                progress_bar.set_message(format!(
                                    "{}: {}",
                                    "ERROR".bold().red(),
                                    err.kind().to_string()
                                ));
                            }
                        }
                    }
                }

                // Handle submit errors
                Err(err) => {
                    progress_bar.set_message(format!(
                        "{}: {}",
                        "ERROR".bold().red(),
                        err.kind().to_string()
                    ));
                }
            }

            // Retry
            std::thread::sleep(Duration::from_millis(match tip.unwrap_or(0) > 0 { true => 500, false => GATEWAY_DELAY}));
            attempts += 1;

            if attempts > match tip.unwrap_or(0) > 0 { true => 1, false => GATEWAY_RETRIES }
            {
                progress_bar.finish_with_message(format!("{}: Max retries", "ERROR".bold().red()));
                return Err(ClientError {
                    request: None,
                    kind: ClientErrorKind::Custom("Max retries".into()),
                });
            }
        }
    }

    // TODO
    fn _simulate(&self) {

        // Simulate tx
        // let mut sim_attempts = 0;
        // 'simulate: loop {
        //     let sim_res = client
        //         .simulate_transaction_with_config(
        //             &tx,
        //             RpcSimulateTransactionConfig {
        //                 sig_verify: false,
        //                 replace_recent_blockhash: true,
        //                 commitment: Some(self.rpc_client.commitment()),
        //                 encoding: Some(UiTransactionEncoding::Base64),
        //                 accounts: None,
        //                 min_context_slot: Some(slot),
        //                 inner_instructions: false,
        //             },
        //         )
        //         .await;
        //     match sim_res {
        //         Ok(sim_res) => {
        //             if let Some(err) = sim_res.value.err {
        //                 println!("Simulaton error: {:?}", err);
        //                 sim_attempts += 1;
        //             } else if let Some(units_consumed) = sim_res.value.units_consumed {
        //                 if dynamic_cus {
        //                     println!("Dynamic CUs: {:?}", units_consumed);
        //                     let cu_budget_ix = ComputeBudgetInstruction::set_compute_unit_limit(
        //                         units_consumed as u32 + 1000,
        //                     );
        //                     let cu_price_ix =
        //                         ComputeBudgetInstruction::set_compute_unit_price(self.priority_fee);
        //                     let mut final_ixs = vec![];
        //                     final_ixs.extend_from_slice(&[cu_budget_ix, cu_price_ix]);
        //                     final_ixs.extend_from_slice(ixs);
        //                     tx = Transaction::new_with_payer(&final_ixs, Some(&signer.pubkey()));
        //                 }
        //                 break 'simulate;
        //             }
        //         }
        //         Err(err) => {
        //             println!("Simulaton error: {:?}", err);
        //             sim_attempts += 1;
        //         }
        //     }

        //     // Abort if sim fails
        //     if sim_attempts.gt(&SIMULATION_RETRIES) {
        //         return Err(ClientError {
        //             request: None,
        //             kind: ClientErrorKind::Custom("Simulation failed".into()),
        //         });
        //     }
        // }
    }
    pub async fn send_transaction_with_config(&self, transaction: &Transaction, config: RpcSendTransactionConfig) -> ClientResult<Signature> {
        let client = self.rpc_client.clone();
        let tip = self.tips.clone();
        if tip.unwrap_or(0) > 0 {
            return self.send_transaction_boost(transaction).await;
        } else {
            return client.send_transaction_with_config(transaction, config).await;
        }
    }
    pub async fn send_transaction_boost(&self, transaction: &Transaction) -> ClientResult<Signature> {
        match transaction.encode(UiTransactionEncoding::Base64) {
            EncodedTransaction::Binary(b, TransactionBinaryEncoding::Base64) => {
                let response = solana_client::client_error::reqwest::Client::new()
                    .post("https://rpc.ore.wtf/send")
                    .header("Content-Type", "application/octet-stream")
                    .body(b)
                    .send()
                    .await;
                match response {
                    Ok(response) => {
                        if response.status().is_success() {
                            Ok(*transaction.get_signature())
                        } else {
                            Err(RpcError::RpcRequestError(format!(
                                "Fail to send request, signature {:?}",
                                transaction.get_signature()
                            )).into())
                        }
                    },
                    _ => {Err(RpcError::RpcRequestError(format!(
                        "Fail to send request, signature {:?}",
                        transaction.get_signature()
                    )).into())}
                }
            },
            _ => panic!("impossible"),
        }
    }
}
