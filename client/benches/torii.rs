#![allow(missing_docs, clippy::pedantic)]

use std::thread;

use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use iroha::samples::{construct_executor, get_config};
use iroha_client::{
    client::{asset, Client},
    crypto::KeyPair,
    data_model::prelude::*,
};
use iroha_genesis::{GenesisNetwork, RawGenesisBlockBuilder};
use iroha_primitives::unique_vec;
use iroha_version::Encode;
use test_network::{get_chain_id, get_key_pair, Peer as TestPeer, PeerBuilder, TestRuntime};
use tokio::runtime::Runtime;

const MINIMUM_SUCCESS_REQUEST_RATIO: f32 = 0.9;

// assumes that config is having a complete genesis key pair
fn get_genesis_key_pair(config: &iroha_config::iroha::Configuration) -> KeyPair {
    if let (public_key, Some(private_key)) =
        (&config.genesis.public_key, &config.genesis.private_key)
    {
        KeyPair::new(public_key.clone(), private_key.clone()).expect("Should be valid")
    } else {
        panic!("Cannot get genesis key pair from the config. Probably a bug.")
    }
}

fn query_requests(criterion: &mut Criterion) {
    let mut peer = <TestPeer>::new().expect("Failed to create peer");

    let chain_id = get_chain_id();
    let configuration = get_config(
        unique_vec![peer.id.clone()],
        Some(chain_id.clone()),
        Some(get_key_pair()),
    );

    let rt = Runtime::test();
    let genesis = GenesisNetwork::new(
        RawGenesisBlockBuilder::default()
            .domain("wonderland".parse().expect("Valid"))
            .account(
                "alice".parse().expect("Valid"),
                get_key_pair().public_key().clone(),
            )
            .finish_domain()
            .executor(
                construct_executor("../default_executor").expect("Failed to construct executor"),
            )
            .build(),
        &chain_id,
        &get_genesis_key_pair(&configuration),
    )
    .expect("genesis creation failed");

    let builder = PeerBuilder::new()
        .with_configuration(configuration)
        .with_into_genesis(genesis);

    rt.block_on(builder.start_with_peer(&mut peer));
    rt.block_on(async {
        iroha_logger::test_logger()
            .reload_level(iroha_client::data_model::Level::ERROR)
            .await
            .unwrap()
    });
    let mut group = criterion.benchmark_group("query-requests");
    let domain_id: DomainId = "domain".parse().expect("Valid");
    let create_domain = Register::domain(Domain::new(domain_id.clone()));
    let account_id = AccountId::new(domain_id.clone(), "account".parse().expect("Valid"));
    let (public_key, _) = KeyPair::generate()
        .expect("Failed to generate KeyPair")
        .into();
    let create_account = Register::account(Account::new(account_id.clone(), [public_key]));
    let asset_definition_id = AssetDefinitionId::new(domain_id, "xor".parse().expect("Valid"));
    let create_asset =
        Register::asset_definition(AssetDefinition::quantity(asset_definition_id.clone()));
    let quantity: u32 = 200;
    let mint_asset = Mint::asset_quantity(
        quantity,
        AssetId::new(asset_definition_id, account_id.clone()),
    );
    let mut client_config =
        iroha_client::samples::get_client_config(get_chain_id(), &get_key_pair());

    client_config.torii_api_url = format!("http://{}", peer.api_address).parse().unwrap();

    let iroha_client = Client::new(&client_config).expect("Invalid client configuration");
    thread::sleep(std::time::Duration::from_millis(5000));

    let instructions: [InstructionBox; 4] = [
        create_domain.into(),
        create_account.into(),
        create_asset.into(),
        mint_asset.into(),
    ];
    let _ = iroha_client
        .submit_all(instructions)
        .expect("Failed to prepare state");

    let request = asset::by_account_id(account_id);
    thread::sleep(std::time::Duration::from_millis(1500));
    let mut success_count = 0;
    let mut failures_count = 0;
    let _dropable = group.throughput(Throughput::Bytes(request.encode().len() as u64));
    let _dropable2 = group.bench_function("query", |b| {
        b.iter(|| {
            let iter: Result<Vec<_>, _> = iroha_client
                .request(request.clone())
                .and_then(Iterator::collect);

            match iter {
                Ok(assets) => {
                    assert!(!assets.is_empty());
                    success_count += 1;
                }
                Err(e) => {
                    eprintln!("Query failed: {e}");
                    failures_count += 1;
                }
            }
        });
    });
    println!("Success count: {success_count}, Failures count: {failures_count}");
    group.finish();
    if (failures_count + success_count) > 0 {
        assert!(
            success_count as f32 / (failures_count + success_count) as f32
                > MINIMUM_SUCCESS_REQUEST_RATIO
        );
    }
}

fn instruction_submits(criterion: &mut Criterion) {
    println!("instruction submits");
    let rt = Runtime::test();
    let mut peer = <TestPeer>::new().expect("Failed to create peer");

    let chain_id = get_chain_id();
    let configuration = get_config(
        unique_vec![peer.id.clone()],
        Some(chain_id.clone()),
        Some(get_key_pair()),
    );
    let genesis = GenesisNetwork::new(
        RawGenesisBlockBuilder::default()
            .domain("wonderland".parse().expect("Valid"))
            .account(
                "alice".parse().expect("Valid"),
                configuration.public_key.clone(),
            )
            .finish_domain()
            .executor(
                construct_executor("../default_executor").expect("Failed to construct executor"),
            )
            .build(),
        &chain_id,
        &get_genesis_key_pair(&configuration),
    )
    .expect("failed to create genesis");
    let builder = PeerBuilder::new()
        .with_configuration(configuration)
        .with_into_genesis(genesis);
    rt.block_on(builder.start_with_peer(&mut peer));
    let mut group = criterion.benchmark_group("instruction-requests");
    let domain_id: DomainId = "domain".parse().expect("Valid");
    let create_domain: InstructionBox = Register::domain(Domain::new(domain_id.clone())).into();
    let account_id = AccountId::new(domain_id.clone(), "account".parse().expect("Valid"));
    let (public_key, _) = KeyPair::generate()
        .expect("Failed to generate Key-pair.")
        .into();
    let create_account = Register::account(Account::new(account_id.clone(), [public_key])).into();
    let asset_definition_id = AssetDefinitionId::new(domain_id, "xor".parse().expect("Valid"));
    let mut client_config =
        iroha_client::samples::get_client_config(get_chain_id(), &get_key_pair());
    client_config.torii_api_url = format!("http://{}", peer.api_address).parse().unwrap();
    let iroha_client = Client::new(&client_config).expect("Invalid client configuration");
    thread::sleep(std::time::Duration::from_millis(5000));
    let _ = iroha_client
        .submit_all([create_domain, create_account])
        .expect("Failed to create role.");
    thread::sleep(std::time::Duration::from_millis(500));
    let mut success_count = 0;
    let mut failures_count = 0;
    let _dropable = group.bench_function("instructions", |b| {
        b.iter(|| {
            let quantity: u32 = 200;
            let mint_asset = Mint::asset_quantity(
                quantity,
                AssetId::new(asset_definition_id.clone(), account_id.clone()),
            );
            match iroha_client.submit(mint_asset) {
                Ok(_) => success_count += 1,
                Err(e) => {
                    eprintln!("Failed to execute instruction: {e}");
                    failures_count += 1;
                }
            };
        })
    });
    println!("Success count: {success_count}, Failures count: {failures_count}");
    group.finish();
    if (failures_count + success_count) > 0 {
        assert!(
            success_count as f32 / (failures_count + success_count) as f32
                > MINIMUM_SUCCESS_REQUEST_RATIO
        );
    }
}

criterion_group!(instructions, instruction_submits);
criterion_group!(queries, query_requests);
criterion_main!(queries, instructions);
