// Copyright 2024 RISC Zero, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

// This application demonstrates how to send an off-chain proof request
// to the Bonsai proving service and publish the received proofs directly
// to your deployed app contract.

use alloy_primitives::{address, Address};
use alloy_sol_types::{sol, SolCall, SolInterface, SolValue};
use anyhow::{Context, Result};
use apps::{BonsaiProver, TxSender};
use clap::Parser;
use methods::IS_EVEN_ELF;
use risc0_steel::{config::ETH_SEPOLIA_CHAIN_SPEC, ethereum::EthViewCallEnv, EvmHeader, ViewCall};
use risc0_zkvm::serde::to_vec;

const CONTRACT: Address = address!("dAC17F958D2ee523a2206206994597C13D831ec7");
const HOLDER: Address = address!("f977814e90da44bfa03b6295a0616a897441acec");

sol! {
    /// ERC-20 balance function signature.
    interface IERC20 {
        function balanceOf(address account) external view returns (uint);
    }
}

// `ICounter` interface automatically generated via the alloy `sol!` macro.
sol! {
    interface ICounter {
        function increment(bytes calldata journal, bytes32 post_state_digest, bytes calldata seal);
    }
}

/// Arguments of the publisher CLI.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Ethereum chain ID
    #[clap(long)]
    chain_id: u64,

    /// Ethereum Node endpoint.
    #[clap(long, env)]
    eth_wallet_private_key: String,

    /// Ethereum Node endpoint.
    #[clap(long)]
    rpc_url: String,

    /// Application's contract address on Ethereum
    #[clap(long)]
    contract: String,

    /// Account address to read the balance_of on Ethereum
    #[clap(long)]
    account: Address,
}

fn main() -> Result<()> {
    env_logger::init();
    let args = Args::parse();

    // Create a new `TxSender`.
    // let tx_sender = TxSender::new(
    //     args.chain_id,
    //     &args.rpc_url,
    //     &args.eth_wallet_private_key,
    //     &args.contract,
    // )?;

    let env =
        EthViewCallEnv::from_rpc(&args.rpc_url, None)?.with_chain_spec(&ETH_SEPOLIA_CHAIN_SPEC);
    let number = env.header().number();

    // Function to call
    let account = HOLDER; // args.account;
    let call = IERC20::balanceOfCall { account };

    // Preflight the view call to construct the input that is required to execute the function in
    // the guest. It also returns the result of the call.
    let (view_call_input, returns) = ViewCall::new(call, CONTRACT).preflight(env)?;
    println!(
        "For block {} `{}` returns: {}",
        number,
        IERC20::balanceOfCall::SIGNATURE,
        returns._0
    );

    // Send an off-chain proof request to the Bonsai proving service.
    let input = InputBuilder::new()
        .write(view_call_input)
        .unwrap()
        .write(account)
        .unwrap()
        .bytes();

    let (journal, post_state_digest, seal) = BonsaiProver::prove(IS_EVEN_ELF, &input)?;

    // // Encode the function call for `ICounter.increment(journal, post_state_digest, seal)`.
    // let calldata = ICounter::ICounterCalls::increment(ICounter::incrementCall {
    //     journal,
    //     post_state_digest,
    //     seal,
    // })
    // .abi_encode();

    // // Send the calldata to Ethereum.
    // let runtime = tokio::runtime::Runtime::new()?;
    // runtime.block_on(tx_sender.send(calldata))?;

    Ok(())
}

pub struct InputBuilder {
    input: Vec<u32>,
}

impl InputBuilder {
    pub fn new() -> Self {
        InputBuilder { input: Vec::new() }
    }

    pub fn write(mut self, input: impl serde::Serialize) -> Result<Self> {
        self.input.extend(to_vec(&input)?);
        Ok(self)
    }

    pub fn bytes(self) -> Vec<u8> {
        bytemuck::cast_slice(&self.input).to_vec()
    }
}
