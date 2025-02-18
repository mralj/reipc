use std::{env, fmt::Debug, path::Path, str::FromStr, sync::Arc, thread::JoinHandle};

use alloy_primitives::Address;
use alloy_rpc_types_eth::{Block, BlockNumberOrTag, EIP1186AccountProofResponse};
use reipc::RpcProvider;

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        let msg =
            "Usage: reipc <ipc_path> <test_thread_count>or cargo run -- <ipc_pah> <test_thread_count>\nExample: cargo run -- ../nmc.ipc 10";

        panic!("{msg}");
    }

    let limit = if args.len() < 3 {
        4
    } else {
        args[2].parse::<usize>()?
    };

    let rpc_provider = RpcProvider::try_connect(Path::new(&args[1]), None)?;

    let mut jhs = vec![];
    for _ in 0..limit {
        let jh = execute_call_in_thread::<_, Block>(
            rpc_provider.clone(),
            "eth_getBlockByNumber".into(),
            (BlockNumberOrTag::Latest, true),
        );
        jhs.push(jh);
    }

    let jh2 = execute_call_in_thread::<_, EIP1186AccountProofResponse>(
        rpc_provider.clone(),
        "eth_getProof".into(),
        (
            Address::from_str("0xe5cB067E90D5Cd1F8052B83562Ae670bA4A211a8")?,
            (),
            BlockNumberOrTag::Latest,
        ),
    );

    jhs.into_iter().for_each(|jh| {
        if let Err(e) = jh.join().unwrap() {
            println!("{e:?}");
        }
    });
    let _ = jh2.join().unwrap();

    Ok(())
}

fn execute_call_in_thread<Params, Resp>(
    rpc_provider: Arc<RpcProvider>,
    method: String,
    params: Params,
) -> JoinHandle<anyhow::Result<()>>
where
    Params: alloy_json_rpc::RpcSend + 'static,
    Resp: Debug + serde::de::DeserializeOwned,
{
    std::thread::spawn(move || -> anyhow::Result<()> {
        let resp = rpc_provider.call::<Params, Resp>(method, params)?;
        let separator = "===============================================================";
        println!("{:?}\n{separator}\n{separator}", resp);
        Ok(())
    })
}
