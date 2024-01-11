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

use nom::{
    branch::alt,
    bytes::complete::{is_not, tag, take_till},
    character::{complete, is_newline},
    combinator::{complete, recognize},
    IResult,
};

struct ConnectionCred {
    rpcconnect: String,
    rpcuser: String,
    rpcpassword: String,
}

pub fn equal_sign(input: &str) -> IResult<&str, char> {
    complete::char('=')(input)
}

pub fn rpcconnect(input: &str) -> IResult<&str, &str> {
    tag("rpcconnect")(input)
}

pub fn rpcuser(input: &str) -> IResult<&str, &str> {
    tag("rpcuser")(input)
}

pub fn rpcpassword(input: &str) -> IResult<&str, &str> {
    tag("rpcpassword")(input)
}

pub fn until_space_or_hash(input: &str) -> IResult<&str, &str> {
    take_till(|c| (c == '\n' || c == '#'))(input)
}

fn not_whitespace(input: &str) -> nom::IResult<&str, &str> {
    is_not(" \t")(input)
}

// -------

pub fn hello_parser(i: &str) -> nom::IResult<&str, &str> {
    nom::bytes::complete::tag("hello")(i)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn thistest() {
        assert!(true);
    }
}
