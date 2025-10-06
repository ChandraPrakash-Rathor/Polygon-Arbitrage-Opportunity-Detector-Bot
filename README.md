# Polygon Arbitrage Bot

A Rust bot that monitors the Polygon blockchain for arbitrage opportunities between QuickSwap and SushiSwap. It tracks the **WETH/USDC** trading pair, calculates potential profit, and stores opportunities in a SQLite database (`arbitrage.db`).

---

## Project Structure

```

polygon-arbitrage-bot/
├── Cargo.toml                        # Rust dependencies
├── README.md                         # Project documentation
├── config.toml                       # Bot configuration: RPC, DEX, tokens, thresholds
├── arbitrage.db                            # SQLite database for detected opportunities
├── src/
│   ├── main.rs                       # Main bot logic
│   └── db.rs                         # Database setup and connection
└── abi/
└── uniswap_v2_router02_abi.json  # ABI for DEX routers

````

---

## Features

- Fetches token prices from multiple DEXs on Polygon.
- Detects profitable arbitrage opportunities above a configurable threshold.
- Estimates net profit after gas fees.
- Logs opportunities to console and SQLite database.
- Fully configurable via `config.toml`.

---

## Tech Stack

- **Blockchain Network**: Polygon  
- **DEX Platforms**: QuickSwap & SushiSwap (Uniswap V2 ABI)  
- **Language**: Rust  
- **Database**: SQLite (`rusqlite`)  
- **RPC Access**: Alchemy, Ankr, or other Polygon endpoints  

---

## Database Schema

**Database:** `arbitrage.db`  
**Table:** `arbitrage_bot`

| Column        | Type    | Description                          |
| ------------- | ------- | ------------------------------------ |
| id            | INTEGER | Auto-incrementing ID                  |
| buy_dex       | TEXT    | DEX to buy from                       |
| sell_dex      | TEXT    | DEX to sell on                        |
| profit_usdc   | REAL    | Estimated profit in USDC              |
| timestamp     | TEXT    | UTC timestamp of the opportunity      |

---

## Setup

### 1. Install Rust
Install Rust from [rust-lang.org](https://www.rust-lang.org/tools/install).

### 2. Configure the Bot
Edit `config.toml`:

```toml
rpc_url = "https://polygon-mainnet.g.alchemy.com/v2/YOUR_API_KEY"

[dex]
quickswap = "QUICKSWAP_ROUTER_ADDRESS"
sushiswap = "SUSHISWAP_ROUTER_ADDRESS"

[tokens]
weth = "WETH_ADDRESS"
usdc = "USDC_ADDRESS"

[settings]
min_profit_usdc = 10.0
trade_size = 1
est_gas_cost_usdc = 5.0
refresh_rate = 30
````

Place your ABI in `abi/uniswap_v2_router02_abi.json`.

### 3. Run the Bot

```bash
cargo run
```

* Creates `arbitrage.db` automatically.
* Fetches prices at intervals specified in `config.toml`.
* Example output:

```
Checking prices...
QuickSwap: 4147.445571 USDC | SushiSwap: 4097.557421 USDC
Arbitrage Opportunity: Buy on SushiSwap → Sell on QuickSwap
Net Profit (after gas): 44.88815 USDC
Opportunity saved!
```

---

## How It Works

1. Loads configuration and ABI.
2. Connects to Polygon RPC.
3. Initializes DEX contracts for querying prices.
4. Loops every `refresh_rate` seconds:

   * Fetches prices for 1 WETH → USDC from both DEXs.
   * Compares prices to identify profitable trades.
   * Calculates net profit after gas.
   * Logs and saves opportunities to SQLite (`arbitrage.db`).

---

### Arbitrage Logic

* **Price Fetching:** `getAmountsOut(1 WETH, [WETH, USDC])`
* **Compare Prices:** Buy on lower-price DEX, sell on higher-price DEX.
* **Profit Calculation:** `profit = price_difference - gas_fee`
* **Threshold Filter:** Log only if `profit > min_profit_usdc`.

---

## Usage

* **Start Monitoring:** `cargo run`
* **Stop the Bot:** Ctrl+C
* **View Opportunities:** Open `arbitrage.db` using [DB Browser for SQLite](https://sqlitebrowser.org/)

```sql
SELECT * FROM arbitrage_bot;
