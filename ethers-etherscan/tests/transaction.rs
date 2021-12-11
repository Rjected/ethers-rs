mod common;
pub use common::run_at_least_duration;
use std::time::Duration;
use ethers_etherscan::{errors::EtherscanError, Client};
use ethers_core::types::Chain;
use serial_test::serial;

#[tokio::test]
#[serial]
async fn check_contract_execution_status_success() {
    run_at_least_duration(Duration::from_millis(250), async {
        let client = Client::new_from_env(Chain::Mainnet).unwrap();

        let status = client
            .check_contract_execution_status(
                "0x16197e2a0eacc44c1ebdfddcfcfcafb3538de557c759a66e0ba95263b23d9007",
            )
            .await;

        assert!(status.is_ok());
    })
    .await
}

#[tokio::test]
#[serial]
async fn check_contract_execution_status_error() {
    run_at_least_duration(Duration::from_millis(250), async {
        let client = Client::new_from_env(Chain::Mainnet).unwrap();

        let err = client
            .check_contract_execution_status(
                "0x15f8e5ea1079d9a0bb04a4c58ae5fe7654b5b2b4463375ff7ffb490aa0032f3a",
            )
            .await
            .unwrap_err();

        assert!(matches!(err, EtherscanError::ExecutionFailed(_)));
        assert_eq!(err.to_string(), "contract execution call failed: Bad jump destination");
    })
    .await
}

#[tokio::test]
#[serial]
async fn check_transaction_receipt_status_success() {
    run_at_least_duration(Duration::from_millis(250), async {
        let client = Client::new_from_env(Chain::Mainnet).unwrap();

        let success = client
            .check_transaction_receipt_status(
                "0x513c1ba0bebf66436b5fed86ab668452b7805593c05073eb2d51d3a52f480a76",
            )
            .await;

        assert!(success.is_ok());
    })
    .await
}

#[tokio::test]
#[serial]
async fn check_transaction_receipt_status_failed() {
    run_at_least_duration(Duration::from_millis(250), async {
        let client = Client::new_from_env(Chain::Mainnet).unwrap();

        let err = client
            .check_transaction_receipt_status(
                "0x21a29a497cb5d4bf514c0cca8d9235844bd0215c8fab8607217546a892fd0758",
            )
            .await
            .unwrap_err();

        assert!(matches!(err, EtherscanError::TransactionReceiptFailed));
    })
    .await
}
