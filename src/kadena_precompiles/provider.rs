use alloy_primitives::Address;
use once_cell::race::OnceBox;
use reth::revm::{
    context::{Cfg, ContextTr},
    handler::{EthPrecompiles, PrecompileProvider},
    interpreter::{InputsImpl, InterpreterResult},
    precompile::Precompiles,
    primitives::hardfork::SpecId,
};
use reth_evm::precompiles::PrecompilesMap;

use crate::kadena_precompiles::Sha512Precompile;

use super::{SHA512_256_ADDR};

pub struct KadenaPrecompiles {
    pub eth_precompiles: EthPrecompiles,
}

impl KadenaPrecompiles {
    pub fn new() -> Self {
        let spec = SpecId::default();
        let eth_precompiles = EthPrecompiles { 
            precompiles: KadenaPrecompiles::kadena_precompiles(),
            spec,
        };
        Self { eth_precompiles, }
    }

    fn kadena_precompiles() -> &'static Precompiles {
        static INSTANCE: OnceBox<Precompiles> = OnceBox::new();
        INSTANCE.get_or_init(|| {
            let mut precompiles = Precompiles::prague().clone();
            precompiles.extend([Sha512Precompile::precompile()]);
            Box::new(precompiles)
        })
    }

    pub fn precompiles_map(&self) -> PrecompilesMap {
        PrecompilesMap::from_static(self.eth_precompiles.precompiles)
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


        Ok(None)
    }

    /// Returns addresses of the precompiles.
    fn warm_addresses(&self) -> Box<impl Iterator<Item = Address>> {
        let mut precompiles = self.eth_precompiles.precompiles.addresses_set().clone();
        precompiles.insert(SHA512_256_ADDR);
        Box::new(precompiles.into_iter())
    }

    fn contains(&self, address: &Address) -> bool {
        self.eth_precompiles.contains(address)
            || *address == SHA512_256_ADDR
    }


}
