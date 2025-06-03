use std::sync::{Arc, RwLock};

use reth::revm::{
    context::{result::ExecutionResult, TxEnv},
    State,
};
use reth_chainspec::ChainSpec;
use reth_evm::{
    block::{BlockExecutionError, BlockExecutor, BlockValidationError, ExecutableTx},
    eth::EthBlockExecutor,
    Database, Evm, OnStateHook,
};
use reth_evm_ethereum::RethReceiptBuilder;
use reth_primitives::{Receipt, TransactionSigned};
use reth_provider::BlockExecutionResult;

use super::KadenaMint;

pub struct KadenaBlockExecutor<'a, Evm> {
    /// Inner Ethereum execution strategy.
    pub inner: EthBlockExecutor<'a, Evm, &'a Arc<ChainSpec>, &'a RethReceiptBuilder>,
    pub mints: Arc<RwLock<Vec<KadenaMint>>>,
}

impl<'a, Evm> KadenaBlockExecutor<'a, Evm> {
    /// Creates a new [`KadenaBlockExecutor`].
    pub fn new(
        inner: EthBlockExecutor<'a, Evm, &'a Arc<ChainSpec>, &'a RethReceiptBuilder>,
        mints: Arc<RwLock<Vec<KadenaMint>>>,
    ) -> Self {
        Self { inner, mints }
    }
}

impl<'db, DB, E> BlockExecutor for KadenaBlockExecutor<'_, E>
where
    DB: Database + 'db,
    E: Evm<DB = &'db mut State<DB>, Tx = TxEnv>,
{
    type Transaction = TransactionSigned;
    type Receipt = Receipt;
    type Evm = E;

    fn apply_pre_execution_changes(&mut self) -> Result<(), BlockExecutionError> {
        self.inner.apply_pre_execution_changes()
    }

    fn execute_transaction_with_result_closure(
        &mut self,
        tx: impl ExecutableTx<Self>,
        f: impl FnOnce(&ExecutionResult<<Self::Evm as Evm>::HaltReason>),
    ) -> Result<u64, BlockExecutionError> {
        self.inner.execute_transaction_with_result_closure(tx, f)
    }

    fn finish(mut self) -> Result<(Self::Evm, BlockExecutionResult<Receipt>), BlockExecutionError> {
        {
            let mints = self.mints.read().unwrap().clone();
            self.inner
                .evm_mut()
                .db_mut()
                .increment_balances(mints.into_iter().map(|x| x.into()))
                .map_err(|_| BlockValidationError::IncrementBalanceFailed)?;
        }

        let mut m = self.mints.write().unwrap();
        *m = Vec::new(); // Clear mints after applying them

        // Invoke inner finish method to apply Ethereum post-execution changes
        self.inner.finish()
    }

    fn set_state_hook(&mut self, _hook: Option<Box<dyn OnStateHook>>) {
        self.inner.set_state_hook(_hook)
    }

    fn evm_mut(&mut self) -> &mut Self::Evm {
        self.inner.evm_mut()
    }

    fn evm(&self) -> &Self::Evm {
        self.inner.evm()
    }
}
