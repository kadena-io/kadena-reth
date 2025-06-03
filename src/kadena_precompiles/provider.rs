use std::sync::{Arc, RwLock};

use alloy_primitives::{Address, Bytes};
use once_cell::race::OnceBox;
use reth::revm::{
    context::{Cfg, ContextTr},
    handler::{EthPrecompiles, PrecompileProvider},
    interpreter::{Gas, InputsImpl, InstructionResult, InterpreterResult},
    precompile::Precompiles,
    primitives::hardfork::SpecId,
};

use super::{
    KadenaMint, KadenaMintPrecompile, BURN_XCHAIN_ADDR, BURN_XCHAIN_PRECOMPILE, MINT_XCHAIN_ADDR,
    SHA512_256_ADDR, SHA512_PRECOMPILE,
};

pub struct KadenaPrecompiles {
    eth_precompiles: EthPrecompiles,
    mints: KadenaMintPrecompile,
}

impl KadenaPrecompiles {
    pub fn new(mints: Arc<RwLock<Vec<KadenaMint>>>) -> Self {
        let spec = SpecId::default();
        let eth_precompiles = EthPrecompiles {
            precompiles: KadenaPrecompiles::kadena_precompiles(),
            spec,
        };
        Self {
            eth_precompiles: eth_precompiles,
            mints: KadenaMintPrecompile::new(mints.clone()),
        }
    }

    fn kadena_precompiles() -> &'static Precompiles {
        static INSTANCE: OnceBox<Precompiles> = OnceBox::new();
        INSTANCE.get_or_init(|| {
            let mut precompiles = Precompiles::prague().clone();
            precompiles.extend([SHA512_PRECOMPILE, BURN_XCHAIN_PRECOMPILE]);
            Box::new(precompiles)
        })
    }
}

impl<CTX: ContextTr> PrecompileProvider<CTX> for KadenaPrecompiles {
    type Output = InterpreterResult;

    fn set_spec(&mut self, _spec: <CTX::Cfg as Cfg>::Spec) -> bool {
        true
    }

    fn run(
        &mut self,
        context: &mut CTX,
        address: &Address,
        inputs: &InputsImpl,
        is_static: bool,
        gas_limit: u64,
    ) -> Result<Option<Self::Output>, String> {
        if self.eth_precompiles.contains(address) {
            return self
                .eth_precompiles
                .run(context, address, inputs, is_static, gas_limit);
        }

        if *address == MINT_XCHAIN_ADDR {
            let mut result = InterpreterResult {
                result: InstructionResult::Return,
                gas: Gas::new(gas_limit),
                output: Bytes::new(),
            };

            match self.mints.call_mint(&inputs.input , gas_limit) {
                Ok(output) => {
                    result.result = InstructionResult::Return;

                    // Todo: is this safe? May need an assert here or to throw a benign error
                    let _ = result.gas.record_cost(output.gas_used);
                    result.output = output.bytes;
                    return Ok(Some(result));
                }
                Err(e) => {
                    return Err(format!("Failed to call KadenaMint precompile: {}", e));
                }
            }

        }

        Ok(None)
    }

    /// Returns addresses of the precompiles.
    fn warm_addresses(&self) -> Box<impl Iterator<Item = Address>> {
        let mut precompiles = self.eth_precompiles.precompiles.addresses_set().clone();
        precompiles.insert(BURN_XCHAIN_ADDR);
        precompiles.insert(SHA512_256_ADDR);
        precompiles.insert(MINT_XCHAIN_ADDR);
        Box::new(precompiles.into_iter())
    }

    fn contains(&self, address: &Address) -> bool {
        self.eth_precompiles.contains(address)
            || *address == BURN_XCHAIN_ADDR
            || *address == SHA512_256_ADDR
            || *address == MINT_XCHAIN_ADDR
    }
}
