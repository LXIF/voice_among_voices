use std::str::FromStr;

use crate::structs::AuthorizationError;
use crate::token_address;
use crate::{siwe_principal, storage::dev_mode};
use alloy::{
    primitives::{Address, Uint},
    providers::{ProviderBuilder, RootProvider},
    sol,
    transports::icp::{IcpConfig, IcpTransport, L2MainnetService, RpcService},
};
use candid::CandidType;
use ic_cdk::api::call::CallResult;
use ic_cdk::{api::caller, call};
use ic_stable_structures::{storable::Bound, Storable};
use serde::Deserialize;
use IERC721::IERC721Instance;

// Define relevant ERC721 interface
sol! {
    #[sol(rpc)]
    contract IERC721 {
        // function balanceOf(address owner) external view returns (uint256);
        // function tokenOfOwnerByIndex(address owner, uint256 index) external view returns (uint256);
        function ownerOf(uint256 tokenId) external view returns (address);
    }
}

#[derive(CandidType, Debug, Deserialize)]
enum GetAddressResponse {
    Ok(String),
    Err(String),
}

pub async fn get_caller_wallet_address() -> Result<String, String> {
    let result = call::<_, (GetAddressResponse,)>(
        siwe_principal(),
        "get_address",
        (caller().to_bytes().into_owned(),),
    )
    .await
    .map_err(|(_code, err)| err)?;

    ic_cdk::println!("result: {:?}", result);

    match result.0 {
        GetAddressResponse::Ok(address) => Ok(address),
        GetAddressResponse::Err(err) => Err(err),
    }
}

// async fn retry<F, Fut, T>(mut f: F, max_retries: u32) -> Result<T, String>
// where
//     F: FnMut() -> Fut,
//     Fut: Future<Output = Result<T, String>>,
// {
//     let mut attempt = 0;

//     loop {
//         match f().await {
//             Ok(result) => return Ok(result),
//             Err(e) => {
//                 attempt += 1;
//                 if attempt >= max_retries {
//                     return Err(e);
//                 }
//             }
//         }
//     }
// }

// TODO: this function is affected by one-after issue
async fn caller_is_owner_of(token_id: u64) -> Result<Address, AuthorizationError> {
    let CallObjects {
        owner,
        token_contract,
    } = setup_call_objects()
        .await
        .map_err(AuthorizationError::EvmError)?;

    // let call_response = retry(
    //     || async {
    //         token_contract
    //             .ownerOf(Uint::from(token_id))
    //             .call()
    //             .await
    //             .map_err(|e| format!("Failed to call ownerOf: {}", e))
    //     },
    //     3,
    // )
    // .await?;

    let call_response = token_contract
        .ownerOf(Uint::from(token_id))
        .call()
        .await
        .map_err(|e| AuthorizationError::EvmError(format!("Failed to call ownerOf: {}", e)))?;

    if call_response._0 == owner {
        Ok(owner)
    } else {
        Err(AuthorizationError::Unauthorized)
    }
}

pub async fn check_auth_for_single_node_id(node_id: usize) -> Result<Address, AuthorizationError> {
    if dev_mode() {
        return Ok(Address::ZERO);
    };
    caller_is_owner_of(node_id as u64).await
}

struct CallObjects {
    owner: Address,
    token_contract: IERC721Instance<IcpTransport, RootProvider<IcpTransport>>,
}

async fn setup_call_objects() -> Result<CallObjects, String> {
    let caller_address = get_caller_wallet_address().await?;
    let provider = setup_evm_provider();
    let contract_address = token_address();

    let owner =
        Address::from_str(&caller_address).map_err(|e| format!("Invalid address: {}", e))?;
    let token_contract = IERC721::new(contract_address, provider);
    Ok(CallObjects {
        owner,
        token_contract,
    })
}

fn setup_evm_provider() -> RootProvider<IcpTransport> {
    //TODO: change for mainnet (or put in config)
    let rpc_service = RpcService::BaseMainnet(L2MainnetService::PublicNode);
    let config = IcpConfig::new(rpc_service);
    ProviderBuilder::new().on_icp(config)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StorableAddress(pub Address);

impl Storable for StorableAddress {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        self.0.to_bytes()
    }
    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        StorableAddress(Address::from_slice(bytes.as_ref()))
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: 20,
        is_fixed_size: true,
    };
}
