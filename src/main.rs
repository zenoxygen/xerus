// Copyright (c) 2020 zenoxygen
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

#[macro_use]
extern crate log;

mod args;
mod client;
mod handshake;
mod message;
mod peer;
mod piece;
mod torrent;
mod worker;

use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};

use args::parse_args;
use torrent::*;

fn run(args: clap::ArgMatches) -> Result<()> {
    let torrent = args.value_of("torrent").unwrap();
    let file = args.value_of("file").unwrap();

    // Check if torrent file exists
    if Path::new(&torrent).exists() {
        let torrent_filepath = PathBuf::from(torrent);
        let output_filepath = PathBuf::from(file);

        // Create new file
        let mut output_file = match File::create(output_filepath) {
            Ok(file) => file,
            Err(_) => return Err(anyhow!("could not create file")),
        };

        // Open and download torrent
        let mut torrent = Torrent::new();
        torrent.open(torrent_filepath)?;
        let data: Vec<u8> = torrent.download()?;

        // Save data to file
        if output_file.write(&data).is_err() {
            return Err(anyhow!("could not write data to file"));
        }
    } else {
        return Err(anyhow!("could not find torrent"));
    }

    println!("Saved in {:?}.", file);

    Ok(())
}

fn main() {
    // Initialize logger
    pretty_env_logger::init_timed();

    // Parse arguments
    let args = parse_args();

    // Run program, eventually exit failure
    if let Err(error) = run(args) {
        eprintln!("Error: {}", error);
        std::process::exit(1);
    }

    // Exit success
    std::process::exit(0);
}
