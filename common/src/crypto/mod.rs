//! Utils for signing zksync transactions.
//! This crate is compiled into wasm to be used in `zksync.js`.

pub mod signature;
mod utils;
pub mod zklink_private_key;

use crate::crypto::utils::set_panic_hook;
use crate::crypto::zklink_private_key::privkey_from_slice;
pub use franklin_crypto::bellman::pairing::bn256::{Bn256 as Engine, Fr};
use franklin_crypto::rescue::bn256::Bn256RescueParams;
use franklin_crypto::{
    alt_babyjubjub::{edwards, fs::FsRepr, AltJubjubBn256, FixedGenerators},
    bellman::pairing::ff::{PrimeField, PrimeFieldRepr},
    eddsa::{PublicKey, Seed, Signature as EddsaSignature},
    jubjub::JubjubEngine,
};
use wasm_bindgen::prelude::*;

const PACKED_POINT_SIZE: usize = 32;
const PACKED_SIGNATURE_SIZE: usize = 64;

pub type Fs = <Engine as JubjubEngine>::Fs;
pub type Signature = EddsaSignature<Engine>;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

thread_local! {
    pub static JUBJUB_PARAMS: AltJubjubBn256 = AltJubjubBn256::new();
    pub static RESCUE_PARAMS: Bn256RescueParams = Bn256RescueParams::new_checked_2_into_1();
}

/// This method initializes params for current thread, otherwise they will be initialized when signing
/// first message.
#[wasm_bindgen(start)]
pub fn zksync_crypto_init() {
    JUBJUB_PARAMS.with(|_| {});
    RESCUE_PARAMS.with(|_| {});
    set_panic_hook();
}

/// get the public key hash from public key
#[wasm_bindgen(js_name = pubKeyHash)]
pub fn pub_key_hash(pubkey: &[u8]) -> Result<Vec<u8>, JsValue> {
    let pubkey = JUBJUB_PARAMS
        .with(|params| PublicKey::read(pubkey, params))
        .map_err(|_| JsValue::from_str("couldn't read public key"))?;
    Ok(utils::pub_key_hash(&pubkey))
}

#[wasm_bindgen(js_name = "rescueHash")]
pub fn rescue_hash_tx_msg(msg: &[u8]) -> Vec<u8> {
    utils::rescue_hash_tx_msg(msg)
}

/// `msg` should be represented by 2 concatenated
/// serialized orders of the swap transaction
#[wasm_bindgen(js_name = "rescueHashOrders")]
pub fn rescue_hash_orders(msg: &[u8]) -> Vec<u8> {
    utils::rescue_hash_orders(msg)
}

/// We use musig Schnorr signature scheme.
/// It is impossible to restore signer for signature, that is why we provide public key of the signer
/// along with signature.
/// [0..32] - packed public key of signer.
/// [32..64] - packed r point of the signature.
/// [64..96] - s poing of the signature.
#[wasm_bindgen]
pub fn sign_musig(private_key: &[u8], msg: &[u8]) -> Result<Vec<u8>, JsValue> {
    let mut packed_full_signature = Vec::with_capacity(PACKED_POINT_SIZE + PACKED_SIGNATURE_SIZE);
    let p_g = FixedGenerators::SpendingKeyGenerator;
    let private_key = privkey_from_slice(private_key)?;

    {
        let public_key =
            JUBJUB_PARAMS.with(|params| PublicKey::from_private(&private_key, p_g, params));
        public_key
            .write(&mut packed_full_signature)
            .expect("failed to write pubkey to packed_point");
    };

    let signature = JUBJUB_PARAMS.with(|jubjub_params| {
        RESCUE_PARAMS.with(|rescue_params| {
            let hashed_msg = utils::rescue_hash_tx_msg(msg);
            let seed = Seed::deterministic_seed(&private_key, &hashed_msg);
            private_key.musig_rescue_sign(&hashed_msg, &seed, p_g, rescue_params, jubjub_params)
        })
    });

    signature
        .r
        .write(&mut packed_full_signature)
        .expect("failed to write signature");
    signature
        .s
        .into_repr()
        .write_le(&mut packed_full_signature)
        .expect("failed to write signature repr");

    assert_eq!(
        packed_full_signature.len(),
        PACKED_POINT_SIZE + PACKED_SIGNATURE_SIZE,
        "incorrect signature size when signing"
    );

    Ok(packed_full_signature)
}

#[wasm_bindgen]
pub fn verify_musig(msg: &[u8], signature: &[u8]) -> Result<bool, JsValue> {
    if signature.len() != PACKED_POINT_SIZE + PACKED_SIGNATURE_SIZE {
        return Err(JsValue::from_str("Signature length is not 96 bytes. Make sure it contains both the public key and the signature itself."));
    }

    let pubkey = &signature[..PACKED_POINT_SIZE];
    let pubkey = JUBJUB_PARAMS
        .with(|params| edwards::Point::read(&*pubkey, params).map(PublicKey))
        .map_err(|_| JsValue::from_str("couldn't read public key"))?;

    let signature = deserialize_signature(&signature[PACKED_POINT_SIZE..])?;

    let msg = utils::rescue_hash_tx_msg(msg);
    let value = JUBJUB_PARAMS.with(|jubjub_params| {
        RESCUE_PARAMS.with(|rescue_params| {
            pubkey.verify_musig_rescue(
                &msg,
                &signature,
                FixedGenerators::SpendingKeyGenerator,
                rescue_params,
                jubjub_params,
            )
        })
    });

    Ok(value)
}

fn deserialize_signature(bytes: &[u8]) -> Result<Signature, JsValue> {
    let (r_bar, s_bar) = bytes.split_at(PACKED_POINT_SIZE);

    let r = JUBJUB_PARAMS
        .with(|params| edwards::Point::read(r_bar, params))
        .map_err(|_| JsValue::from_str("Failed to parse signature"))?;

    let mut s_repr = FsRepr::default();
    s_repr
        .read_le(s_bar)
        .map_err(|_| JsValue::from_str("Failed to parse signature"))?;

    let s = <Engine as JubjubEngine>::Fs::from_repr(s_repr)
        .map_err(|_| JsValue::from_str("Failed to parse signature"))?;

    Ok(Signature { r, s })
}
