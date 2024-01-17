// Copyright (C) 2024      Whittier Digital Technologies LLC
//
// This file is part of rust_realm2024.
//
// rust_realm2024 is free software: you can redistribute it and/or modify it under the terms of the
// GNU General Public License as published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// rust_realm2024 is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY;
// without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License along with Foobar. If not, see
// <https://www.gnu.org/licenses/>.

#![warn(clippy::pedantic)]

use crate::parsers::{parse_config_file, ConnectionCred};
use bitcoin::{
    hashes::sha256d,
    secp256k1::Secp256k1,
    sign_message::{signed_msg_hash, MessageSignature},
    Address, Network, PublicKey, Txid, Witness,
};
use bitcoincore_rpc::{bitcoin, Auth, Client, RpcApi};
use clap::{Parser, Subcommand};
use dirs::home_dir;
use miniscript::{Descriptor, DescriptorPublicKey};
use std::{
    fs::File,
    io::{BufRead, BufReader},
    str::FromStr,
};
mod parsers;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Verify a message
    VerifyMessage {
        address: String,
        signature: String,
        message: String,
    },
    /// Get the address that signed a message
    VerifyMessageRecovery { signature: String, message: String },
    /// Get the hash of a block
    GetBlockHash { height: u64 },
    /// Get the number of vouts for a block at a given  height
    GetBlockOuts { height: u64 },
    /// Derive an address given a descriptor
    DeriveAddress { descriptor: String },
    /// Problem 005
    Do005,
    /// Problem 006
    Do006,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut homedir = home_dir().expect("Could not determine user's home directory");
    homedir.push(".bitcoin/bitcoin.conf");

    let file = if let Ok(file) = File::open("bitcoin.conf") {
        file
    } else if let Ok(file) = File::open(&homedir) {
        file
    } else {
        panic!("Could read neither bitcoin.conf nor ~/.bitcoin/bitcoin.conf ")
    };

    let mut should_read = true; // State to distinguish default, main, and test sections
    let mut connection_cred = ConnectionCred::new();
    for line in BufReader::new(file).lines().flatten() {
        let done_parsing = parse_config_file(&line, &mut connection_cred, &mut should_read);
        if done_parsing {
            break;
        }
    }
    assert!(
        connection_cred.all_set(),
        "Could not read credentials: {connection_cred:?}"
    );

    let rpc = Client::new(
        &connection_cred.rpcconnect.expect("a connection"),
        Auth::UserPass(
            connection_cred.rpcuser.expect("a user"),
            connection_cred.rpcpassword.expect("a password"),
        ),
    )
    .expect("a client");

    let cli = Cli::parse();

    if let Some(command) = cli.command {
        match command {
            Commands::VerifyMessage {
                address,
                signature,
                message,
            } => {
                let is_verified = verify_message_from_string(&address, &signature, &message)?;
                println!("{is_verified}");
            }
            Commands::VerifyMessageRecovery { signature, message } => {
                let pubkey = verify_message_recover_from_string(&signature, &message)?;
                let recovered_address = Address::p2pkh(&pubkey, Network::Bitcoin);
                println!("{recovered_address}");
            }
            Commands::GetBlockHash { height } => {
                let hash = rpc.get_block_hash(height)?;
                println!("{hash}");
            }
            Commands::GetBlockOuts { height } => {
                let outs = get_block_outs(height, &rpc)?;
                println!("{outs}");
            }
            Commands::DeriveAddress { descriptor } => {
                let descriptor = Descriptor::<DescriptorPublicKey>::from_str(&descriptor)?;
                let index_0 = descriptor.at_derivation_index(0)?;
                let address = index_0.address(Network::Bitcoin)?;
                println!("{address}");
            }
            Commands::Do005 => {
                let _ = do_005(&rpc);
            }
            Commands::Do006 => {
                let _ = do_006(&rpc);
            }
        }
    }

    Ok(())
}

fn do_006(rpc: &Client) -> Result<(), Box<dyn std::error::Error>> {
    let coinbase_hash = rpc.get_block_hash(256_128)?;
    let coinbase_txid = rpc.get_block_info(&coinbase_hash)?.tx[0];
    let block_hash = rpc.get_block_hash(257_343)?;
    let block = rpc.get_block_txs(&block_hash)?;
    block
        .tx
        .iter()
        .map(|tx| {
            tx.vin
                .iter()
                .map(|vin| {
                    if let Some(txid) = vin.txid {
                        if txid == coinbase_txid {
                            println!("{}", tx.txid);
                        }
                    }
                })
                .for_each(drop);
        })
        .for_each(drop);
    Ok(())
}

fn do_005(client: &Client) -> Result<(), Box<dyn std::error::Error>> {
    let txid = Txid::from_str("37d966a263350fe747f1c606b159987545844a493dd38d84b070027a895c4517")?;
    let tx = client.get_raw_transaction_info(&txid, None)?;
    let x = tx
        .vin
        .into_iter()
        .filter_map(|vin| {
            vin.txinwitness
                .as_ref()
                .map(|txinwitness| Witness::from_slice(txinwitness))
        })
        .map(|witness| {
            let x = witness.nth(1).expect("a value");
            let mut vals = String::new();
            for i in x {
                vals.push_str(&format!("{i:02x}"));
            }
            vals
        })
        .collect::<Vec<String>>();
    let s = format!("sh(multi(1,{},{},{},{}))", x[0], x[1], x[2], x[3]);
    let descriptor = miniscript::Descriptor::<bitcoin::PublicKey>::from_str(&s).unwrap();
    let address = descriptor.address(Network::Bitcoin)?;
    println!("{address}");
    Ok(())
}

fn get_block_outs(height: u64, client: &Client) -> Result<usize, Box<dyn std::error::Error>> {
    Ok(client.get_block_stats(height)?.outs)
}

fn verify_message_recover_from_string(
    signature: &str,
    message: &str,
) -> Result<PublicKey, Box<dyn std::error::Error>> {
    let signature = MessageSignature::from_str(signature)?;
    let msg_hash = signed_msg_hash(message);
    let secp_ctx = Secp256k1::new();
    Ok(signature.recover_pubkey(&secp_ctx, msg_hash)?)
}

fn verify_message_from_string(
    address: &str,
    signature: &str,
    message: &str,
) -> Result<bool, Box<dyn std::error::Error>> {
    let address: Address = Address::from_str(address)
        .unwrap()
        .require_network(Network::Bitcoin)
        .unwrap();
    let signature = MessageSignature::from_str(signature)?;
    let msg_hash = signed_msg_hash(message);
    verify_message(&address, signature, msg_hash)
}

fn verify_message(
    address: &Address,
    signature: MessageSignature,
    msg_hash: sha256d::Hash,
) -> Result<bool, Box<dyn std::error::Error>> {
    let secp_ctx = Secp256k1::new();
    Ok(signature.is_signed_by_address(&secp_ctx, address, msg_hash)?)
}
