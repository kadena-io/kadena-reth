use crate::kadena_precompiles::*;
use reth::{payload::PayloadBuilderService, providers::CanonStateSubscriptions, revm::{
    context::{
        result::{EVMError, HaltReason}, Context, TxEnv
    },
    inspector::NoOpInspector,
    interpreter::interpreter::EthInterpreter,
    primitives::hardfork::SpecId,
    Inspector,
    MainBuilder,
    MainContext
}, transaction_pool::{PoolTransaction, TransactionPool}};
use reth_chainspec::ChainSpec;
use reth_evm::{eth::EthEvmContext, precompiles::PrecompilesMap, Database, EthEvm, EvmFactory};
use reth_evm_ethereum::EthEvmConfig;
use reth_node_api::{FullNodeTypes, NodeTypes};
use reth_node_builder::{components::ExecutorBuilder, components::PayloadServiceBuilder, BuilderContext, PayloadBuilderConfig};
use reth_primitives::EthPrimitives;
use reth_node_ethereum::EthEngineTypes;
use reth_basic_payload_builder::{BasicPayloadJobGenerator, BasicPayloadJobGeneratorConfig};
use reth_ethereum::TransactionSigned;
use reth_ethereum_payload_builder::{EthereumBuilderConfig};
use reth_payload_builder::{PayloadBuilderHandle};


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
    Node: FullNodeTypes<Types: NodeTypes<Payload = EthEngineTypes, ChainSpec = ChainSpec, Primitives = EthPrimitives>>,
{
    type EVM = EthEvmConfig<KadenaEvmFactory>;
    async fn build_evm(self, ctx: &BuilderContext<Node>) -> eyre::Result<Self::EVM> {
        let evm_config = EthEvmConfig::new_with_evm_factory(ctx.chain_spec(), KadenaEvmFactory::default());
        Ok(evm_config)
    }
}

#[derive(Debug, Default, Clone, Copy)]
#[non_exhaustive]
pub struct KadenaPayloadBuilder;
impl<Node, Pool> PayloadServiceBuilder<Node, Pool, EthEvmConfig<KadenaEvmFactory>> for KadenaPayloadBuilder
where
    Node: FullNodeTypes<
        Types: NodeTypes<
            Payload = EthEngineTypes,
            ChainSpec = ChainSpec,
            Primitives = EthPrimitives,
        >,
    >,
    Pool: TransactionPool<Transaction: PoolTransaction<Consensus = TransactionSigned>>
        + Unpin
        + 'static,
{
    async fn spawn_payload_builder_service(
        self,
        ctx: &BuilderContext<Node>,
        pool: Pool,
        evm_config: EthEvmConfig<KadenaEvmFactory>,
    ) -> eyre::Result<PayloadBuilderHandle<<Node::Types as NodeTypes>::Payload>> {

        let payload_builder = reth_ethereum_payload_builder::EthereumPayloadBuilder::new(
            ctx.provider().clone(),
            pool,
            evm_config,
            EthereumBuilderConfig::new(),
        );

        let conf = ctx.payload_builder_config();

        let payload_job_config = BasicPayloadJobGeneratorConfig::default()
            .interval(conf.interval())
            .nodeadline()
            .keep_payload_jobs_alive();

        let payload_generator = BasicPayloadJobGenerator::with_builder(
            ctx.provider().clone(),
            ctx.task_executor().clone(),
            payload_job_config,
            payload_builder,
        );

        let (payload_service, payload_builder) =
            PayloadBuilderService::new(payload_generator, ctx.provider().canonical_state_stream(), conf.max_payload_tasks());

        ctx.task_executor()
            .spawn_critical("custom payload builder service", Box::pin(payload_service));

        Ok(payload_builder)
    }
}
