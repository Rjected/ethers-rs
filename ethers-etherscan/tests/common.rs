use ethers_etherscan::{errors::EtherscanError, Client};
use ethers_core::types::Chain;
use std::{
    future::Future,
    time::{Duration, SystemTime},
};

pub async fn run_at_least_duration(duration: Duration, block: impl Future) {
    let start = SystemTime::now();
    block.await;
    if let Some(sleep) = duration.checked_sub(start.elapsed().unwrap()) {
        tokio::time::sleep(sleep).await;
    }
}

#[test]
fn chain_not_supported() {
    let err = Client::new_from_env(Chain::XDai).unwrap_err();

    assert!(matches!(err, EtherscanError::ChainNotSupported(_)));
    assert_eq!(err.to_string(), "chain XDai not supported");
}
