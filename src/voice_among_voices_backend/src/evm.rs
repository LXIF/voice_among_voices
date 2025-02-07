use std::str::FromStr;

use crate::token_address;
use crate::{siwe_principal, storage::dev_mode};
use alloy::transports::BoxFuture;
use alloy::{
    primitives::{Address, Uint},
    providers::{ProviderBuilder, RootProvider},
    sol,
    transports::icp::{EthSepoliaService, IcpConfig, IcpTransport, RpcService},
};
use futures::{future, Future};
use ic_cdk::{call, caller};
use ic_stable_structures::{storable::Bound, Storable};
use serde_bytes::ByteBuf;
use IERC721::IERC721Instance;

// Define relevant ERC721 interface
sol! {
    #[sol(rpc)]
    contract IERC721 {
        function balanceOf(address owner) external view returns (uint256);
        function tokenOfOwnerByIndex(address owner, uint256 index) external view returns (uint256);
        function ownerOf(uint256 tokenId) external view returns (address);
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

async fn retry<F, Fut, T>(mut f: F, max_retries: u32) -> Result<T, String>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, String>>,
{
    let mut attempt = 0;

    loop {
        match f().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                attempt += 1;
                if attempt >= max_retries {
                    return Err(e);
                }
            }
        }
    }
}

// TODO: try to fix this async version (got compiler errored)
pub async fn get_caller_owned_tokens() -> Result<Vec<Uint<256, 4>>, String> {
    let CallObjects {
        owner,
        token_contract,
    } = setup_call_objects().await?;

    // First get the balance with retry
    let balance = retry(
        || async {
            token_contract
                .balanceOf(owner)
                .call()
                .await
                .map_err(|e| format!("Failed to call balanceOf: {}", e))
        },
        7, // max retries
    )
    .await?;

    let mut calls: Vec<BoxFuture<Result<Uint<256, 4>, String>>> = Vec::new();

    let balance_i32 = balance
        ._0
        .try_into()
        .map_err(|_| format!("Failed to parse token number"))?;

    for i in 0..balance_i32 {
        let token_contract = token_contract.clone();
        calls.push(Box::pin(async move {
            retry(
                || async {
                    token_contract
                        .tokenOfOwnerByIndex(owner, Uint::from(i))
                        .call()
                        .await
                        .map(|res| res._0)
                        .map_err(|e| format!("Failed to get token at index {}: {}", i, e))
                },
                7, // max retries
            )
            .await
        }));
    }

    let results = future::join_all(calls).await;
    let mut tokens = Vec::new();
    for result in results {
        tokens.push(result?);
    }

    Ok(tokens)
}

// pub async fn get_caller_owned_tokens() -> Result<Vec<Uint<256, 4>>, String> {
//     let CallObjects {
//         owner,
//         token_contract,
//     } = setup_call_objects().await?;

//     // First get the balance with retry
//     let balance = retry(
//         || async {
//             token_contract
//                 .balanceOf(owner)
//                 .call()
//                 .await
//                 .map_err(|e| format!("Failed to call balanceOf: {}", e))
//         },
//         3,
//     )
//     .await?;

//     let balance_u64 = balance
//         ._0
//         .try_into()
//         .map_err(|_| format!("Failed to parse token number"))?;

//     let mut tokens = Vec::new();

//     // Do sequential calls with retry instead of parallel
//     for i in 0..balance_u64 {
//         let token_id = retry(
//             || async {
//                 token_contract
//                     .tokenOfOwnerByIndex(owner, Uint::from(i))
//                     .call()
//                     .await
//                     .map(|res| res._0)
//                     .map_err(|e| format!("Failed to get token at index {}: {}", i, e))
//             },
//             3,
//         )
//         .await?;

//         tokens.push(token_id);
//     }

//     Ok(tokens)
// }

pub async fn get_caller_balance() -> Result<Uint<256, 4>, String> {
    let CallObjects {
        owner,
        token_contract,
    } = setup_call_objects().await?;

    let call_response = retry(
        || async {
            token_contract
                .balanceOf(owner)
                .call()
                .await
                .map_err(|e| format!("Failed to call balanceOf: {}", e))
        },
        7,
    )
    .await?;

    Ok(call_response._0)
}

pub async fn caller_is_owner_of(token_id: u64) -> Result<bool, String> {
    let CallObjects {
        owner,
        token_contract,
    } = setup_call_objects().await?;

    let call_response = retry(
        || async {
            token_contract
                .ownerOf(Uint::from(token_id))
                .call()
                .await
                .map_err(|e| format!("Failed to call ownerOf: {}", e))
        },
        7,
    )
    .await?;

    Ok(call_response._0 == owner)
}

pub async fn check_auth_for_single_node_id(node_id: usize) {
    if dev_mode() {
        return ();
    };
    match caller_is_owner_of(node_id as u64).await {
        Ok(res) => match res {
            true => (),
            false => ic_cdk::trap("Unauthorized"),
        },
        Err(err) => ic_cdk::trap(&format!("Failed to check authorization: {}", err)),
    }
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
    let rpc_service = RpcService::EthSepolia(EthSepoliaService::Alchemy);
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
