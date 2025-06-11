use alloy_primitives::{address, b256, Address, Bytes, FixedBytes, B256};
use alloy_sol_types::{sol, SolValue};
use reth::revm::precompile::{PrecompileError, PrecompileOutput, PrecompileResult, PrecompileWithAddress};
use reth_tracing::tracing::debug;
use serde::{Deserialize, Serialize};

// Keccak256("/Chainweb/KIP-34/VERIFY/SVP/")
pub const BURN_XCHAIN_ADDR: Address = address!("48c3b4d2757447601776837b6a85f31ef88a87bf");
pub const BURN_XCHAIN_EVENT: FixedBytes<32> =
    b256!("a8a9f5b5396df0df430f98612e808d453d59dff13ad5aed824dce438df3ddcf0");

pub const BURN_XCHAIN_PRECOMPILE: PrecompileWithAddress = PrecompileWithAddress (BURN_XCHAIN_ADDR, call_burn);


sol! {
    struct CrosschainOrigin {
        uint32 originChainId;
        address originContractAddress;
        uint64 originBlockHeight;
        uint64 originTransactionIndex;
        uint64 originEventIndex;
    }
}

// Our cross-chain data
sol! {
  struct CrosschainMessage {
    uint32 targetChainId;
    address targetContractAddress;
    uint64 crosschainOperationName;
    bytes crosschainData;
    CrosschainOrigin origin;
  }
}

// The following struct comes from this format we are trying to deserialize:
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
struct XProofOrigin {
    #[serde(rename = "chainId")]
    chain_id: u32,
    contract: B256,
    height: u64,
    #[serde(rename = "transactionIdx")]
    transaction_idx: u64,
    #[serde(rename = "eventIdx")]
    event_idx: u64,
}

// The following struct comes from this format we are trying to deserialize:
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
struct XProofSubject {
    origin: XProofOrigin,
    #[serde(rename = "targetChainId")]
    target_chain_id: u32,
    #[serde(rename = "targetContract")]
    target_contract: B256,
    #[serde(rename = "operationName")]
    operation_type: u64,
    data: Bytes,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
struct XProof {
    chain: u32,
    object: String,
    subject: XProofSubject,
    algorithm: String,
}

fn to_precompile_err<T: ToString>(value: T) -> PrecompileError {
    PrecompileError::other(value.to_string())
}

// Our precompile needs to
// - Query the sister node for the events from the last 100 blocks from some address
// - The bytes parameter is going to the be the address of the contract to query the events from
//

pub fn call_burn(bytes: &[u8], _gas_limit: u64) -> PrecompileResult {
    let proof_slice = bytes.as_ref();
    debug!("Proof bytes hex {:?}", bytes);

    let xchain_proof: XProof =
        serde_json::from_slice(proof_slice).map_err(to_precompile_err)?;
    debug!("Decoded xchain proof: {:?}", xchain_proof);

    let origin_contract_addr = Address::from_word(xchain_proof.subject.origin.contract);
    let target_contract_addr = Address::from_word(xchain_proof.subject.target_contract);
    let target_bytes: Bytes =
        alloy_primitives::Bytes::abi_decode(&xchain_proof.subject.data.as_ref())
            .map_err(to_precompile_err)?;

    let cx_origin: CrosschainOrigin = CrosschainOrigin {
        originChainId: xchain_proof.subject.origin.chain_id,
        originContractAddress: origin_contract_addr,
        originBlockHeight: xchain_proof.subject.origin.height,
        originTransactionIndex: xchain_proof.subject.origin.transaction_idx,
        originEventIndex: xchain_proof.subject.origin.event_idx,
    };
    let output = CrosschainMessage {
        crosschainOperationName: xchain_proof.subject.operation_type,
        targetChainId: xchain_proof.subject.target_chain_id,
        targetContractAddress: target_contract_addr,
        crosschainData: target_bytes,
        origin: cx_origin,
    };
    let encoded = Bytes::from(output.abi_encode());
    let out = PrecompileOutput::new(1000, encoded);
    Ok(out)
}


#[cfg(test)]
mod test {
    use alloy_primitives::{b256, bytes};

    use crate::kadena_precompiles::spv_precompile::{XProofOrigin, XProofSubject};

    use super::XProof;

    #[test]
    fn test_xproof_decoding() {
        let json_data = String::from("{\"chain\":1,\"object\":\"U05BS0VPSUw\",\"subject\":{\"data\":\"0x0000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000010000000000000000000000005c8b984deb026110310f617c5dba96fd39704835000000000000000000000000000000000000000000000000000000000000000a\",\"operationName\":1,\"origin\":{\"chainId\":0,\"contract\":\"0x0000000000000000000000005c8b984deb026110310f617c5dba96fd39704835\",\"eventIdx\":1,\"height\":35,\"transactionIdx\":0},\"targetChainId\":1,\"targetContract\":\"0x0000000000000000000000008849babddcfc1327ad199877861b577cebd8a7b6\"},\"algorithm\":\"SHA512t_256\"}");

        let expected_origin = XProofOrigin {
            chain_id: 0,
            contract: b256!("0000000000000000000000005c8b984deb026110310f617c5dba96fd39704835"),
            height: 35,
            transaction_idx: 0,
            event_idx: 1,
        };

        let expected_subject: XProofSubject = XProofSubject {
        origin: expected_origin,
        target_chain_id: 1,
        target_contract: b256!("0000000000000000000000008849babddcfc1327ad199877861b577cebd8a7b6"),
        operation_type: 1,
        data: bytes!("0000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000010000000000000000000000005c8b984deb026110310f617c5dba96fd39704835000000000000000000000000000000000000000000000000000000000000000a"),
    };

        let expected: XProof = XProof {
            chain: 1,
            object: String::from("U05BS0VPSUw"),
            subject: expected_subject,
            algorithm: String::from("SHA512t_256"),
        };

        let decoded: XProof =
            serde_json::from_str(&json_data).expect("failed to deserialize XProof");

        assert_eq!(decoded, expected);
    }
}
