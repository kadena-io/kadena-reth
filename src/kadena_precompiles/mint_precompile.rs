use std::sync::{Arc, RwLock};

use alloy_sol_types::SolValue;
use alloy_primitives::{address, Address, Bytes};
use reth::revm::precompile::{PrecompileError, PrecompileOutput, PrecompileResult};

pub const MINT_XCHAIN_ADDR: Address = address!("0000000000000000000000000000000000000425");


#[doc = "```solidity\nstruct KadenaMint { uint128 mintAmount; address targetAddress; }\n```"]
#[allow(non_camel_case_types,non_snake_case,clippy::pub_underscore_fields)]
#[derive(Debug, Clone)]
pub struct KadenaMint {
    #[allow(missing_docs)]
    pub mintAmount:u128, #[allow(missing_docs)]
    pub targetAddress: ::alloy_sol_types::private::Address
}
#[allow(non_camel_case_types,non_snake_case,clippy::pub_underscore_fields,clippy::style)]
const _:() = {
    use::alloy_sol_types as alloy_sol_types;
    #[doc(hidden)]
    type UnderlyingSolTuple<'a>  = (::alloy_sol_types::sol_data::Uint<128> , ::alloy_sol_types::sol_data::Address,);
    #[doc(hidden)]
    type UnderlyingRustTuple<'a>  = (u128, ::alloy_sol_types::private::Address,);
    #[cfg(test)]
    #[allow(dead_code,unreachable_patterns)]
    fn _type_assertion(_t:alloy_sol_types::private::AssertTypeEq<UnderlyingRustTuple>){
        match _t {
            alloy_sol_types::private::AssertTypeEq:: <<UnderlyingSolTuple as alloy_sol_types::SolType> ::RustType>(_) => {}


            }
    }
    #[automatically_derived]
    #[doc(hidden)]
    impl ::core::convert::From<KadenaMint>for UnderlyingRustTuple<'_>{
        fn from(value:KadenaMint) -> Self {
            (value.mintAmount,value.targetAddress,)
        }

        }
    #[automatically_derived]
    #[doc(hidden)]
    impl ::core::convert::From<UnderlyingRustTuple<'_>>for KadenaMint {
        fn from(tuple:UnderlyingRustTuple<'_>) -> Self {
            Self {
                mintAmount:tuple.0,targetAddress:tuple.1
            }
        }

        }
    #[automatically_derived]
    impl alloy_sol_types::SolValue for KadenaMint {
        type SolType = Self;
    }
    #[automatically_derived]
    impl alloy_sol_types::private::SolTypeValue<Self>for KadenaMint {
        #[inline]
        fn stv_to_tokens(&self) ->  <Self as alloy_sol_types::SolType> ::Token<'_>{
            (< ::alloy_sol_types::sol_data::Uint<128>as alloy_sol_types::SolType> ::tokenize(&self.mintAmount), < ::alloy_sol_types::sol_data::Address as alloy_sol_types::SolType> ::tokenize(&self.targetAddress),)
        }
        #[inline]
        fn stv_abi_encoded_size(&self) -> usize {
            if let Some(size) =  <Self as alloy_sol_types::SolType> ::ENCODED_SIZE {
                return size;
            }let tuple =  <UnderlyingRustTuple<'_>as ::core::convert::From<Self>> ::from(self.clone());
            <UnderlyingSolTuple<'_>as alloy_sol_types::SolType> ::abi_encoded_size(&tuple)
        }
        #[inline]
        fn stv_eip712_data_word(&self) -> alloy_sol_types::Word {
            <Self as alloy_sol_types::SolStruct> ::eip712_hash_struct(self)
        }
        #[inline]
        fn stv_abi_encode_packed_to(&self,out: &mut alloy_sol_types::private::Vec<u8>){
            let tuple =  <UnderlyingRustTuple<'_>as ::core::convert::From<Self>> ::from(self.clone());
            <UnderlyingSolTuple<'_>as alloy_sol_types::SolType> ::abi_encode_packed_to(&tuple,out)
        }
        #[inline]
        fn stv_abi_packed_encoded_size(&self) -> usize {
            if let Some(size) =  <Self as alloy_sol_types::SolType> ::PACKED_ENCODED_SIZE {
                return size;
            }let tuple =  <UnderlyingRustTuple<'_>as ::core::convert::From<Self>> ::from(self.clone());
            <UnderlyingSolTuple<'_>as alloy_sol_types::SolType> ::abi_packed_encoded_size(&tuple)
        }

        }
    #[automatically_derived]
    impl alloy_sol_types::SolType for KadenaMint {
        type RustType = Self;
        type Token<'a>  =  <UnderlyingSolTuple<'a>as alloy_sol_types::SolType> ::Token<'a> ;
        const SOL_NAME: &'static str =  <Self as alloy_sol_types::SolStruct> ::NAME;
        const ENCODED_SIZE:Option<usize>  =  <UnderlyingSolTuple<'_>as alloy_sol_types::SolType> ::ENCODED_SIZE;
        const PACKED_ENCODED_SIZE:Option<usize>  =  <UnderlyingSolTuple<'_>as alloy_sol_types::SolType> ::PACKED_ENCODED_SIZE;
        #[inline]
        fn valid_token(token: &Self::Token<'_>) -> bool {
            <UnderlyingSolTuple<'_>as alloy_sol_types::SolType> ::valid_token(token)
        }
        #[inline]
        fn detokenize(token:Self::Token<'_>) -> Self::RustType {
            let tuple =  <UnderlyingSolTuple<'_>as alloy_sol_types::SolType> ::detokenize(token);
            <Self as ::core::convert::From<UnderlyingRustTuple<'_>> > ::from(tuple)
        }

        }
    #[automatically_derived]
    impl alloy_sol_types::SolStruct for KadenaMint {
        const NAME: &'static str = "KadenaMint";
        #[inline]
        fn eip712_root_type() -> alloy_sol_types::private::Cow<'static,str>{
            alloy_sol_types::private::Cow::Borrowed("KadenaMint(uint128 mintAmount,address targetAddress)")
        }
        #[inline]
        fn eip712_components() -> alloy_sol_types::private::Vec<alloy_sol_types::private::Cow<'static,str>>{
            alloy_sol_types::private::Vec::new()
        }
        #[inline]
        fn eip712_encode_type() -> alloy_sol_types::private::Cow<'static,str>{
            <Self as alloy_sol_types::SolStruct> ::eip712_root_type()
        }
        #[inline]
        fn eip712_encode_data(&self) -> alloy_sol_types::private::Vec<u8>{
            [< ::alloy_sol_types::sol_data::Uint<128>as alloy_sol_types::SolType> ::eip712_data_word(&self.mintAmount).0, < ::alloy_sol_types::sol_data::Address as alloy_sol_types::SolType> ::eip712_data_word(&self.targetAddress).0,].concat()
        }

        }
    #[automatically_derived]
    impl alloy_sol_types::EventTopic for KadenaMint {
        #[inline]
        fn topic_preimage_length(rust: &Self::RustType) -> usize {
            0usize+ < ::alloy_sol_types::sol_data::Uint<128>as alloy_sol_types::EventTopic> ::topic_preimage_length(&rust.mintAmount)+ < ::alloy_sol_types::sol_data::Address as alloy_sol_types::EventTopic> ::topic_preimage_length(&rust.targetAddress)
        }
        #[inline]
        fn encode_topic_preimage(rust: &Self::RustType,out: &mut alloy_sol_types::private::Vec<u8>){
            out.reserve(<Self as alloy_sol_types::EventTopic> ::topic_preimage_length(rust));
            < ::alloy_sol_types::sol_data::Uint<128>as alloy_sol_types::EventTopic> ::encode_topic_preimage(&rust.mintAmount,out);
            < ::alloy_sol_types::sol_data::Address as alloy_sol_types::EventTopic> ::encode_topic_preimage(&rust.targetAddress,out);
        }
        #[inline]
        fn encode_topic(rust: &Self::RustType) -> alloy_sol_types::abi::token::WordToken {
            let mut out = alloy_sol_types::private::Vec::new();
            <Self as alloy_sol_types::EventTopic> ::encode_topic_preimage(rust, &mut out);
            alloy_sol_types::abi::token::WordToken(alloy_sol_types::private::keccak256(out))
        }

        }

    };

impl Into<(Address, u128)> for KadenaMint {
    fn into(self) -> (Address, u128) {
        (self.targetAddress, self.mintAmount)
    }
}

pub struct KadenaMintPrecompile {
    pub mints: Arc<RwLock<Vec<KadenaMint>>>,
}

impl KadenaMintPrecompile {
    pub fn new(m: Arc<RwLock<Vec<KadenaMint>>>) -> Self {
        Self {
            mints: m
        }
    }

    pub fn call_mint(&mut self, bytes: &Bytes, _gas_limit: u64) -> PrecompileResult {
        let mint = <KadenaMint as SolValue>::abi_decode(bytes)
            .map_err(|_|  PrecompileError::Other("Failed to decode KadenaMint".to_string()))?;

        self.mints.write().unwrap().push(mint);
        Ok(PrecompileOutput::new(0, Bytes::default()))
    }
}

