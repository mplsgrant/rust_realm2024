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

//#![warn(clippy::pedantic)]

use crate::parsers::{parse_config_file, ConnectionCred};
use bitcoin::{
    hashes::sha256d,
    secp256k1::Secp256k1,
    sign_message::{signed_msg_hash, MessageSignature},
    Address, Network, PublicKey,
};
use bitcoincore_rpc::{bitcoin, Auth, Client, RpcApi};
use clap::{Parser, Subcommand};
use dirs::home_dir;
use std::path::PathBuf;
use std::{
    fs::File,
    io::{BufRead, BufReader},
    str::FromStr,
};
mod parsers;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Optional name to operate on
    name: Option<String>,

    /// Sets a custom config file
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,

    /// Turn debugging information on
    #[arg(short, long, action = clap::ArgAction::Count)]
    debug: u8,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// does testing things
    VerifyMessage {
        address: String,
        signature: String,
        message: String,
    },
    VerifyMessageRecovery {
        signature: String,
        message: String,
    },
    GetBlockHash {
        block: u64,
    },
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
            Commands::GetBlockHash { block } => {
                let hash = rpc.get_block_hash(block)?;
                println!("{hash}");
            }
        }
    }

    Ok(())
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
