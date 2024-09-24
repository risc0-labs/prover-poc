use alloy_sol_types::sol;
use fuel_core_executor::executor::{ExecutionInstance, ExecutionOptions, OnceTransactionsSource};
use fuel_core_executor::ports::RelayerPort;
use fuel_core_storage::column::Column;
use fuel_core_storage::StorageAsMut;
use fuel_core_storage::tables::ConsensusParametersVersions;
use fuel_core_storage::transactional::{IntoTransaction, ReadTransaction};
use fuel_core_types::blockchain::header::{ConsensusHeader, PartialBlockHeader};
use fuel_core_types::blockchain::primitives::DaBlockHeight;
use fuel_core_types::fuel_asm::{op, RegId};
use fuel_core_types::fuel_tx::{Chargeable, ConsensusParameters, Finalizable, Input, Transaction, TransactionBuilder, TxPointer, UtxoId};
use fuel_core_types::fuel_vm::checked_transaction::EstimatePredicates;
use fuel_core_types::fuel_vm::interpreter::MemoryInstance;
use fuel_core_types::services::block_producer::Components;
use fuel_core_types::services::executor::ExecutionResult;
use fuel_core_types::services::relayer::Event;
use fuel_core_types::tai64::Tai64;
use crate::memory::InMemoryStorage;

extern crate alloc;

const CONSENSUS_PARAMETERS: &[u8] = include_bytes!("consensus_parameters.json");

sol! {
    /// The public values encoded as a struct that can be easily deserialized inside Solidity.
    struct PublicValuesStruct {
        uint32 n;
        uint32 a;
        uint32 b;
    }
}

struct FakeRelayer;

impl RelayerPort for FakeRelayer {
    fn enabled(&self) -> bool {
        false
    }

    fn get_events(&self, _: &DaBlockHeight) -> anyhow::Result<Vec<Event>> {
        unreachable!("This should never be called")
    }
}

mod memory {
    use fuel_core_storage::kv_store::{KeyValueInspect, StorageColumn, Value};
    use fuel_core_storage::Result as StorageResult;
    use alloc::collections::BTreeMap;

    type Storage = BTreeMap<(u32, Vec<u8>), Value>;

    /// The in-memory storage for testing purposes.
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct InMemoryStorage<Column> {
        pub(crate) storage: Storage,
        _marker: core::marker::PhantomData<Column>,
    }

    impl<Column> Default for InMemoryStorage<Column> {
        fn default() -> Self {
            Self {
                storage: Default::default(),
                _marker: Default::default(),
            }
        }
    }

    impl<Column> KeyValueInspect for InMemoryStorage<Column>
        where
            Column: StorageColumn,
    {
        type Column = Column;

        fn get(&self, key: &[u8], column: Self::Column) -> StorageResult<Option<Value>> {
            let value = self.storage.get(&(column.id(), key.to_vec())).cloned();
            Ok(value)
        }
    }
}

fn transaction(consensus_parameters: &ConsensusParameters) -> Transaction {
    let script = [
        op::movi(0x10, 1024),
        op::addi(0x11, 0x10, 1024),
        op::jmpb(RegId::ZERO, 0),
    ].into_iter().collect();

    let predicate: Vec<u8> = vec![op::ret(RegId::ONE)].into_iter().collect();
    let owner = Input::predicate_owner(&predicate);
    let amount = 10000;

    let gas_costs = consensus_parameters.gas_costs();
    let fee_params = consensus_parameters.fee_params();

    let mut tx = TransactionBuilder::script(script, vec![])
        .add_input(Input::coin_predicate(
            UtxoId::default(),
            owner,
            amount,
            *consensus_parameters.base_asset_id(),
            TxPointer::default(),
            0,
            predicate,
            vec![],
        ))
        .script_gas_limit(consensus_parameters.block_gas_limit() - 500_000)
        .finalize();
    tx.estimate_predicates(
        &consensus_parameters.clone().into(),
        MemoryInstance::new(),
    )
        .unwrap();
    println!("Max gas: {}", 2 * tx.max_gas(gas_costs, fee_params));
    tx.into()
}

pub fn fibonacci(_: u32) -> (u32, u32) {
    let mut memory_storage = InMemoryStorage::<Column>::default().into_transaction();

    let consensus_parameters = serde_json::from_slice::<ConsensusParameters>(CONSENSUS_PARAMETERS).unwrap();

    memory_storage.storage_as_mut::<ConsensusParametersVersions>().insert(&0, &consensus_parameters).unwrap();

    let executor = ExecutionInstance::new(FakeRelayer, memory_storage.read_transaction(), ExecutionOptions {
        extra_tx_checks: false,
        backtrace: false,
    });

    let header_to_produce = PartialBlockHeader {
        application: Default::default(),
        consensus: ConsensusHeader {
            height: 1u32.into(),
            time: Tai64::UNIX_EPOCH,
            ..Default::default()
        },
    };

    let components = Components {
        header_to_produce,
        transactions_source: OnceTransactionsSource::new(vec![transaction(&consensus_parameters)]),
        coinbase_recipient: Default::default(),
        gas_price: 0,
    };

    let ExecutionResult { block: _block, .. } = executor.produce_without_commit(components, false).unwrap().into_result();


    // let validator = ExecutionInstance::new(FakeRelayer, memory_storage, ExecutionOptions {
    //     extra_tx_checks: false,
    //     backtrace: false,
    // });
    // let _ = validator.validate_without_commit(&_block).unwrap();

    (1, 1)
}

#[cfg(test)]
mod tests {
    use std::time::Instant;
    use super::*;

    #[test]
    fn dummy() {
        let instance = Instant::now();
        fibonacci(0);
        println!("Execution time: {:?}", instance.elapsed());
    }
}