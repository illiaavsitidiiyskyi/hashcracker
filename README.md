# hashcracker

A multithreaded MD5/SHA256 hash cracker written in Rust.

## Features

- Dictionary attack (wordlist-based)
- Bruteforce attack (generates all combinations up to a given length)
- Automatic algorithm detection (MD5 or SHA256, based on hash length)
- Multithreaded via `rayon` for fast hashing
- Real-time progress bar with `indicatif`
- Clean CLI powered by `clap`

## Usage

Build the project:
```bash
cargo build --release
```

Dictionary mode:
```bash
cargo run -- <hash> --wordlist wordlist.txt
```

Bruteforce mode:
```bash
cargo run -- <hash> --bruteforce <max_length>
```

Example:
```bash
cargo run -- 5f4dcc3b5aa765d61d8327deb882cf99 --wordlist wordlist.txt
# Detected algorithm: md5
# Found: password
```

## Disclaimer

This tool is intended for educational purposes and authorized security testing only.