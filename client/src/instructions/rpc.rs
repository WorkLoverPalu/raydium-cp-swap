use anyhow::{anyhow, Result};
use solana_client::{
    rpc_client::RpcClient,
    rpc_config::RpcSendTransactionConfig,
    rpc_request::RpcRequest,
    rpc_response::{RpcResult, RpcSimulateTransactionResult},
};
use solana_sdk::{
    account::Account, commitment_config::CommitmentConfig, program_pack::Pack as TokenPack,
    pubkey::Pubkey, signature::Signature, transaction::Transaction,
};
use std::convert::Into;

// 交易模拟执行
pub fn simulate_transaction(
    client: &RpcClient,        // RPC客户端实例
    transaction: &Transaction, // 待模拟交易
    sig_verify: bool,          // 是否验证签名
    cfg: CommitmentConfig,     // 确认级别配置
) -> RpcResult<RpcSimulateTransactionResult> {
    let serialized_encoded = bs58::encode(bincode::serialize(transaction).unwrap()).into_string();
    client.send(
        RpcRequest::SimulateTransaction,
        serde_json::json!([serialized_encoded, {
            "sigVerify": sig_verify, "commitment": cfg.commitment
        }]),
    )
}

// 交易发送
pub fn send_txn(
    client: &RpcClient, // RPC客户端实例
    txn: &Transaction,  // 待发送交易
    wait_confirm: bool, // 是否等待确认
) -> Result<Signature> {
    Ok(client.send_and_confirm_transaction_with_spinner_and_config(
        txn,
        if wait_confirm {
            CommitmentConfig::confirmed()
        } else {
            CommitmentConfig::processed()
        },
        RpcSendTransactionConfig {
            skip_preflight: true,
            ..RpcSendTransactionConfig::default()
        },
    )?)
}

// 账户数据查询
pub fn get_token_account<T: TokenPack>(
    client: &RpcClient, // RPC客户端实例
    addr: &Pubkey,      // 账户地址
) -> Result<T> {
    let account = client
        .get_account_with_commitment(addr, CommitmentConfig::processed())?
        .value
        .map_or(Err(anyhow!("Account not found")), Ok)?;
    T::unpack_from_slice(&account.data).map_err(Into::into)
}

// 批量查询账户原始数据
pub fn get_multiple_accounts(
    client: &RpcClient, // RPC客户端实例
    pubkeys: &[Pubkey], // 账户地址列表
) -> Result<Vec<Option<Account>>> {
    Ok(client.get_multiple_accounts(pubkeys)?)
}
