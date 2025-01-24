use crate::siwe_principal;
use ic_cdk::{call, caller};
use serde_bytes::ByteBuf;

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
