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

use bitcoin::{
    secp256k1::Secp256k1,
    sign_message::{signed_msg_hash, MessageSignature},
    Address, Network,
};
use bitcoincore_rpc::{bitcoin, Auth, Client, RpcApi};
use dirs::home_dir;
use std::{
    fs::File,
    io::{BufRead, BufReader},
    str::FromStr,
};

use crate::parsers::{parse_config_file, ConnectionCred};
mod parsers;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let address: Address = Address::from_str("1E9YwDtYf9R29ekNAfbV7MvB4LNv7v3fGa")
        .unwrap()
        .require_network(Network::Bitcoin)
        .unwrap();
    let address_two: Address = Address::from_str("1NChfewU45oy7Dgn51HwkBFSixaTnyakfj")
        .unwrap()
        .require_network(Network::Bitcoin)
        .unwrap();

    let secp_ctx = Secp256k1::new();
    let signature_str = MessageSignature::from_str(
        "HCsBcgB+Wcm8kOGMH8IpNeg0H4gjCrlqwDf/GlSXphZGBYxm0QkKEPhh9DTJRp2IDNUhVr0FhP9qCqo2W0recNM=",
    )?;
    let message_hash = signed_msg_hash("1E9YwDtYf9R29ekNAfbV7MvB4LNv7v3fGa");

    let is_signed = signature_str.is_signed_by_address(&secp_ctx, &address, message_hash)?;
    let is_signed_two =
        signature_str.is_signed_by_address(&secp_ctx, &address_two, message_hash)?;
    println!(
        "Was {}, signed by address {}? Answer: {}",
        signature_str, address, is_signed
    );
    println!(
        "Was {}, signed by address {}? Answer: {}",
        signature_str, address_two, is_signed_two
    );

    // Recover public key and then check that.
    let recovered_pubkey = signature_str.recover_pubkey(&secp_ctx, message_hash)?;
    let recovered_address = Address::p2pkh(&recovered_pubkey, Network::Bitcoin);
    println!("The recovered address: {}", recovered_address);
    let recovered_is_good =
        signature_str.is_signed_by_address(&secp_ctx, &recovered_address, message_hash)?;

    println!(
        "Was {}, signed by address: {}? Answer: {}",
        signature_str, recovered_address, recovered_is_good
    );
    // ------------------

    let ctx = Secp256k1::new();
    let sig = MessageSignature::from_str(
        "HCsBcgB+Wcm8kOGMH8IpNeg0H4gjCrlqwDf/GlSXphZGBYxm0QkKEPhh9DTJRp2IDNUhVr0FhP9qCqo2W0recNM=",
    )?;
    println!("sig: {:?}", sig.signature);
    let address = Address::from_str("1E9YwDtYf9R29ekNAfbV7MvB4LNv7v3fGa")?
        .require_network(Network::Bitcoin)?;
    let message = "1E9YwDtYf9R29ekNAfbV7MvB4LNv7v3fGa";
    let msg_hash = signed_msg_hash(message);
    let is_good_sig = sig.is_signed_by_address(&ctx, &address, msg_hash)?;
    println!("{} signed the message: {}", address, is_good_sig);

    let recovered_pubkey = sig.recover_pubkey(&ctx, msg_hash)?;
    let recovered_address = Address::p2pkh(&recovered_pubkey, Network::Bitcoin);

    let recovered_is_good = sig.is_signed_by_address(&ctx, &recovered_address, msg_hash)?;
    println!(
        "{} signed the message: {}",
        recovered_address, recovered_is_good
    );

    let sig = MessageSignature::from_str(
        "HCsBcgB+Wcm8kOGMH8IpNeg0H4gjCrlqwDf/GlSXphZGBYxm0QkKEPhh9DTJRp2IDNUhVr0FhP9qCqo2W0recNM=",
    )?;
    let signature = sig.signature.to_standard();

    // ----------

    // ----------

    let mut args = std::env::args();

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
    if !connection_cred.all_set() {
        panic!("Could not read credentials: {:?}", connection_cred)
    }

    let _exe_name = args.next().unwrap();

    let rpc = Client::new(
        &connection_cred.rpcconnect.expect("a connection"),
        Auth::UserPass(
            connection_cred.rpcuser.expect("a user"),
            connection_cred.rpcpassword.expect("a password"),
        ),
    )
    .expect("a client");

    let x = rpc.get_block_hash(123456)?;
    println!("{:?}", x);

    let y = rpc.verify_message(&address, &signature, message)?;
    println!("y: {}", y);
    Ok(())
}
