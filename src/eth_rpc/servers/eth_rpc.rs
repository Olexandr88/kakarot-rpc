use std::sync::Arc;

use crate::starknet_client::errors::EthApiError;
use crate::{eth_provider::provider::EthereumProvider, starknet_client::KakarotClient};
use jsonrpsee::core::{async_trait, RpcResult as Result};
use reth_primitives::{AccessListWithGasUsed, Address, BlockId, BlockNumberOrTag, Bytes, H256, H64, U128, U256, U64};
use reth_rpc_types::{
    CallRequest, EIP1186AccountProofResponse, FeeHistory, Filter, FilterChanges, Index, RichBlock, SyncStatus,
    Transaction as EtherTransaction, TransactionReceipt, TransactionRequest, Work,
};
use serde_json::Value;
use starknet::providers::Provider;

use crate::eth_rpc::api::eth_api::EthApiServer;

/// The RPC module for the Ethereum protocol required by Kakarot.
pub struct KakarotEthRpc<P, SP>
where
    P: EthereumProvider,
    SP: Provider + Send + Sync,
{
    pub eth_provider: P,
    // TODO remove kakaort_client from here
    pub kakarot_client: Arc<KakarotClient<SP>>,
}

impl<P, SP> KakarotEthRpc<P, SP>
where
    P: EthereumProvider,
    SP: Provider + Send + Sync,
{
    pub fn new(eth_provider: P, kakarot_client: Arc<KakarotClient<SP>>) -> Self {
        Self { eth_provider, kakarot_client }
    }
}

#[async_trait]
impl<P, SP> EthApiServer for KakarotEthRpc<P, SP>
where
    P: EthereumProvider + Send + Sync + 'static,
    SP: Provider + Send + Sync + 'static,
{
    #[tracing::instrument(skip_all, ret, err)]
    async fn block_number(&self) -> Result<U64> {
        Ok(self.eth_provider.block_number().await?)
    }

    #[tracing::instrument(skip_all, ret, err)]
    async fn syncing(&self) -> Result<SyncStatus> {
        Ok(self.syncing().await?)
    }

    async fn coinbase(&self) -> Result<Address> {
        Err(EthApiError::MethodNotSupported("eth_coinbase".to_string()).into())
    }

    #[tracing::instrument(skip_all, ret, err)]
    async fn accounts(&self) -> Result<Vec<Address>> {
        Ok(Vec::new())
    }

    #[tracing::instrument(skip_all, ret, err)]
    async fn chain_id(&self) -> Result<Option<U64>> {
        Ok(self.eth_provider.chain_id().await?)
    }

    #[tracing::instrument(skip_all, ret, err, fields(hash = %hash))]
    async fn block_by_hash(&self, hash: H256, full: bool) -> Result<Option<RichBlock>> {
        Ok(self.eth_provider.block_by_hash(hash, full).await?)
    }

    #[tracing::instrument(skip_all, ret, err, fields(number = %number, full = full))]
    async fn block_by_number(&self, number: BlockNumberOrTag, full: bool) -> Result<Option<RichBlock>> {
        Ok(self.eth_provider.block_by_number(number, full).await?)
    }

    #[tracing::instrument(skip_all, ret, err, fields(hash = %hash))]
    async fn block_transaction_count_by_hash(&self, hash: H256) -> Result<U64> {
        Ok(self.eth_provider.block_transaction_count_by_hash(hash).await?)
    }

    #[tracing::instrument(skip_all, ret, err, fields(number = %number))]
    async fn block_transaction_count_by_number(&self, number: BlockNumberOrTag) -> Result<U64> {
        Ok(self.eth_provider.block_transaction_count_by_number(number).await?)
    }

    #[tracing::instrument(skip_all, ret, err, fields(hash = %_hash))]
    async fn block_uncles_count_by_block_hash(&self, _hash: H256) -> Result<U256> {
        tracing::warn!("Kakarot chain does not produce uncles");
        Ok(U256::ZERO)
    }

    #[tracing::instrument(skip_all, ret, err, fields(number = %_number))]
    async fn block_uncles_count_by_block_number(&self, _number: BlockNumberOrTag) -> Result<U256> {
        tracing::warn!("Kakarot chain does not produce uncles");
        Ok(U256::ZERO)
    }

    #[tracing::instrument(skip_all, ret, err, fields(hash = %_hash, index = ?_index))]
    async fn uncle_by_block_hash_and_index(&self, _hash: H256, _index: Index) -> Result<Option<RichBlock>> {
        tracing::warn!("Kakarot chain does not produce uncles");
        Ok(None)
    }

    #[tracing::instrument(skip_all, ret, err, fields(hash = %_number, index = ?_index))]
    async fn uncle_by_block_number_and_index(
        &self,
        _number: BlockNumberOrTag,
        _index: Index,
    ) -> Result<Option<RichBlock>> {
        tracing::warn!("Kakarot chain does not produce uncles");
        Ok(None)
    }

    #[tracing::instrument(skip_all, ret, err, fields(hash = %hash))]
    async fn transaction_by_hash(&self, hash: H256) -> Result<Option<EtherTransaction>> {
        Ok(self.eth_provider.transaction_by_hash(hash).await?)
    }

    #[tracing::instrument(skip_all, ret, err, fields(hash = %hash, index = ?index))]
    async fn transaction_by_block_hash_and_index(&self, hash: H256, index: Index) -> Result<Option<EtherTransaction>> {
        Ok(self.eth_provider.transaction_by_block_hash_and_index(hash, index).await?)
    }

    #[tracing::instrument(skip_all, ret, err, fields(number = %number, index = ?index))]
    async fn transaction_by_block_number_and_index(
        &self,
        number: BlockNumberOrTag,
        index: Index,
    ) -> Result<Option<EtherTransaction>> {
        Ok(self.eth_provider.transaction_by_block_number_and_index(number, index).await?)
    }

    #[tracing::instrument(skip_all, ret, err, fields(hash = %hash))]
    async fn transaction_receipt(&self, hash: H256) -> Result<Option<TransactionReceipt>> {
        Ok(self.eth_provider.transaction_receipt(hash).await?)
    }

    #[tracing::instrument(skip_all, ret, err, fields(address = %address, block_id = ?block_id))]
    async fn balance(&self, address: Address, block_id: Option<BlockId>) -> Result<U256> {
        Ok(self.eth_provider.balance(address, block_id).await?)
    }

    #[tracing::instrument(skip_all, ret, err, fields(address = %address, index = ?index, block_id = ?block_id))]
    async fn storage_at(&self, address: Address, index: U256, block_id: Option<BlockId>) -> Result<U256> {
        Ok(self.eth_provider.storage_at(address, index, block_id).await?)
    }

    #[tracing::instrument(skip_all, ret, err, fields(address = %address, block_id = ?block_id))]
    async fn transaction_count(&self, address: Address, block_id: Option<BlockId>) -> Result<U256> {
        Ok(self.eth_provider.transaction_count(address, block_id).await?)
    }

    #[tracing::instrument(skip_all, ret, err, fields(address = %address, block_id = ?block_id))]
    async fn get_code(&self, address: Address, block_id: Option<BlockId>) -> Result<Bytes> {
        Ok(self.eth_provider.get_code(address, block_id).await?)
    }

    #[tracing::instrument(skip_all, ret, err, fields(filter = ?filter))]
    async fn get_logs(&self, filter: Filter) -> Result<FilterChanges> {
        Ok(self.eth_provider.get_logs(filter).await?)
    }

    #[tracing::instrument(skip_all, ret, err, fields(request = ?request, block_id = ?block_id))]
    async fn call(&self, request: CallRequest, block_id: Option<BlockId>) -> Result<Bytes> {
        let block_id = block_id.unwrap_or(BlockId::Number(BlockNumberOrTag::Latest));
        let result = self.kakarot_client.call(request, block_id).await?;

        Ok(result)
    }

    async fn create_access_list(
        &self,
        _request: CallRequest,
        _block_id: Option<BlockId>,
    ) -> Result<AccessListWithGasUsed> {
        Err(EthApiError::MethodNotSupported("eth_createAccessList".to_string()).into())
    }

    #[tracing::instrument(skip_all, ret, fields(request = ?request, block_id = ?block_id))]
    async fn estimate_gas(&self, request: CallRequest, block_id: Option<BlockId>) -> Result<U256> {
        let block_id = block_id.unwrap_or(BlockId::Number(BlockNumberOrTag::Latest));

        Ok(self.kakarot_client.estimate_gas(request, block_id).await?)
    }

    #[tracing::instrument(skip_all, ret, err)]
    async fn gas_price(&self) -> Result<U256> {
        let gas_price = self.kakarot_client.base_fee_per_gas();
        Ok(gas_price)
    }

    #[tracing::instrument(skip_all, ret, err, fields(block_count = %block_count, newest_block = %newest_block, reward_percentiles = ?reward_percentiles))]
    async fn fee_history(
        &self,
        block_count: U256,
        newest_block: BlockNumberOrTag,
        reward_percentiles: Option<Vec<f64>>,
    ) -> Result<FeeHistory> {
        let fee_history = self.kakarot_client.fee_history(block_count, newest_block, reward_percentiles).await?;

        Ok(fee_history)
    }

    #[tracing::instrument(skip_all, ret, err)]
    async fn max_priority_fee_per_gas(&self) -> Result<U128> {
        let max_priority_fee = self.kakarot_client.max_priority_fee_per_gas();
        Ok(max_priority_fee)
    }

    async fn mining(&self) -> Result<bool> {
        Err(EthApiError::MethodNotSupported("eth_mining".to_string()).into())
    }

    async fn hashrate(&self) -> Result<U256> {
        Err(EthApiError::MethodNotSupported("eth_hashrate".to_string()).into())
    }

    async fn get_work(&self) -> Result<Work> {
        Err(EthApiError::MethodNotSupported("eth_getWork".to_string()).into())
    }

    async fn submit_hashrate(&self, _hashrate: U256, _id: H256) -> Result<bool> {
        Err(EthApiError::MethodNotSupported("eth_submitHashrate".to_string()).into())
    }

    async fn submit_work(&self, _nonce: H64, _pow_hash: H256, _mix_digest: H256) -> Result<bool> {
        Err(EthApiError::MethodNotSupported("eth_submitWork".to_string()).into())
    }

    async fn send_transaction(&self, _request: TransactionRequest) -> Result<H256> {
        Err(EthApiError::MethodNotSupported("eth_sendTransaction".to_string()).into())
    }

    #[tracing::instrument(skip_all, ret, err, fields(bytes = %bytes))]
    async fn send_raw_transaction(&self, bytes: Bytes) -> Result<H256> {
        let transaction_hash = self.kakarot_client.send_transaction(bytes).await?;
        Ok(transaction_hash)
    }

    async fn sign(&self, _address: Address, _message: Bytes) -> Result<Bytes> {
        Err(EthApiError::MethodNotSupported("eth_sign".to_string()).into())
    }

    async fn sign_transaction(&self, _transaction: CallRequest) -> Result<Bytes> {
        Err(EthApiError::MethodNotSupported("eth_signTransaction".to_string()).into())
    }

    async fn sign_typed_data(&self, _address: Address, _data: Value) -> Result<Bytes> {
        Err(EthApiError::MethodNotSupported("eth_signTypedData".to_string()).into())
    }

    async fn get_proof(
        &self,
        _address: Address,
        _keys: Vec<H256>,
        _block_id: Option<BlockId>,
    ) -> Result<EIP1186AccountProofResponse> {
        Err(EthApiError::MethodNotSupported("eth_getProof".to_string()).into())
    }

    async fn new_filter(&self, _filter: Filter) -> Result<U64> {
        Err(EthApiError::MethodNotSupported("eth_newFilter".to_string()).into())
    }

    async fn new_block_filter(&self) -> Result<U64> {
        Err(EthApiError::MethodNotSupported("eth_newBlockFilter".to_string()).into())
    }

    async fn new_pending_transaction_filter(&self) -> Result<U64> {
        Err(EthApiError::MethodNotSupported("eth_newPendingTransactionFilter".to_string()).into())
    }

    async fn uninstall_filter(&self, _id: U64) -> Result<bool> {
        Err(EthApiError::MethodNotSupported("eth_uninstallFilter".to_string()).into())
    }

    async fn get_filter_changes(&self, _id: U64) -> Result<FilterChanges> {
        Err(EthApiError::MethodNotSupported("eth_getFilterChanges".to_string()).into())
    }

    async fn get_filter_logs(&self, _id: U64) -> Result<FilterChanges> {
        Err(EthApiError::MethodNotSupported("eth_getFilterLogs".to_string()).into())
    }
}