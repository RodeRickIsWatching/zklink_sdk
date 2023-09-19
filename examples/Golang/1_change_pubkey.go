package main

import (
	sdk "github.com/zkLinkProtocol/zklink_sdk/go_example/generated/uniffi/zklink_sdk"
	"net/http"
	"math/big"
	"encoding/json"
	"fmt"
	"time"
	"bytes"
	"io/ioutil"
)


type RPCTransaction struct {
     Id      int64             `json:"id"`
     JsonRpc string            `json:"jsonrpc"`
     Method  string            `json:"method"`
     Params  []json.RawMessage `json:"params"`
}

func LowLevel() {
    privateKey := "0xbe725250b123a39dab5b7579334d5888987c72a58f4508062545fe6e08ca94f4"
    // create eth signer
    ethSigner, err := sdk.NewPrivateKeySigner(privateKey)
    if err != nil {
        return
    }

    // create zklink signer
	zklinkSigner, err := sdk.ZkLinkSignerNewFromHexEthSigner(privateKey)
	if err != nil {
		return
	}

	chainId := sdk.ChainId(1)
	accountId := sdk.AccountId(2)
	subAccountId := sdk.SubAccountId(4)
    newPkHash:= sdk.PubKeyHash("0xd8d5fb6a6caef06aa3dc2abdcdc240987e5330fe")
    feeToken := sdk.TokenId(1)
    fee := big.NewInt(100)
    nonce := sdk.Nonce(100)
    // TODO: create ethSignature
    ethSignature := sdk.PackedEthSignature("0x000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001b")
    // get current timestamp
    now := time.Now()
    timeStamp := sdk.TimeStamp(now.Unix())

    // create ChangePubKey transaction type without signed
	builder := sdk.ChangePubKeyBuilder{
		chainId,
		accountId,
		subAccountId,
		newPkHash,
		feeToken,
		*fee,
		nonce,
		&ethSignature,
		timeStamp,
	}
	tx := sdk.NewChangePubKey(builder)

	// create ethAuthData
	// AuthData has 3 types of enum
	// 1. sdk.ChangePubKeyAuthDataOnChain{}
	// 2. sdk.ChangePubKeyAuthDataEthCreate2 { Data: sdk.Create2Data }
	// 3. sdk.ChangePubKeyAuthDataEthEcdsa

	// TODO: use real main contract address
    main_contract := sdk.ZkLinkAddress("0x0000000000000000000000000000000000000000")
    l1_client_id := uint32(1)
    ethSignature, err = sdk.EthSignatureOfChangePubkey(l1_client_id, tx, ethSigner, main_contract);
    if err != nil {
        return
    }
    ethAuthData := sdk.ChangePubKeyAuthDataEthEcdsa {
        EthSignature: ethSignature,
    }

    // sign the changePubKey, add the ethAuthData
    tx, err = sdk.CreateSignedChangePubkey(zklinkSigner, tx, ethAuthData)
    if err != nil {
        return
    }
    // check if the signature is valid
    valid, err := tx.IsSignatureValid();
    if err != nil || !valid {
        fmt.Println("sign tx failed")
        return
    }
    zklinkTx := sdk.ZklinkTxFromChangePubkey(tx)
    fmt.Println("changePubKey tx: %v", zklinkTx)

    // create submitter signature
    txHash := tx.TxHash()
    submitter_signature, err := zklinkSigner.SignMusig(txHash)
    if err != nil {
        return
    }
	json_str_of_submitter_signature := sdk.JsonStrOfZklinkSignature(submitter_signature)
    fmt.Println("changePubKey submitter signature: %v", json_str_of_submitter_signature)

    // rpc request with `sendTransaction`
    // [a, b, c]
	tx_req := RPCTransaction {
		Id:      1,
		JsonRpc: "2.0",
		Method:  "sendTransaction",
		Params: []json.RawMessage{
		[]byte(zklinkTx),
		nil,
		[]byte(json_str_of_submitter_signature)},
    }
	JsonTx, err := json.Marshal(tx_req)
	fmt.Println("ChangePubKey rpc request:",  string(JsonTx))
	zklinkUrl := sdk.ZklinkTestNetUrl()
	response, err := http.Post(zklinkUrl, "application/json",bytes.NewBuffer(JsonTx))
	if err != nil {
        fmt.Println(err)
    }
    defer response.Body.Close()
    body, _ := ioutil.ReadAll(response.Body)
    fmt.Println(string(body))
}

func HighLevel() {
    privateKey := "0xbe725250b123a39dab5b7579334d5888987c72a58f4508062545fe6e08ca94f4"
	chainId := sdk.ChainId(1)
	accountId := sdk.AccountId(2)
	subAccountId := sdk.SubAccountId(4)
    newPkHash:= sdk.PubKeyHash("0xd8d5fb6a6caef06aa3dc2abdcdc240987e5330fe")
    feeToken := sdk.TokenId(1)
    fee := big.NewInt(100)
    nonce := sdk.Nonce(100)
    // TODO: create ethSignature
    ethSignature := sdk.PackedEthSignature("0x000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001b")
    // get current timestamp
    now := time.Now()
    timeStamp := sdk.TimeStamp(now.Unix())
	// TODO: use real main contract address
    main_contract := sdk.ZkLinkAddress("0x0000000000000000000000000000000000000000")

    // create ChangePubKey transaction type without signed
	changePubKeyBuilder := sdk.ChangePubKeyBuilder{
		chainId,
		accountId,
		subAccountId,
		newPkHash,
		feeToken,
		*fee,
		nonce,
		&ethSignature,
		timeStamp,
	}
    l1_client_id := uint32(1)
    params, err := sdk.BuildChangePubkeyRequestWithEthEcdsaAuthData(privateKey, changePubKeyBuilder, l1_client_id, main_contract)
    if err != nil {
        return
    }
    // rpc request with `sendTransaction`
    // [a, b, c] json string
    // [[a, b, c]]
    fmt.Println("xxxxxxxxx: %s", params)
    params_bytes := []byte(params)
	tx := RPCTransaction{
		Id:      1,
		JsonRpc: "2.0",
		Method:  "sendTransaction",
		Params: []json.RawMessage{params_bytes},
    }
	JsonTx, err := json.Marshal(tx)
	fmt.Println("ChangePubKey rpc request:",  string(JsonTx))
	zklinkUrl := sdk.ZklinkTestNetUrl()
	response, err := http.Post(zklinkUrl, "application/json", bytes.NewBuffer(JsonTx))
	if err != nil {
        fmt.Println(err)
    }
    defer response.Body.Close()
    body, _ := ioutil.ReadAll(response.Body)
    fmt.Println(string(body))
}

func main() {
    LowLevel()
    HighLevel()
}
