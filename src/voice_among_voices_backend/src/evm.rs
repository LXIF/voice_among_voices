use std::str::FromStr;

use crate::siwe_principal;
use alloy::{
    network::TransactionBuilder,
    primitives::{Address, Uint},
    providers::{Provider, ProviderBuilder, RootProvider},
    rpc::types::TransactionRequest,
    sol,
    sol_types::SolCall,
    transports::icp::{EthSepoliaService, IcpConfig, IcpTransport, RpcService},
};
use ic_cdk::{call, caller};
use serde_bytes::ByteBuf;
use IERC721::balanceOfReturn;

// Define relevant ERC721 interface
sol! {
    #[sol(rpc)]
    contract IERC721 {
        function balanceOf(address owner) external view returns (uint256);
        function tokenOfOwnerByIndex(address owner, uint256 index) external view returns (uint256);
        function ownerTokens(address owner) external view returns (uint256[]);
    }
}

pub async fn get_caller_wallet_address() -> Result<String, String> {
    let (result,): (Result<String, String>,) = call(
        siwe_principal(),
        "get_address",
        (ByteBuf::from(caller().as_slice().to_vec()),),
    )
    .await
    .map_err(|(code, msg)| format!("Error code: {:?}, message: {}", code, msg))?;
    result
}

pub async fn get_caller_owned_tokens() -> Result<Vec<Uint<256, 4>>, String> {
    let caller_address = get_caller_wallet_address().await?;
    let provider = setup_evm_provider();

    let owner =
        Address::from_str(&caller_address).map_err(|e| format!("Invalid address: {}", e))?;

    let contract_address =
        Address::from_str("CONTRACT ADDRESS") //TODO
            .map_err(|e| format!("Failed to parse contract address: {}", e))?;

    let token_contract = IERC721::new(contract_address, provider);

    let call_builder = token_contract.ownerTokens(owner);

    let call_response = call_builder
        .call()
        .await
        .map_err(|e| format!("Failed to call ownerTokens: {}", e))?;

    Ok(call_response._0)
}

pub async fn get_caller_balance() -> Result<Uint<256, 4>, String> {
    let caller_address = get_caller_wallet_address().await?;
    let provider = setup_evm_provider();

    let owner =
        Address::from_str(&caller_address).map_err(|e| format!("Invalid address: {}", e))?;

    let contract_address =
        Address::from_str("CONTRACT ADDRESS") //TODO
            .map_err(|e| format!("Failed to parse contract address: {}", e))?;

    let token_contract = IERC721::new(contract_address, provider);

    let call_builder = token_contract.balanceOf(owner);

    let call_response = call_builder
        .call()
        .await
        .map_err(|e| format!("Failed to call balanceOf: {}", e))?;

    Ok(call_response._0)
}

fn setup_evm_provider() -> RootProvider<IcpTransport> {
    let rpc_service = RpcService::EthSepolia(EthSepoliaService::Alchemy);
    let config = IcpConfig::new(rpc_service);
    ProviderBuilder::new().on_icp(config)
}
