use crate::settings::read_settings;
use agent::{create_agent_from_identity, create_anonymous_identity, create_identity_from_pem_file};
use openssl::ssl::{SslConnector, SslMethod};
use postgres_openssl::MakeTlsConnector;
use std::env;

use db_updates::get_db_updates;
use kong_backend::KongBackend;
use kong_data::KongData;

mod agent;
mod claims;
mod db_updates;
mod kong_backend;
mod kong_data;
mod kong_settings;
mod kong_update;
mod lp_tokens;
mod math_helpers;
mod nat_helpers;
mod pools;
mod requests;
mod settings;
mod tokens;
mod transfers;
mod txs;
mod users;

const LOCAL_REPLICA: &str = "http://localhost:4943";
const MAINNET_REPLICA: &str = "https://ic0.app";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = env::args().collect::<Vec<String>>();
    let config = read_settings()?;
    let dfx_pem_file = &config.dfx_pem_file;
    let db_host = &config.database.host;
    let db_port = &config.database.port;
    let db_user = &config.database.user;
    let db_password = &config.database.password;
    let mut builder = SslConnector::builder(SslMethod::tls()).map_err(|e| format!("SSL error: {}", e))?;
    if config.database.ca_cert.is_some() {
        builder
            .set_ca_file(config.database.ca_cert.as_ref().unwrap())
            .map_err(|e| format!("CA file error: {}", e))?;
    }
    let tls = MakeTlsConnector::new(builder.build());
    let db_name = &config.database.db_name;
    let mut db_config = tokio_postgres::Config::new();
    db_config.host(db_host);
    db_config.port(*db_port);
    db_config.user(db_user);
    db_config.password(db_password);
    db_config.dbname(db_name);
    let (db_client, connection) = db_config.connect(tls).await?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("DB connection error: {}", e);
        }
    });

    let (replica_url, is_mainnet) = if args.contains(&"--mainnet".to_string()) {
        (MAINNET_REPLICA, true)
    } else {
        (LOCAL_REPLICA, false)
    };

    // read from flat files (./backups) and update kong_data
    if args.contains(&"--kong_data".to_string()) {
        let dfx_pem_file = dfx_pem_file.as_ref().ok_or("dfx identity required for Kong Data")?;
        let identity = create_identity_from_pem_file(dfx_pem_file);
        let agent = create_agent_from_identity(replica_url, identity, is_mainnet).await?;
        let kong_data = KongData::new(&agent, is_mainnet).await;
        // Dump to kong_data
        users::update_users(&kong_data).await?;
        tokens::update_tokens(&kong_data).await?;
        pools::update_pools(&kong_data).await?;
        lp_tokens::update_lp_tokens(&kong_data).await?;
        requests::update_requests(&kong_data).await?;
        claims::update_claims_on_kong_data(&kong_data).await?;
        transfers::update_transfers(&kong_data).await?;
        txs::update_txs(&kong_data).await?;
    }

    // read from flat files (./backups) and update kong_backend. used for development
    if args.contains(&"--kong_backend".to_string()) {
        let dfx_pem_file = dfx_pem_file.as_ref().ok_or("dfx identity required for Kong Backend")?;
        let identity = create_identity_from_pem_file(dfx_pem_file);
        let agent = create_agent_from_identity(replica_url, identity, is_mainnet).await?;
        let kong_backend = KongBackend::new(&agent).await;
        // Dump to kong_backend
        users::update_users(&kong_backend).await?;
        tokens::update_tokens(&kong_backend).await?;
        pools::update_pools(&kong_backend).await?;
        // lp_tokens::update_lp_tokens(&kong_backend).await?;
        // requests::update_requests(&kong_backend).await?;
        // claims::update_claims(&kong_backend).await?;
        // transfers::update_transfers(&kong_backend).await?;
        // txs::update_txs(&kong_backend).await?;
    }

    let tokens_map;
    let pools_map;
    // read from flat files (./backups) and update database
    if args.contains(&"--database".to_string()) {
        // Dump to database
        users::update_users_on_database(&db_client).await?;
        tokens_map = tokens::update_tokens_on_database(&db_client).await?;
        pools_map = pools::update_pools_on_database(&db_client, &tokens_map).await?;
        lp_tokens::update_lp_tokens_on_database(&db_client, &tokens_map).await?;
        requests::update_requests_on_database(&db_client).await?;
        claims::update_claims_on_database(&db_client, &tokens_map).await?;
        transfers::update_transfers_on_database(&db_client, &tokens_map).await?;
        txs::update_txs_on_database(&db_client, &tokens_map, &pools_map).await?;
    } else {
        tokens_map = tokens::load_tokens_from_database(&db_client).await?;
        pools_map = pools::load_pools_from_database(&db_client).await?;
    }

    // read from kong_data and update database
    if args.contains(&"--db_updates".to_string()) {
        let identity = create_anonymous_identity();
        let agent = create_agent_from_identity(replica_url, identity, is_mainnet).await?;
        let kong_data = KongData::new(&agent, is_mainnet).await;
        get_db_updates(None, &kong_data, &db_client, tokens_map, pools_map).await?;
    }

    Ok(())
}
