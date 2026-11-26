# HeirSwitch Deployment Guide

## 1. Build Optimized Contract WASM

```bash
./scripts/build-wasm.sh
```

## 2. Deploy to Stellar Testnet

```bash
stellar contract deploy \
  --wasm target/wasm32v1-none/release/heirswitch.wasm \
  --source <OWNER_KEY> \
  --network testnet
```

## 3. Heartbeat & Soroban TTL Renewal

Every `heartbeat()` call resets the inactivity countdown **and** extends both instance and persistent storage TTL (`extend_ttl`) so long-term savings are never archived unexpectedly:

```bash
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source <OWNER_KEY> \
  --network testnet \
  -- heartbeat
```
