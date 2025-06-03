use std::sync::{Arc, RwLock};

use crate::kadena_precompiles::*;
use reth::revm::{
    context::{
        result::{EVMError, HaltReason},
        Context, TxEnv,
    },
    inspector::NoOpInspector,
    interpreter::interpreter::EthInterpreter,
    primitives::hardfork::SpecId,
    Inspector, MainBuilder, MainContext,
};
use reth_chainspec::ChainSpec;
use reth_evm::{
    block::BlockExecutorFactory,
    eth::{EthBlockExecutionCtx, EthBlockExecutor, EthEvmContext},
    ConfigureEvm, Database, EthEvm, EvmFactory, ExecutionCtxFor,
};
use reth_evm_ethereum::{EthBlockAssembler, EthEvmConfig};
use reth_node_api::{FullNodeTypes, NodeTypes};
use reth_node_builder::{components::ExecutorBuilder, BuilderContext};
use reth_primitives::{EthPrimitives, Receipt, TransactionSigned};

/// Custom EVM configuration
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct KadenaEvmFactory {
    mints: Arc<RwLock<Vec<KadenaMint>>>,
}

impl EvmFactory for KadenaEvmFactory {
    type Evm<DB: Database, I: Inspector<EthEvmContext<DB>, EthInterpreter>> =
        EthEvm<DB, I, KadenaPrecompiles>;
    type Tx = TxEnv;
    type Error<DBError: std::error::Error + Send + Sync + 'static> = EVMError<DBError>;
    type HaltReason = HaltReason;
    type Context<DB: Database> = EthEvmContext<DB>;
    type Spec = SpecId;

    fn create_evm<DB: Database>(
        &self,
        db: DB,
        input: reth_evm::EvmEnv,
    ) -> Self::Evm<DB, NoOpInspector> {
        let evm = Context::mainnet()
            .with_db(db)
            .with_cfg(input.cfg_env)
            .with_block(input.block_env)
            .build_mainnet_with_inspector(NoOpInspector {})
            .with_precompiles(KadenaPrecompiles::new(self.mints.clone()));

        EthEvm::new(evm, false)
    }

    fn create_evm_with_inspector<DB: Database, I: Inspector<Self::Context<DB>, EthInterpreter>>(
        &self,
        db: DB,
        input: reth_evm::EvmEnv,
        inspector: I,
    ) -> Self::Evm<DB, I> {
        EthEvm::new(
            self.create_evm(db, input)
                .into_inner()
                .with_inspector(inspector),
            true,
        )
    }
}
#[derive(Debug, Clone)]
pub struct KadenaEvmConfig {
    pub inner: EthEvmConfig<KadenaEvmFactory>,
}

impl Unpin for KadenaEvmConfig {}

unsafe impl Send for KadenaEvmConfig {}

unsafe impl Sync for KadenaEvmConfig {}

impl KadenaEvmConfig {
    /// Creates a new instance of the Kadena EVM configuration.
    pub fn new(chain_spec: Arc<ChainSpec>) -> Self {
        let evm_factory = KadenaEvmFactory::default();
        let inner = EthEvmConfig::new_with_evm_factory(chain_spec, evm_factory);
        Self { inner }
    }
}

impl BlockExecutorFactory for KadenaEvmConfig {
    type EvmFactory = KadenaEvmFactory;
    type ExecutionCtx<'a> = EthBlockExecutionCtx<'a>;
    type Transaction = TransactionSigned;
    type Receipt = Receipt;

    fn evm_factory(&self) -> &Self::EvmFactory {
        &self.inner.evm_factory()
    }

    fn create_executor<'a, DB, I>(
        &'a self,
        evm: <Self::EvmFactory as EvmFactory>::Evm<&'a mut reth::revm::State<DB>, I>,
        ctx: Self::ExecutionCtx<'a>,
    ) -> impl reth_evm::block::BlockExecutorFor<'a, Self, DB, I>
    where
        DB: Database + 'a,
        I: Inspector<<Self::EvmFactory as EvmFactory>::Context<&'a mut reth::revm::State<DB>>> + 'a,
    {
        let mints = Arc::new(RwLock::new(Vec::new()));
        KadenaBlockExecutor {
            inner: EthBlockExecutor::new(evm, ctx, self.inner.chain_spec(),self.inner.executor_factory.receipt_builder()),
            mints
        }
    }
}

impl ConfigureEvm for KadenaEvmConfig {
    #[doc = " The primitives type used by the EVM."]
    type Primitives = <EthEvmConfig as ConfigureEvm>::Primitives;

    #[doc = " The error type that is returned by [`Self::next_evm_env`]."]
    type Error = <EthEvmConfig as ConfigureEvm>::Error;

    #[doc = " Context required for configuring next block environment."]
    #[doc = ""]
    #[doc = " Contains values that can\'t be derived from the parent block."]
    type NextBlockEnvCtx = <EthEvmConfig as ConfigureEvm>::NextBlockEnvCtx;

    #[doc = " Configured [`BlockExecutorFactory`], contains [`EvmFactory`] internally."]
    type BlockExecutorFactory = Self;

    #[doc = " A type that knows how to build a block."]
    type BlockAssembler = EthBlockAssembler;

    #[doc = " Returns reference to the configured [`BlockExecutorFactory`]."]
    fn block_executor_factory(&self) -> &Self::BlockExecutorFactory {
        self
    }

    #[doc = " Returns reference to the configured [`BlockAssembler`]."]
    fn block_assembler(&self) -> &Self::BlockAssembler {
        self.inner.block_assembler()
    }

    #[doc = " Creates a new [`EvmEnv`] for the given header."]
    fn evm_env(&self, header: &reth_primitives::HeaderTy<Self::Primitives>) -> reth_evm::EvmEnvFor<Self> {
        self.inner.evm_env(header)
    }

    #[doc = " Returns the configured [`EvmEnv`] for `parent + 1` block."]
    #[doc = ""]
    #[doc = " This is intended for usage in block building after the merge and requires additional"]
    #[doc = " attributes that can\'t be derived from the parent block: attributes that are determined by"]
    #[doc = " the CL, such as the timestamp, suggested fee recipient, and randomness value."]
    fn next_evm_env(
        &self,
        parent: &reth_primitives::HeaderTy<Self::Primitives>,
        attributes: &Self::NextBlockEnvCtx,
    ) -> Result<reth_evm::EvmEnvFor<Self>, Self::Error> {
        self.inner.next_evm_env(parent, attributes)
    }

    #[doc = " Returns the configured [`BlockExecutorFactory::ExecutionCtx`] for a given block."]
    fn context_for_block<'a>(
        &self,
        block: &'a reth_primitives::SealedBlock<reth_primitives::BlockTy<Self::Primitives>>,
    ) -> ExecutionCtxFor<'a, Self> {
        self.inner.context_for_block(block)
    }

    #[doc = " Returns the configured [`BlockExecutorFactory::ExecutionCtx`] for `parent + 1`"]
    #[doc = " block."]
    fn context_for_next_block(
        &self,
        parent: &reth_primitives::SealedHeader<reth_primitives::HeaderTy<Self::Primitives>>,
        attributes: Self::NextBlockEnvCtx,
    ) -> ExecutionCtxFor<'_, Self> {
        self.inner.context_for_next_block(parent, attributes)
    }
}

// Builds a regular ethereum block executor that uses the custom EVM.
#[derive(Debug, Default, Clone, Copy)]
#[non_exhaustive]
pub struct KadenaExecutorBuilder;

impl<Node> ExecutorBuilder<Node> for KadenaExecutorBuilder
where
    Node: FullNodeTypes<Types: NodeTypes<ChainSpec = ChainSpec, Primitives = EthPrimitives>>,
{
    type EVM = KadenaEvmConfig;
    async fn build_evm(self, ctx: &BuilderContext<Node>) -> eyre::Result<Self::EVM> {
        let evm_config = KadenaEvmConfig::new(ctx.chain_spec().clone());
        Ok(evm_config)
    }
}
