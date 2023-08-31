/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
mod crypto;
use crate::crypto::{get_public_key, get_public_key_hash, verify_musig};

use std::str::FromStr;

use zklink_crypto::zklink_signer::error::ZkSignerError;
use zklink_crypto::zklink_signer::pk_signer::ZkLinkSigner;
use zklink_crypto::zklink_signer::private_key::PackedPrivateKey;
use zklink_crypto::zklink_signer::pubkey_hash::PubKeyHash;
use zklink_crypto::zklink_signer::public_key::PackedPublicKey;
use zklink_crypto::zklink_signer::signature::{PackedSignature, ZkLinkSignature};

use zklink_types::basic_types::error::TypeError;
use zklink_types::basic_types::tx_hash::TxHash;
use zklink_types::basic_types::zklink_address::ZkLinkAddress;
use zklink_types::basic_types::{
    AccountId, BigUint, BlockNumber, ChainId, EthBlockId, Nonce, PairId, PriorityOpId, SlotId,
    SubAccountId, TimeStamp, TokenId, H256,
};
use zklink_types::tx_type::deposit::Deposit;

macro_rules! ffi_convert {
    ($(#[$attr:meta])* $name:ident, $type:ty) => {
        impl UniffiCustomTypeConverter for $name {
            type Builtin = $type;
            fn into_custom(val: Self::Builtin) -> uniffi::Result<Self> {
                Ok($name(val))
            }
            fn from_custom(obj: Self) -> Self::Builtin {
                obj.0
            }
        }
    };
}

ffi_convert!(SlotId, u32);
ffi_convert!(TokenId, u32);
ffi_convert!(PairId, u16);
ffi_convert!(TimeStamp, u32);
ffi_convert!(AccountId, u32);
ffi_convert!(BlockNumber, u32);
ffi_convert!(Nonce, u32);
ffi_convert!(PriorityOpId, u64);
ffi_convert!(EthBlockId, u64);
ffi_convert!(ChainId, u8);
ffi_convert!(SubAccountId, u8);

macro_rules! ffi_str_convert {
    ($(#[$attr:meta])* $name:ident) => {
        impl UniffiCustomTypeConverter for $name {
            type Builtin = String;
            fn into_custom(val: Self::Builtin) -> uniffi::Result<Self> {
                let s = $name::from_str(&val)?;
                Ok(s)
            }
            fn from_custom(obj: Self) -> Self::Builtin {
                obj.to_string()
            }
        }
    };
}

ffi_str_convert!(BigUint);
ffi_str_convert!(ZkLinkAddress);

macro_rules! ffi_hex_convert {
    ($(#[$attr:meta])* $name:ident) => {
        impl UniffiCustomTypeConverter for $name {
            type Builtin = String;
            fn into_custom(val: Self::Builtin) -> uniffi::Result<Self> {
                let s = $name::from_hex(&val)?;
                Ok(s)
            }
            fn from_custom(obj: Self) -> Self::Builtin {
                obj.as_hex()
            }
        }
    };
}
ffi_hex_convert!(TxHash);
ffi_hex_convert!(PackedPublicKey);
ffi_hex_convert!(PackedSignature);
ffi_hex_convert!(PubKeyHash);

impl UniffiCustomTypeConverter for H256 {
    type Builtin = String;
    fn into_custom(val: String) -> uniffi::Result<Self> {
        let s = val.as_str().strip_prefix("0x").unwrap_or(&val);
        let raw = hex::decode(s)?;
        if raw.len() != 32 {
            return Err(TypeError::SizeMismatch.into());
        }
        let h = H256::from_slice(&raw);
        Ok(h)
    }

    fn from_custom(obj: Self) -> Self::Builtin {
        let s = hex::encode(obj.as_bytes());
        format!("0x{s}")
    }
}

include!(concat!(env!("OUT_DIR"), "/ffi.uniffi.rs"));

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_convert() {
        let h = H256::zero();
        let s = H256::from_custom(h);
        println!("{s}");
        let h2 = H256::into_custom(s).unwrap();
        println!("{h2:?}");

        // test BigUnit
        let b = BigUint::default();
        let s = b.to_string();
        let b2 = BigUint::from_str("1234567890987654321").unwrap();
        println!("big uint: {:?}", s);
        println!("big uint: {:?}", b2.to_string());
    }
}
