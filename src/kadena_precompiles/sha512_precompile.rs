use alloy_primitives::{address, Bytes};
use reth::revm::{precompile::PrecompileWithAddress, primitives::{
    Precompile, PrecompileError, PrecompileOutput, PrecompileResult,
}};
use sha2::Digest;

pub const SHA512: PrecompileWithAddress = PrecompileWithAddress(
    address!("0000000000000000000000000000000000000420"),
    Precompile::Standard(sha512_run),
);

pub fn calc_linear_cost_u32(len: usize, base: u64, word: u64) -> u64 {
    (len as u64 + 32 - 1) / 32 * word + base
}

/// Computes the SHA-512 hash of the input data.
pub fn sha512_run(input: &Bytes, gas_limit: u64) -> PrecompileResult {
    let cost = calc_linear_cost_u32(input.len(), 60, 12);
    if cost > gas_limit {
        Err(PrecompileError::OutOfGas.into())
    } else {
        let output = sha2::Sha512_256::digest(input);
        Ok(PrecompileOutput::new(cost, output.to_vec().into()))
    }
}
