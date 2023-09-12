use zklink_signers::zklink_signer::signature::ZkLinkSignature;
use crate::{TxSignature, UniffiCustomTypeConverter};

macro_rules! ffi_json_convert {
    ($(#[$attr:meta])* $name:ident) => {
        impl UniffiCustomTypeConverter for $name {
            type Builtin = String;
            fn into_custom(val: Self::Builtin) -> uniffi::Result<Self> {
                let s: $name = serde_json::from_str(&val)?;
                Ok(s)
            }
            fn from_custom(obj: Self) -> Self::Builtin {
                serde_json::to_string(&obj).expect("invalid string")
            }
        }
    };
}

ffi_json_convert!(TxSignature);

pub fn json_str_of_zklink_signature(signature: ZkLinkSignature) -> String {
    serde_json::to_string(&signature).unwrap()
}
