# PumpBTC Stellar

PumpBTC Stellar is a suite of Soroban smart contracts that allows users to stake Wrapped Bitcoin (WBTC, sBTC, BTCB) and earn rewards in BTC through Babylon staking. The system provides both standard and instant unstake options with configurable fees on the Stellar network.

![Overview](overview.png)

The project facilitates secure Bitcoin staking with a 10-day unstaking period and instant unstake capabilities. This README provides detailed instructions on deployment, configuration, and usage of the contract functions.

## Deployment

### Setup Environment

1. Install Rust and Soroban CLI:
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Soroban CLI
cargo install --locked soroban-cli
```

2. Configure Soroban for your target network:
```bash
# For Testnet
soroban network add --global testnet \
  --rpc-url https://soroban-testnet.stellar.org:443 \
  --network-passphrase "Test SDF Network ; September 2015"

# For Mainnet
soroban network add --global mainnet \
  --rpc-url https://soroban-mainnet.stellar.org:443 \
  --network-passphrase "Public Global Stellar Network ; September 2015"
```

### Build & Deploy

1. Build all contracts:
```bash
cargo build --target wasm32-unknown-unknown --release
```

2. Optimize contracts:
```bash
soroban contract optimize --wasm target/wasm32-unknown-unknown/release/asset_token.wasm
soroban contract optimize --wasm target/wasm32-unknown-unknown/release/pump_token.wasm
soroban contract optimize --wasm target/wasm32-unknown-unknown/release/pumpbtc_staking.wasm
```

3. Deploy contracts to testnet:
```bash
# Deploy Asset Token (WBTC)
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/asset_token.optimized.wasm \
  --source <YOUR_SECRET_KEY> \
  --network testnet

# Deploy Pump Token
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/pump_token.optimized.wasm \
  --source <YOUR_SECRET_KEY> \
  --network testnet

# Deploy PumpBTC Staking
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/pumpbtc_staking.optimized.wasm \
  --source <YOUR_SECRET_KEY> \
  --network testnet
```

4. Initialize contracts:
```bash
# Initialize Asset Token
soroban contract invoke \
  --id <ASSET_TOKEN_CONTRACT_ID> \
  --source <YOUR_SECRET_KEY> \
  --network testnet \
  -- initialize \
  --admin <ADMIN_ADDRESS> \
  --decimal 8 \
  --name "Wrapped Bitcoin" \
  --symbol "WBTC"

# Initialize Pump Token
soroban contract invoke \
  --id <PUMP_TOKEN_CONTRACT_ID> \
  --source <YOUR_SECRET_KEY> \
  --network testnet \
  -- initialize \
  --admin <ADMIN_ADDRESS> \
  --minter <STAKING_CONTRACT_ID> \
  --name "PumpBTC" \
  --symbol "pumpBTC"

# Initialize PumpBTC Staking
soroban contract invoke \
  --id <STAKING_CONTRACT_ID> \
  --source <YOUR_SECRET_KEY> \
  --network testnet \
  -- initialize \
  --admin <ADMIN_ADDRESS> \
  --pump_token_address <PUMP_TOKEN_CONTRACT_ID> \
  --asset_token_address <ASSET_TOKEN_CONTRACT_ID>
```

## Contract Overview

### Theory

Users deposit Wrapped BTC into our Soroban contract, from which the operator manually exchanges Wrapped BTC for BTC daily and deposits it into Babylon. This amount is referred to as **X**.

When a user requests to unstake Wrapped BTC, the operator withdraws BTC from Babylon, which takes approximately 7 days, and then exchanges it for Wrapped BTC over the next day, delivering it to the user by date **T+10**. This amount is referred to as **Y**.

Users can also request an instant unstake, receiving Wrapped BTC immediately from other users' stakes that have not yet been exchanged for BTC, subject to an extra fee. This amount is referred to as **Z**, and must satisfy **Z < X**.

![Asset Flow](flow.png)

### Contracts and Architecture

The system consists of three main contracts:

1. **Asset Token**: A standard Stellar token contract representing Wrapped Bitcoin (8 decimals)
2. **Pump Token**: A mintable token contract representing staked Bitcoin (pumpBTC, 8 decimals)
3. **PumpBTC Staking**: The main staking contract containing all staking logic

Upon staking Wrapped BTC, the staking contract mints pumpBTC tokens for the user at a 1:1 ratio.

## Main Contract Functions

**Note**: All quantity-related variables use 8 decimal places to match Bitcoin precision.

### View Functions

| Function | Return Type | Description |
|----------|-------------|-------------|
| `get_total_staking_amount()` | `i128` | Total amount of BTC currently staked |
| `get_total_staking_cap()` | `i128` | Maximum cap for BTC staking |
| `get_total_requested_amount()` | `i128` | Total amount of BTC requested for unstake, not yet claimed |
| `get_total_claimable_amount()` | `i128` | Total amount of BTC available for claiming |
| `get_pending_stake_amount()` | `i128` | Amount staked today minus amount instantly unstaked today (X - Z) |
| `get_collected_fee()` | `i128` | Collected fees (in WBTC, 8 decimals) |
| `get_operator()` | `Option<Address>` | Address able to withdraw or deposit BTC to the contract |
| `get_instant_unstake_fee()` | `i128` | Fee rate for instant unstake (default is 300 = 3%) |
| `get_normal_unstake_fee()` | `i128` | Fee rate for normal unstake (default is 0 = 0%) |
| `get_pending_unstake_time(user, slot)` | `u64` | Timestamp for a user's unstake request in a specific date slot |
| `get_pending_unstake_amount(user, slot)` | `i128` | Amount requested for unstake by a user in a specific date slot |
| `is_paused()` | `bool` | Whether the contract is currently paused |

### Admin Write Functions

| Function | Parameters | Description |
|----------|------------|-------------|
| `set_stake_asset_cap(new_cap)` | `new_cap: i128` | Set the staking cap |
| `set_instant_unstake_fee(new_fee)` | `new_fee: i128` | Set the fee rate for instant unstake (0-10000) |
| `set_normal_unstake_fee(new_fee)` | `new_fee: i128` | Set the fee rate for normal unstake (0-10000) |
| `set_operator(new_operator)` | `new_operator: Address` | Set the operator address for withdrawals and deposits |
| `set_only_allow_stake(allow)` | `allow: bool` | Enable/disable unstaking (for initial staking phase) |
| `collect_fee()` | - | Transfer collected fees out of the contract |
| `withdraw()` | - | Withdraw pending stake amount (X - Z) from contract |
| `deposit(amount)` | `amount: i128` | Deposit WBTC equivalent to unstake requests after 10 days |
| `withdraw_and_deposit(amount)` | `amount: i128` | Combine withdraw and deposit operations |
| `pause()` | - | Pause all contract operations |
| `unpause()` | - | Resume contract operations |
| `transfer_admin(new_admin)` | `new_admin: Address` | Initiate admin transfer |
| `accept_admin()` | - | Accept admin transfer (must be called by pending admin) |

### User Write Functions

| Function | Parameters | Description |
|----------|------------|-------------|
| `stake(user, amount)` | `user: Address, amount: i128` | Stake a specified amount of WBTC (8 decimals) |
| `unstake_request(user, amount)` | `user: Address, amount: i128` | Request to unstake a specified amount of WBTC |
| `claim_slot(user, slot)` | `user: Address, slot: u32` | Claim unstaked WBTC for a specific slot after 10-day period |
| `claim_all(user)` | `user: Address` | Claim all available unstaked WBTC after 10-day period |
| `unstake_instant(user, amount)` | `user: Address, amount: i128` | Instantly unstake WBTC with fee |

### Events

Each function emits corresponding events for tracking and integration:

- `StakeEvent`: When users stake WBTC
- `UnstakeRequestEvent`: When users request unstaking
- `UnstakeInstantEvent`: When users instantly unstake
- `ClaimSlotEvent`: When users claim specific slots
- `ClaimAllEvent`: When users claim all available amounts
- `WithdrawEvent`: When operator withdraws funds
- `DepositEvent`: When operator deposits funds
- `CollectFeeEvent`: When admin collects fees

## Testing

Run the comprehensive test suite:

```bash
cargo test
```

The test suite includes:
- Basic staking and unstaking functionality
- Fee collection and calculation
- Time-based claiming mechanisms
- Admin functions and access control
- Error handling and edge cases
- Pause/unpause functionality

## Security Features

- **Two-step admin transfer**: Prevents accidental admin changes
- **Operator role separation**: Separates admin and operational duties
- **Pause functionality**: Emergency stop capability
- **Fee validation**: Ensures fees are within valid ranges (0-100%)
- **Time-based claiming**: Enforces 10-day unstaking period
- **Reentrancy protection**: Built-in Soroban security features

## Date Slots and Unstaking Cycle

The contract uses a date slot system for managing unstake requests:
- Each day has a unique slot identifier
- Users can have multiple unstake requests across different slots
- Claims are only available after a 10-day waiting period
- Maximum of 10 active date slots at any time
