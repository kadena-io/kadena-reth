use alloy_primitives::{address, Address, Bytes};
use reth::revm::primitives::{
       PrecompileOutput, PrecompileResult, StatefulPrecompile,
    };


pub const CHAIN_ID_PRECOMPILE_ADDR: Address = address!("0000000000000000000000000000000000000422");
pub struct ChainIdPrecompile {
    chainweb_chain_id: u32,
}

impl ChainIdPrecompile {
  pub fn new(chainweb_chain_id: u32) -> ChainIdPrecompile {
    ChainIdPrecompile {
        chainweb_chain_id,
    }
  }
}


impl StatefulPrecompile for ChainIdPrecompile {
    fn call(
        &self,
        _bytes: &Bytes,
        _gas_limit: u64,
        _env: &reth::revm::primitives::Env,
    ) -> PrecompileResult {
        Ok(PrecompileOutput::new(5, self.chainweb_chain_id.to_be_bytes().into()))
    }
}
