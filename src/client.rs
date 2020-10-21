extern crate anyhow;
extern crate url;

use crate::peers::*;

use anyhow::{anyhow, Result};
use url::Url;

use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::time::Duration;

/// Client structure.
#[derive(Clone)]
pub struct Client {
    pub ip: Ipv4Addr,
    pub port: u16,
}

impl Client {
    pub fn new(peer: &Peer, peer_id: Vec<u8>, info_hash: Vec<u8>) -> Result<Peer> {}
}
