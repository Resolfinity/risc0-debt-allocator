export COUNTER_ADDRESS=0x9737100D2F42a196DE56ED0d1f6fF598a250E7E4
export RPC=http://localhost:8545
export ETH_WALLET_PRIVATE_KEY=0x9737100D2F42a196DE56ED0d1f6fF598a250E7E4
export BONSAI_API_KEY="8enKC2PnrQ62yVhZqh4fl9OfJgufYZ969wGOOfoA"
export BONSAI_API_URL="https://api.bonsai.xyz/"

cargo run --bin publisher -- \
    --chain-id=1337 \
    --rpc-url=${RPC:?} \
    --contract=${COUNTER_ADDRESS:?} \
    --eth-wallet-private-key=${ETH_WALLET_PRIVATE_KEY:?} \
    --account=0x9737100D2F42a196DE56ED0d1f6fF598a250E7E4