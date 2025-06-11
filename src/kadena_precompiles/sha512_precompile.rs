use alloy_primitives::{address, Address};

use reth::revm::precompile::{PrecompileError, PrecompileOutput, PrecompileWithAddress};
use sha2::Digest;

pub const SHA512_256_ADDR: Address = address!("0000000000000000000000000000000000000420");

pub const SHA512_PRECOMPILE: PrecompileWithAddress = PrecompileWithAddress (SHA512_256_ADDR, sha512_run);

pub fn calc_linear_cost_u32(len: usize, base: u64, word: u64) -> u64 {
    (len as u64 + 32 - 1) / 32 * word + base
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
