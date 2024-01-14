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
    bytes::complete::{tag, take_till},
    character::complete::multispace0,
    combinator::opt,
    IResult,
};

#[derive(Default, Debug)]
pub struct ConnectionCred {
    pub rpcconnect: Option<String>,
    pub rpcuser: Option<String>,
    pub rpcpassword: Option<String>,
}
impl ConnectionCred {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn all_set(&self) -> bool {
        self.rpcconnect.is_some() && self.rpcpassword.is_some() && self.rpcuser.is_some()
    }
}

#[derive(Debug, Eq, PartialEq)]
enum ConfigLine<'a> {
    RpcConnect(&'a str),
    RpcUser(&'a str),
    RpcPassword(&'a str),
    MainSection,
    TestSection,
    Comment(&'a str),
    Other(&'a str),
}

pub type DoneParsing = bool;

pub fn parse_config_file(
    input: &str,
    connection_cred: &mut ConnectionCred,
    should_read: &mut bool,
) -> DoneParsing {
    match config_line(input) {
        Ok(line) => match line.1 {
            ConfigLine::RpcConnect(rpcconnect) => {
                connection_cred.rpcconnect = Some(rpcconnect.to_string())
            }
            ConfigLine::RpcUser(rpcuser) => connection_cred.rpcuser = Some(rpcuser.to_string()),
            ConfigLine::RpcPassword(rpcpassword) => {
                connection_cred.rpcpassword = Some(rpcpassword.to_string())
            }
            ConfigLine::MainSection => *should_read = true,
            ConfigLine::TestSection => *should_read = false,
            ConfigLine::Comment(_) => (),
            ConfigLine::Other(_) => (),
        },
        Result::Err(err) => panic!("config_line panic: {}", err),
    }

    connection_cred.all_set()
}

fn config_line(input: &str) -> IResult<&str, ConfigLine> {
    // Consume whitespace from the beginning of input
    let (input, _prefix_whitespace) = multispace0(input)?;

    // Parse out [main], [test], and comment lines (#)
    let (input, maybe_main) = main_section(input)?;
    if let Some(_main) = maybe_main {
        return Ok(("", ConfigLine::MainSection));
    }
    let (input, maybe_test) = test_section(input)?;
    if let Some(_test) = maybe_test {
        return Ok(("", ConfigLine::TestSection));
    }
    let (input, maybe_comment) = comment(input)?;
    if let Some(comment) = maybe_comment {
        return Ok(("", ConfigLine::Comment(comment)));
    }

    let (input, maybe_rpcuser_line) = rpcuser_line(input)?;
    if let Some(_rpcuser_line) = maybe_rpcuser_line {
        let (_, rpcuser) = take_till_space_or_newline(input)?;
        return Ok(("", ConfigLine::RpcUser(rpcuser)));
    }
    let (input, maybe_rpcpassword_line) = rpcpassword_line(input)?;
    if let Some(_rpcpassword_line) = maybe_rpcpassword_line {
        let (_, rpcpassword) = take_till_space_or_newline(input)?;
        return Ok(("", ConfigLine::RpcPassword(rpcpassword)));
    }
    let (input, maybe_rpcconnect_line) = rpcconnect_line(input)?;
    if let Some(_rpcconnect_line) = maybe_rpcconnect_line {
        let (_, rpcconnect) = take_till_space_or_newline(input)?;
        return Ok(("", ConfigLine::RpcConnect(rpcconnect)));
    }

    Ok((input, ConfigLine::Other("")))
}

fn comment(input: &str) -> IResult<&str, Option<&str>> {
    opt(tag("#"))(input)
}

fn main_section(input: &str) -> IResult<&str, Option<&str>> {
    opt(tag("[main]"))(input)
}

fn test_section(input: &str) -> IResult<&str, Option<&str>> {
    opt(tag("[test]"))(input)
}

fn rpcconnect_line(input: &str) -> IResult<&str, Option<&str>> {
    opt(tag("rpcconnect="))(input)
}

fn rpcuser_line(input: &str) -> IResult<&str, Option<&str>> {
    opt(tag("rpcuser="))(input)
}

fn rpcpassword_line(input: &str) -> IResult<&str, Option<&str>> {
    opt(tag("rpcpassword="))(input)
}

fn take_till_space_or_newline(input: &str) -> IResult<&str, &str> {
    take_till(|c| (c == '\n' || c == ' ' || c == '\t'))(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn config_line_spaces() {
        assert_eq!(
            Ok(("", ConfigLine::RpcConnect("127.0.0.1:8333"))),
            config_line("  rpcconnect=127.0.0.1:8333")
        )
    }

    #[test]
    pub fn config_line_no_spaces() {
        assert_eq!(
            Ok(("", ConfigLine::RpcConnect("127.0.0.1:8333"))),
            config_line("rpcconnect=127.0.0.1:8333")
        )
    }

    #[test]
    pub fn config_line_rpcuser() {
        assert_eq!(
            Ok(("", ConfigLine::RpcUser("myuser"))),
            config_line("rpcuser=myuser")
        )
    }

    #[test]
    pub fn config_line_rpcpass() {
        assert_eq!(
            Ok(("", ConfigLine::RpcPassword("Wjj8**#llZ?"))),
            config_line("rpcpassword=Wjj8**#llZ?")
        )
    }

    #[test]
    pub fn config_line_main_section() {
        assert_eq!(Ok(("", ConfigLine::MainSection)), config_line("  [main]"))
    }

    #[test]
    pub fn config_line_test_section() {
        assert_eq!(Ok(("", ConfigLine::TestSection)), config_line("  [test]"))
    }
}
