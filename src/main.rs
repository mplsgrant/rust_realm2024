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
use std::{fs::File, io::Read, str::FromStr};
mod parsers;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ctx = Secp256k1::new();
    let sig = MessageSignature::from_str(
        "HCsBcgB+Wcm8kOGMH8IpNeg0H4gjCrlqwDf/GlSXphZGBYxm0QkKEPhh9DTJRp2IDNUhVr0FhP9qCqo2W0recNM=",
    )?;
    println!("sig: {:?}", sig.signature);
    let address = Address::from_str("1E9YwDtYf9R29ekNAfbV7MvB4LNv7v3fGa")?
        .require_network(Network::Bitcoin)?;
    let msg_hash = signed_msg_hash("1E9YwDtYf9R29ekNAfbV7MvB4LNv7v3fGa");
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

    let mut file = if let Ok(file) = File::open("bitcoin.conf") {
        file
    } else if let Ok(file) = File::open(&homedir) {
        file
    } else {
        panic!("Could read neither bitcoin.conf nor ~/.bitcoin/bitcoin.conf ")
    };

    let mut config_file = String::new();
    file.read_to_string(&mut config_file)?;

    let _exe_name = args.next().unwrap();

    let url = args.next().expect("Usage: <rpc_url> <username> <password>");
    let user = args.next().expect("no user given");
    let pass = args.next().expect("no pass given");

    let rpc = Client::new(&url, Auth::UserPass(user, pass)).unwrap();

    let x = rpc.get_block_hash(123456)?;
    println!("{:?}", x);
    Ok(())
}
