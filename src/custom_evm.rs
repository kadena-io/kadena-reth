use crate::kadena_precompiles::*;
use std::env;
use alloy_primitives::{Address, Bytes};
use reth::{builder::{
    components::{ExecutorBuilder, PayloadServiceBuilder},
    BuilderContext,
}, revm::{precompile::PrecompileWithAddress, primitives::Precompile}, transaction_pool::{PoolTransaction, TransactionPool}};
use reth::payload::{EthBuiltPayload, EthPayloadBuilderAttributes};
use reth::revm::{
    handler::register::EvmHandler, inspector_handle_register, precompile::PrecompileSpecId,
    ContextPrecompile, ContextPrecompiles, Database, Evm, EvmBuilder, GetInspector,
};
// use reth::revm::rpc::types::engine::PayloadAttributes;
// use reth::revm::transaction_pool::TransactionPool;
use reth::revm::primitives::{CfgEnvWithHandlerCfg, Env, TxEnv};
use reth_chainspec::ChainSpec;
use reth_evm::env::EvmEnv;
use reth_evm_ethereum::EthEvmConfig;
use reth_node_api::{
    ConfigureEvm, ConfigureEvmEnv, FullNodeTypes, NextBlockEnvAttributes, NodeTypes, NodeTypesWithEngine, PayloadTypes
};
use reth::rpc::types::engine::PayloadAttributes;
use reth_node_ethereum::{
    node::EthereumPayloadBuilder, BasicBlockExecutorProvider, EthExecutionStrategyFactory,
};
use reth_primitives::{
    // revm::primitives::{CfgEnvWithHandlerCfg, TxEnv},
    EthPrimitives,
    Header,
    TransactionSigned,
};
use std::{convert::Infallible, sync::Arc};

/// Custom EVM configuration
#[derive(Clone)]
#[non_exhaustive]
pub struct KadenaEvmConfig {
    /// Wrapper around mainnet configuration
    inner: EthEvmConfig,
}

impl KadenaEvmConfig {
    pub fn new(chain_spec: Arc<ChainSpec>) -> Self {
        Self {
            inner: EthEvmConfig::new(chain_spec),
        }
    }

    /// Sets the precompiles to the EVM handler
    ///
    /// This will be invoked when the EVM is created via [ConfigureEvm::evm] or
    /// [ConfigureEvm::evm_with_inspector]
    ///
    /// This will use the default mainnet precompiles and add additional precompiles.
    pub fn set_precompiles<EXT, DB>(&self, handler: &mut EvmHandler<EXT, DB>)
    where
        DB: Database,
    {
        // TODO: REMOVE THIS. THIS IS A HACK
        let chainweb_chain_id: u32 = env::var("CHAINWEB_CHAIN_ID").unwrap().parse().unwrap();
        // first we need the evm spec id, which determines the precompiles
        // based on the eth version
        let spec_id = handler.cfg.spec_id;

        // install the precompiles
        handler.pre_execution.load_precompiles = Arc::new(move || {
            let mut precompiles = ContextPrecompiles::new(PrecompileSpecId::from_spec_id(spec_id));
            let chain_id_precompile = ChainIdPrecompile::new(chainweb_chain_id);
            precompiles.extend([PrecompileWithAddress(CHAIN_ID_PRECOMPILE_ADDR, Precompile::new_stateful(chain_id_precompile))]);
            precompiles.extend([SHA512]);
            precompiles.extend([(
                BURN_XCHAIN_ADDR,
                ContextPrecompile::ContextStatefulMut(Box::new(BurnPrecompile)),
            )]);
            precompiles
        });
    }
}

impl ConfigureEvmEnv for KadenaEvmConfig {
    type Error = Infallible;
    type Header = Header;
    type Transaction = TransactionSigned;

    fn fill_tx_env(&self, tx_env: &mut TxEnv, transaction: &TransactionSigned, sender: Address) {
        self.inner.fill_tx_env(tx_env, transaction, sender);
    }

    fn fill_tx_env_system_contract_call(
        &self,
        env: &mut Env,
        caller: Address,
        contract: Address,
        data: Bytes,
    ) {
        self.inner
            .fill_tx_env_system_contract_call(env, caller, contract, data);
    }

    fn fill_cfg_env(&self, cfg_env: &mut CfgEnvWithHandlerCfg, header: &Self::Header) {
        self.inner.fill_cfg_env(cfg_env, header);
    }

    fn next_cfg_and_block_env(
        &self,
        parent: &Self::Header,
        attributes: NextBlockEnvAttributes,
    ) -> Result<EvmEnv, Self::Error> {
        self.inner.next_cfg_and_block_env(parent, attributes)
    }
}

impl ConfigureEvm for KadenaEvmConfig {
    type DefaultExternalContext<'a> = ();

    fn evm<DB: Database>(&self, db: DB) -> Evm<'_, Self::DefaultExternalContext<'_>, DB> {
        EvmBuilder::default()
            .with_db(db)
            // add additional precompiles
            .append_handler_register_box(Box::new(|x| self.set_precompiles(x)))
            .build()
    }

    fn evm_with_inspector<DB, I>(&self, db: DB, inspector: I) -> Evm<'_, I, DB>
    where
        DB: Database,
        I: GetInspector<DB>,
    {
        EvmBuilder::default()
            .with_db(db)
            .with_external_context(inspector)
            // add additional precompiles
            .append_handler_register_box(Box::new(|x| self.set_precompiles(x)))
            .append_handler_register(inspector_handle_register)
            .build()
    }

    fn default_external_context<'a>(&self) -> Self::DefaultExternalContext<'a> {}
}

/// Builds a regular ethereum block executor that uses the custom EVM.
#[derive(Clone, Default)]
#[non_exhaustive]
pub struct KadenaExecutorBuilder;

impl<Node> ExecutorBuilder<Node> for KadenaExecutorBuilder
where
    Node: FullNodeTypes<Types: NodeTypes<ChainSpec = ChainSpec, Primitives = EthPrimitives>>,
{
    type EVM = KadenaEvmConfig;
    type Executor = BasicBlockExecutorProvider<EthExecutionStrategyFactory<Self::EVM>>;

    async fn build_evm(
        self,
        ctx: &BuilderContext<Node>,
    ) -> eyre::Result<(Self::EVM, Self::Executor)> {
        Ok((
            KadenaEvmConfig::new(ctx.chain_spec()),
            BasicBlockExecutorProvider::new(EthExecutionStrategyFactory::new(
                ctx.chain_spec(),
                KadenaEvmConfig::new(ctx.chain_spec()),
            )),
        ))
    }
}

/// Builds a regular ethereum block executor that uses the custom EVM.
#[derive(Clone, Default)]
#[non_exhaustive]
pub struct KadenaPayloadBuilder {
    pub inner: EthereumPayloadBuilder,
}

impl<Types, Node, Pool> PayloadServiceBuilder<Node, Pool> for KadenaPayloadBuilder
where
    Types: NodeTypesWithEngine<ChainSpec = ChainSpec, Primitives = EthPrimitives>,
    Node: FullNodeTypes<Types = Types>,
    Pool: TransactionPool<Transaction: PoolTransaction<Consensus = TransactionSigned>>
        + Unpin
        + 'static,
    Types::Engine: PayloadTypes<
        BuiltPayload = EthBuiltPayload,
        PayloadAttributes = PayloadAttributes,
        PayloadBuilderAttributes = EthPayloadBuilderAttributes,
    >,
{
    async fn spawn_payload_service(
        self,
        ctx: &BuilderContext<Node>,
        pool: Pool,
    ) -> eyre::Result<reth::payload::PayloadBuilderHandle<Types::Engine>> {
        self.inner
            .spawn(KadenaEvmConfig::new(ctx.chain_spec()), ctx, pool)
    }
}
