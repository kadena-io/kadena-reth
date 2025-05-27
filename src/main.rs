#![allow(missing_docs)]

#[global_allocator]
static ALLOC: reth_cli_util::allocator::Allocator = reth_cli_util::allocator::new_allocator();

use clap::Parser;
use custom_evm::KadenaExecutorBuilder;
use reth::{args::RessArgs, cli::Cli};
use reth_ethereum_cli::chainspec::EthereumChainSpecParser;
use reth_node_ethereum::{node::EthereumAddOns, EthereumNode};
use tracing::info;

pub mod custom_evm;
pub mod kadena_precompiles;

fn main() {
    reth_cli_util::sigsegv_handler::install();

    // Enable backtraces unless a RUST_BACKTRACE value has already been explicitly provided.
    if std::env::var_os("RUST_BACKTRACE").is_none() {
        unsafe { std::env::set_var("RUST_BACKTRACE", "1") };
    }

    if let Err(err) =
        Cli::<EthereumChainSpecParser, RessArgs>::parse().run(|builder, _| async move {
            info!(target: "reth::cli", "Launching node");
            let handle = builder
                .with_types::<EthereumNode>()
                .with_components(EthereumNode::components()
                    .executor(KadenaExecutorBuilder::default())
                )
                .with_add_ons(EthereumAddOns::default())
                .launch_with_debug_capabilities()
                .await?;

            handle.node_exit_future.await
        })
    {
        eprintln!("Error: {err:?}");
        std::process::exit(1);
    }
}

