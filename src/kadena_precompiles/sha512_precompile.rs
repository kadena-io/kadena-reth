use alloy_primitives::{address, Address};

use reth::revm::precompile::{Precompile, PrecompileError, PrecompileId, PrecompileOutput};
use sha2::Digest;

pub const SHA512_256_ADDR: Address = address!("0000000000000000000000000000000000000420");

pub const SHA512_PRECOMPILE_ID_NAME: &'static str = "sha512_256";


pub fn calc_linear_cost_u32(len: usize, base: u64, word: u64) -> u64 {
    (len as u64 + 32 - 1) / 32 * word + base
}

pub struct Sha512Precompile;

impl Sha512Precompile {
    pub fn precompile() -> Precompile {
        Precompile::new (PrecompileId::custom(SHA512_PRECOMPILE_ID_NAME),SHA512_256_ADDR, sha512_run)
    }
}

/// Computes the SHA-512 hash of the input data.
pub fn sha512_run(input: &[u8], gas_limit: u64) -> Result<PrecompileOutput, PrecompileError> {
    let cost = calc_linear_cost_u32(input.len(), 60, 12);
    if cost > gas_limit {
        Err(PrecompileError::OutOfGas.into())
    } else {
        let output = sha2::Sha512_256::digest(input);
        Ok(PrecompileOutput::new(cost, output.to_vec().into()))
    }
}
