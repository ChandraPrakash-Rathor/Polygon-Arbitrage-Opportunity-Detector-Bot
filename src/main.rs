use anyhow::Result;
use chrono::Utc;
use ethers::abi::Abi;
use ethers::contract::Contract;
use ethers::core::types::{Address, U256};
use ethers::providers::{Http, Provider};
use rusqlite::Connection;
use serde::Deserialize;
use std::fs;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::interval;

mod db;

#[derive(Debug, Deserialize)]
struct DexRouters {
    quickswap: String,
    sushiswap: String,
}

#[derive(Debug, Deserialize)]
struct Tokens {
    weth: String,
    usdc: String,
}

#[derive(Debug, Deserialize)]
struct BotSettings {
    min_profit_usdc: f64,
    trade_size: u64,
    est_gas_cost_usdc: f64,
    refresh_rate: u64,
}

#[derive(Debug, Deserialize)]
struct Config {
    rpc_url: String,
    dex: DexRouters,
    tokens: Tokens,
    settings: BotSettings,
}

fn load_config(path: &str) -> Result<Config> {
    let file = fs::read_to_string(path)?;
    Ok(toml::from_str(&file)?)
}

fn load_router_abi(path: &str) -> Result<Abi> {
    let abi_data = fs::read_to_string(path)?;
    Ok(serde_json::from_str(&abi_data)?)
}

async fn fetch_price(
    contract: &Contract<Provider<Http>>,
    trade_size: U256,
    path: Vec<Address>,
) -> U256 {
    contract
        .method::<_, Vec<U256>>("getAmountsOut", (trade_size, path))
        .unwrap()
        .call()
        .await
        .unwrap_or_else(|err| {
            eprintln!("Error fetching price: {:?}", err);
            vec![U256::zero(), U256::zero()]
        })
        .get(1)
        .cloned()
        .unwrap_or(U256::zero())
}

#[tokio::main]
async fn main() -> Result<()> {
    let cfg = load_config("config.toml")?;
    println!(" Config loaded: {:?}", cfg);

    let abi = load_router_abi("abi/uniswap_v2_router02_abi.json")?;
    println!(" ABI loaded");

    db::init_db()?;
    let conn = Connection::open("arbitrage.db")?;
    println!(" Database connected");

    let provider = Provider::<Http>::try_from(cfg.rpc_url.clone())?;
    let weth: Address = cfg.tokens.weth.parse()?;
    let usdc: Address = cfg.tokens.usdc.parse()?;
    let trade_size = U256::from(cfg.settings.trade_size);
    let dex1_address: Address = cfg.dex.quickswap.parse()?;
    let dex2_address: Address = cfg.dex.sushiswap.parse()?;

    let quickswap = Contract::new(dex1_address, abi.clone(), Arc::new(provider.clone()));
    let sushiswap = Contract::new(dex2_address, abi.clone(), Arc::new(provider.clone()));

    println!(" DEX contracts ready");

    let mut ticker = interval(Duration::from_secs(cfg.settings.refresh_rate));
    loop {
        ticker.tick().await;

        println!("\n Checking prices...");

        let quick_price = fetch_price(&quickswap, trade_size, vec![weth, usdc]).await;
        let sushi_price = fetch_price(&sushiswap, trade_size, vec![weth, usdc]).await;

        if quick_price.is_zero() || sushi_price.is_zero() {
            println!(" Skipping invalid prices");
            continue;
        }

        let quick_usdc = quick_price.as_u128() as f64 / 1e6;
        let sushi_usdc = sushi_price.as_u128() as f64 / 1e6;

        println!(" QuickSwap: {} USDC | SushiSwap: {} USDC", quick_usdc, sushi_usdc);

        let (buy_on, sell_on, diff) = if quick_price > sushi_price {
            ("SushiSwap", "QuickSwap", quick_price - sushi_price)
        } else if sushi_price > quick_price {
            ("QuickSwap", "SushiSwap", sushi_price - quick_price)
        } else {
            println!(" Prices equal → No arbitrage");
            continue;
        };

        let gas_cost = U256::from((cfg.settings.est_gas_cost_usdc * 1e6) as u128);
        let net_profit = if diff > gas_cost { diff - gas_cost } else { U256::zero() };
        let profit_usdc = net_profit.as_u128() as f64 / 1e6;

        println!(" Net Profit (after gas): {:.6} USDC", profit_usdc);

        if profit_usdc > cfg.settings.min_profit_usdc {
            println!(" Arbitrage Opportunity: Buy on {} → Sell on {}", buy_on, sell_on);
            let timestamp = Utc::now().to_rfc3339();
            conn.execute(
                "INSERT INTO arbitrage_bot (buy_dex, sell_dex, profit_usdc, timestamp) 
                 VALUES (?1, ?2, ?3, ?4)",
                (&buy_on, &sell_on, &profit_usdc, &timestamp),
            )?;
            println!(" Opportunity saved!");
        } else {
            println!(" Profit too small, skipping");
        }
    }
}
