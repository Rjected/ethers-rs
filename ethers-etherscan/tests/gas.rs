mod common;
pub use common::run_at_least_duration;
use std::time::Duration;
use ethers_etherscan::{errors::EtherscanError, Client};
use ethers_core::types::Chain;
use serial_test::serial;

#[tokio::test]
#[serial]
async fn gas_estimate_success() {
    run_at_least_duration(Duration::from_millis(250), async {
        let client = Client::new_from_env(Chain::Mainnet).unwrap();

        let result = client.gas_estimate(2000000000u32.into()).await;

        assert!(result.is_ok());
    })
    .await
}

#[tokio::test]
#[serial]
async fn gas_estimate_error() {
    run_at_least_duration(Duration::from_millis(250), async {
        let client = Client::new_from_env(Chain::Mainnet).unwrap();

        let err = client.gas_estimate(7123189371829732819379218u128.into()).await.unwrap_err();

        assert!(matches!(err, EtherscanError::GasEstimationFailed));
    })
    .await
}

#[tokio::test]
#[serial]
async fn gas_oracle_success() {
    run_at_least_duration(Duration::from_millis(250), async {
        let client = Client::new_from_env(Chain::Mainnet).unwrap();

        let result = client.gas_oracle().await;

        assert!(result.is_ok());

        let oracle = result.unwrap();

        assert!(oracle.safe_gas_price > 0);
        assert!(oracle.propose_gas_price > 0);
        assert!(oracle.fast_gas_price > 0);
        assert!(oracle.last_block > 0);
        assert!(oracle.suggested_base_fee > 0.0);
        assert!(!oracle.gas_used_ratio.is_empty());
    })
    .await
}
