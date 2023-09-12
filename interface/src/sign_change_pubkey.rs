use crate::error::SignError;
use crate::{ChangePubKeyAuthRequest, TxSignature};
#[cfg(feature = "ffi")]
use std::sync::Arc;
use zklink_signers::eth_signer::packed_eth_signature::PackedEthSignature;
use zklink_signers::eth_signer::pk_signer::PrivateKeySigner;
use zklink_signers::zklink_signer::pk_signer::{sha256_bytes, ZkLinkSigner};
use zklink_signers::zklink_signer::signature::ZkLinkSignature;
use zklink_types::basic_types::ZkLinkAddress;
use zklink_types::tx_type::change_pubkey::{ChangePubKey, ChangePubKeyAuthData, Create2Data};

#[cfg(feature = "sync")]
pub fn sign_change_pubkey(
    eth_signer: &PrivateKeySigner,
    zklink_singer: &ZkLinkSigner,
    mut tx: ChangePubKey,
    main_contract: ZkLinkAddress,
    l1_client_id: u32,
    account_address: ZkLinkAddress,
    auth_request: ChangePubKeyAuthRequest,
) -> Result<TxSignature, SignError> {
    let eth_auth_data: Result<ChangePubKeyAuthData, _> = match auth_request {
        ChangePubKeyAuthRequest::OnChain => Ok(ChangePubKeyAuthData::OnChain),
        ChangePubKeyAuthRequest::EthECDSA => {
            let typed_data = tx.to_eip712_request_payload(l1_client_id, &main_contract)?;
            let eth_signature = eth_signer.sign_typed_data(&typed_data)?;
            Ok(ChangePubKeyAuthData::EthECDSA { eth_signature })
        }
        ChangePubKeyAuthRequest::EthCreate2 { data } => {
            // check create2 data
            let pubkey_hash = zklink_singer.public_key().public_key_hash();
            let from_address = data.get_address(pubkey_hash.data.to_vec());
            if from_address.as_bytes() != account_address.as_bytes() {
                Err(SignError::IncorrectTx)
            } else {
                Ok(ChangePubKeyAuthData::EthCreate2 { data })
            }
        }
    };
    tx.eth_auth_data = eth_auth_data?;
    tx.signature = zklink_singer.sign_musig(&tx.get_bytes())?;
    Ok(TxSignature {
        tx: tx.into(),
        eth_signature: None,
    })
}

#[cfg(feature = "ffi")]
pub fn eth_signature_of_change_pubkey(
    l1_client_id: u32,
    tx: Arc<ChangePubKey>,
    eth_signer: Arc<PrivateKeySigner>,
    main_contract: ZkLinkAddress,
) -> Result<PackedEthSignature, SignError>{
    let typed_data = tx.to_eip712_request_payload(l1_client_id, &main_contract)?;
    let eth_signature = eth_signer.sign_byted_data(&typed_data.data_hash)?;
    Ok(eth_signature)
}


#[cfg(feature = "ffi")]
pub fn create_submitter_signature(
    tx_bytes: &[u8],
    zklink_signer: Arc<ZkLinkSigner>,
) -> Result<ZkLinkSignature, SignError> {
    let bytes_sha_256 = sha256_bytes(&tx_bytes);
    let signature = zklink_signer.sign_musig(&bytes_sha_256)?;
    Ok(signature)
}

#[cfg(feature = "ffi")]
pub fn check_create2data(zklink_singer: Arc<ZkLinkSigner>, data: Create2Data, account_address: ZkLinkAddress) -> Result<(), SignError> {
    let pubkey_hash = zklink_singer.public_key().public_key_hash();
    let from_address = data.get_address(pubkey_hash.data.to_vec());
    if from_address.as_bytes() != account_address.as_bytes() {
        Err(SignError::IncorrectTx)
    } else {
        Ok(())
    }
}


#[cfg(feature = "ffi")]
pub fn create_signed_change_pubkey(
    zklink_singer: Arc<ZkLinkSigner>,
    tx: Arc<ChangePubKey>,
    eth_auth_data: ChangePubKeyAuthData,
) -> Result<Arc<ChangePubKey>, SignError> {
    let mut tx = (*tx).clone();
    tx.eth_auth_data = eth_auth_data;
    tx.signature = zklink_singer.sign_musig(&tx.get_bytes())?;
    Ok(Arc::new(tx))
}
