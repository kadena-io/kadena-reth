use crate::kadena_precompiles::*;
use reth::revm::{
    context::{
        result::{EVMError, HaltReason}, Context, TxEnv
    }, inspector, interpreter::interpreter::EthInterpreter, primitives::hardfork::SpecId, Inspector, MainBuilder, MainContext
};
use reth_chainspec::ChainSpec;
use reth_evm::{eth::EthEvmContext, Database, EthEvm, EvmFactory};
use reth_evm_ethereum::EthEvmConfig;
use reth_node_api::{FullNodeTypes, NodeTypes};
use reth_node_builder::{components::ExecutorBuilder, BuilderContext};
use reth_node_ethereum::BasicBlockExecutorProvider;
use reth_primitives::EthPrimitives;

/// Custom EVM configuration
#[derive(Clone, Debug, Default)]
#[non_exhaustive]
pub struct KadenaEvmFactory;

impl EvmFactory for KadenaEvmFactory {
    type Evm<DB: Database, I: Inspector<EthEvmContext<DB>, EthInterpreter>> =
        EthEvm<DB, I, KadenaPrecompiles>;

    type Context<DB: Database> = EthEvmContext<DB>;

    type Tx = TxEnv;

    type Error<DBError: std::error::Error + Send + Sync + 'static> = EVMError<DBError>;

    type HaltReason = HaltReason;

    type Spec = SpecId;

    fn create_evm<DB: Database>(
        &self,
        db: DB,
        evm_env: reth_evm::EvmEnv<Self::Spec>,
    ) -> Self::Evm<DB, reth::revm::inspector::NoOpInspector> {
        let evm = Context::mainnet()
            .with_db(db)
            .with_cfg(evm_env.cfg_env)
            .with_block(evm_env.block_env)
            .build_mainnet()
            .with_precompiles(KadenaPrecompiles::new())
            .with_inspector(inspector::NoOpInspector);

        EthEvm::new(evm, false)

    }

    fn create_evm_with_inspector<DB: Database, I: Inspector<Self::Context<DB>>>(
        &self,
        db: DB,
        input: reth_evm::EvmEnv<Self::Spec>,
        inspector: I,
    ) -> Self::Evm<DB, I> {
        EthEvm::new(self.create_evm(db, input).into_inner().with_inspector(inspector), true)
    }
}

// Builds a regular ethereum block executor that uses the custom EVM.
#[derive(Clone, Default)]
pub struct KadenaExecutorBuilder;

impl<Node> ExecutorBuilder<Node> for KadenaExecutorBuilder
where
    Node: FullNodeTypes<Types: NodeTypes<ChainSpec = ChainSpec, Primitives = EthPrimitives>>,
{
    type EVM = EthEvmConfig<KadenaEvmFactory>;
    type Executor = BasicBlockExecutorProvider<Self::EVM>;

    async fn build_evm(
        self,
        ctx: &BuilderContext<Node>,
    ) -> eyre::Result<(Self::EVM, Self::Executor)> {
        let evm_config = EthEvmConfig::new_with_evm_factory(ctx.chain_spec(), KadenaEvmFactory::default());
        Ok((
            evm_config.clone(),
            BasicBlockExecutorProvider::new(evm_config),
        ))
    }
}