mod common;
pub use common::run_at_least_duration;
use std::time::Duration;
use ethers_etherscan::{contract::VerifyContract, Client};
use ethers_core::types::Chain;
use serial_test::serial;

#[tokio::test]
#[serial]
#[ignore]
async fn can_fetch_contract_abi() {
    run_at_least_duration(Duration::from_millis(250), async {
        let client = Client::new_from_env(Chain::Mainnet).unwrap();

        let _abi = client
            .contract_abi("0xBB9bc244D798123fDe783fCc1C72d3Bb8C189413".parse().unwrap())
            .await
            .unwrap();
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore]
async fn can_fetch_contract_source_code() {
    run_at_least_duration(Duration::from_millis(250), async {
        let client = Client::new_from_env(Chain::Mainnet).unwrap();

        let _meta = client
            .contract_source_code("0xBB9bc244D798123fDe783fCc1C72d3Bb8C189413".parse().unwrap())
            .await
            .unwrap();
    })
    .await
}

#[tokio::test]
#[serial]
#[ignore]
async fn can_verify_contract() {
    run_at_least_duration(Duration::from_millis(250), async {
        // TODO this needs further investigation

        // https://etherscan.io/address/0x9e744c9115b74834c0f33f4097f40c02a9ac5c33#code
        let contract = include_str!("../resources/UniswapExchange.sol");
        let address = "0x9e744c9115b74834c0f33f4097f40c02a9ac5c33".parse().unwrap();
        let compiler_version = "v0.5.17+commit.d19bba13";
        let constructor_args = "0x000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000005f5e1000000000000000000000000000000000000000000000000000000000000000007596179537761700000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000035941590000000000000000000000000000000000000000000000000000000000";

        let client = Client::new_from_env(Chain::Mainnet).unwrap();

        let contract =
            VerifyContract::new(address, contract.to_string(), compiler_version.to_string())
                .constructor_arguments(Some(constructor_args))
                .optimization(true)
                .runs(200);

        let _resp = client.submit_contract_verification(&contract).await;
    }).await
}
