use crate::kadena_precompiles::*;
use reth::revm::{
    context::{
        result::{EVMError, HaltReason}, Context, TxEnv
    },
    inspector::NoOpInspector,
    interpreter::interpreter::EthInterpreter,
    primitives::hardfork::SpecId,
    Inspector,
    MainBuilder,
    MainContext
};
use reth_chainspec::ChainSpec;
use reth_evm::{eth::EthEvmContext, precompiles::PrecompilesMap, Database, EthEvm, EvmFactory};
use reth_evm_ethereum::EthEvmConfig;
use reth_node_api::{FullNodeTypes, NodeTypes};
use reth_node_builder::{components::ExecutorBuilder, BuilderContext};
use reth_primitives::EthPrimitives;

/// Custom EVM configuration
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct KadenaEvmFactory;

impl EvmFactory for KadenaEvmFactory {
    type Evm<DB: Database, I: Inspector<EthEvmContext<DB>, EthInterpreter>> =
        EthEvm<DB, I, Self::Precompiles>;
    type Tx = TxEnv;
    type Error<DBError: std::error::Error + Send + Sync + 'static> = EVMError<DBError>;
    type HaltReason = HaltReason;
    type Context<DB: Database> = EthEvmContext<DB>;
    type Spec = SpecId;
    type Precompiles = PrecompilesMap;

    fn create_evm<DB: Database>(
        &self,
        db: DB,
        input: reth_evm::EvmEnv
    ) -> Self::Evm<DB, NoOpInspector> {
        let evm = Context::mainnet()
            .with_db(db)
            .with_cfg(input.cfg_env)
            .with_block(input.block_env)
            .build_mainnet_with_inspector(NoOpInspector {})
            .with_precompiles(KadenaPrecompiles::new().precompiles_map());
        EthEvm::new(evm, false)
    }

    fn create_evm_with_inspector<DB: Database, I: Inspector<Self::Context<DB>, EthInterpreter>>(
        &self,
        db: DB,
        input: reth_evm::EvmEnv,
        inspector: I,
    ) -> Self::Evm<DB, I> {
        EthEvm::new(self.create_evm(db, input).into_inner().with_inspector(inspector), true)
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
    type EVM = EthEvmConfig<KadenaEvmFactory>;
    async fn build_evm(self, ctx: &BuilderContext<Node>) -> eyre::Result<Self::EVM> {
        let evm_config = EthEvmConfig::new_with_evm_factory(ctx.chain_spec(), KadenaEvmFactory::default());
        Ok(evm_config)
    }
}

