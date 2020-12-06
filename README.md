# xerus

[![Build Status](https://gitlab.com/zenoxygen/xerus/badges/master/pipeline.svg)](https://gitlab.com/zenoxygen/xerus/pipelines)
[![Crates.io](https://img.shields.io/crates/v/xerus.svg)](https://crates.io/crates/xerus)
[![Docs](https://docs.rs/xerus/badge.svg)](https://docs.rs/xerus)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

A command-line BitTorrent client, written in Rust.

## Usage

```
xerus 0.1.0
zenoxygen <zenoxygen@protonmail.com>
A command-line BitTorrent client, written in Rust.

USAGE:
    xerus -f <file> -t <torrent>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -f <file>           The path where to save the file
    -t <torrent>        The path to the torrent
```

## Example

Try to download an official Debian ISO image:

```
$> ./xerus -f debian-10.7.0-amd64-netinst.iso -t debian-10.7.0-amd64-netinst.iso.torrent
Downloading "debian-10.7.0-amd64-netinst.iso" (1344 pieces)
Saved in "debian-10.7.0-amd64-netinst.iso".
```

And verify the checksum matches that expected from the checksum file:

```
$> sha512sum -c SHA512SUM | grep debian-10.7.0-amd64-netinst.iso
debian-10.7.0-amd64-netinst.iso: OK
```

## Debug

Run with the environment variable set:

```
$> RUST_LOG=trace ./xerus -f <file> -t <torrent>
```

## Documentation

Learn more here: [https://docs.rs/xerus](https://docs.rs/xerus).

## License

Xerus is distributed under the terms of the [MIT License](LICENSE).
