use std::sync::Arc;

use dotenv::dotenv;
use eyre::Result;
use kakarot_rpc::eth_provider::database::Database;
use kakarot_rpc::eth_provider::provider::EthDataProvider;
use kakarot_rpc::eth_rpc::config::RPCConfig;
use kakarot_rpc::eth_rpc::rpc::KakarotRpcModuleBuilder;
use kakarot_rpc::eth_rpc::run_server;
use kakarot_rpc::starknet_client::config::{
    env_var, JsonRpcClientBuilder, KakarotRpcConfig, Network, SequencerGatewayProviderBuilder,
};
use kakarot_rpc::starknet_client::KakarotClient;
use mongodb::options::{DatabaseOptions, ReadConcern, WriteConcern};
use starknet::providers::jsonrpc::HttpTransport;
use starknet::providers::{JsonRpcClient, SequencerGatewayProvider};
use tracing_subscriber::util::SubscriberInitExt;

enum StarknetProvider {
    JsonRpcClient(JsonRpcClient<HttpTransport>),
    SequencerGatewayProvider(SequencerGatewayProvider),
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    // Environment variables are safe to use after this
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()?;
    tracing_subscriber::FmtSubscriber::builder().with_env_filter(filter).finish().try_init()?;

    let starknet_config = KakarotRpcConfig::from_env()?;

    let rpc_config = RPCConfig::from_env()?;

    let starknet_provider = match &starknet_config.network {
        Network::Madara | Network::Katana | Network::Sharingan => {
            StarknetProvider::JsonRpcClient(JsonRpcClientBuilder::with_http(&starknet_config).unwrap().build())
        }
        Network::JsonRpcProvider(url) => {
            StarknetProvider::JsonRpcClient(JsonRpcClientBuilder::new(HttpTransport::new(url.clone())).build())
        }
        _ => StarknetProvider::SequencerGatewayProvider(
            SequencerGatewayProviderBuilder::new(&starknet_config.network).build(),
        ),
    };

    let db_client = mongodb::Client::with_uri_str(
        env_var("MONGO_CONNECTION_STRING").expect("Missing MONGO_CONNECTION_STRING .env"),
    )
    .await?;
    let db = Database::new(db_client.database_with_options(
        &env_var("MONGO_DATABASE_NAME").expect("Missing MONGO_DATABASE_NAME from .env"),
        DatabaseOptions::builder().read_concern(ReadConcern::MAJORITY).write_concern(WriteConcern::MAJORITY).build(),
    ));

    let kakarot_rpc_module = match starknet_provider {
        StarknetProvider::JsonRpcClient(starknet_provider) => {
            let starknet_provider = Arc::new(starknet_provider);
            let kakarot_client = Arc::new(KakarotClient::new(starknet_config, starknet_provider.clone()));
            let eth_provider = EthDataProvider::new(db, starknet_provider);
            KakarotRpcModuleBuilder::new(kakarot_client, eth_provider).rpc_module()
        }
        StarknetProvider::SequencerGatewayProvider(starknet_provider) => {
            let starknet_provider = Arc::new(starknet_provider);
            let kakarot_client = Arc::new(KakarotClient::new(starknet_config, starknet_provider.clone()));
            let eth_provider = EthDataProvider::new(db, starknet_provider);
            KakarotRpcModuleBuilder::new(kakarot_client, eth_provider).rpc_module()
        }
    }?;

    let (server_addr, server_handle) = run_server(kakarot_rpc_module, rpc_config).await?;

    let url = format!("http://{server_addr}");

    println!("RPC Server running on {url}...");

    server_handle.stopped().await;

    Ok(())
}