//! An end-to-end example of using the RISC Zero ZKVM to generate and verify a proof of execution
//! of the FuelVM.
//!
//! This program starts a node with a transaction, serializes the input, and passes it to the ZKVM.
//! It then verifies the generated proof to ensure correctness.
//!
//! You can run this script using the following command:
//! ```shell
//! RISC0_DEV_MODE=1 RUST_LOG=info cargo run --release
//! ```
//!
//! The `RISC0_DEV_MODE=1` flag enables development mode, and `RUST_LOG=info` configures logging
//! for better visibility.
use alloy_sol_types::SolType;
use input_provider::start_node_with_transaction_and_produce_prover_input;
use methods::{PROVE_FUEL_ELF, PROVE_FUEL_ID};
use prover::PublicValuesStruct;
use risc0_zkvm::{default_prover, ExecutorEnv};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::filter::EnvFilter::from_default_env())
        .init();

    let service = start_node_with_transaction_and_produce_prover_input()
        .await
        .unwrap();

    let block_id: [u8; 32] = service.input.block.header().id().into();

    let input: Vec<u8> =
        bincode::serialize(&service.input).expect("Failed to serialize service input");

    let env = ExecutorEnv::builder()
        .write(&input)
        .unwrap()
        .build()
        .unwrap();

    let prover = default_prover();
    let prove_info = prover.prove(env, PROVE_FUEL_ELF).unwrap();
    let output: Vec<u8> = prove_info.receipt.journal.decode().unwrap();

    let decoded_output = PublicValuesStruct::abi_decode(&output, true).unwrap();

    assert_eq!(decoded_output.block_id.to_be_bytes(), block_id);

    println!("Proof block id: {:?}", decoded_output.block_id);
    println!("Proof input hash: {:?}", decoded_output.input_hash);

    prove_info
        .receipt
        .verify(PROVE_FUEL_ID)
        .expect("Proof verification failed.");

    println!("Successfully verified proof!");
}
