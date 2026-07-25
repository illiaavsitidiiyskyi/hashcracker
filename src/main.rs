use md5::Md5;
use sha2::{Sha256, Digest};
use rayon::prelude::*;
use itertools::Itertools;
use indicatif::{ProgressBar, ProgressStyle};
use clap::Parser;
use std::fs;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::Instant;

#[derive(Parser)]
#[command(name = "hashcracker")]
#[command(about = "A multithreaded MD5/SHA256 hash cracker", long_about = None)]
struct Cli {
    /// Target hash to crack
    hash: String,

    /// Path to wordlist file (dictionary mode)
    #[arg(short, long)]
    wordlist: Option<String>,

    /// Enable bruteforce mode with given max length
    #[arg(short, long)]
    bruteforce: Option<usize>,
}

fn compute_hash(algo: &str, word: &str) -> String {
    if algo == "md5" {
        let mut hasher = Md5::new();
        hasher.update(word.as_bytes());
        hex::encode(hasher.finalize())
    } else {
        let mut hasher = Sha256::new();
        hasher.update(word.as_bytes());
        hex::encode(hasher.finalize())
    }
}

fn detect_algo(hash: &str) -> Option<&'static str> {
    match hash.len() {
        32 => Some("md5"),
        64 => Some("sha256"),
        _ => None,
    }
}

fn make_progress_bar(total: u64) -> ProgressBar {
    let pb = ProgressBar::new(total);
    pb.set_style(
        ProgressStyle::with_template(
            "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({percent}%) {msg}"
        )
        .unwrap()
        .progress_chars("#>-"),
    );
    pb
}

fn run_dictionary(algo: &str, target_hash: &str, wordlist_path: &str) {
    let content = fs::read_to_string(wordlist_path).expect("Failed to open wordlist file");
    let words: Vec<&str> = content.lines().collect();

    let found = AtomicBool::new(false);
    let counter = AtomicUsize::new(0);
    let start = Instant::now();
    let pb = make_progress_bar(words.len() as u64);

    words.par_iter().for_each(|word| {
        if found.load(Ordering::Relaxed) {
            return;
        }
        counter.fetch_add(1, Ordering::Relaxed);
        pb.inc(1);
        let computed = compute_hash(algo, word);
        if computed == target_hash {
            found.store(true, Ordering::Relaxed);
            pb.set_message(format!("Found: {}", word));
        }
    });

    pb.finish();

    let elapsed = start.elapsed().as_secs_f64();
    let attempts = counter.load(Ordering::Relaxed);
    println!("Attempts: {} in {:.3}s ({:.0} hashes/sec)", attempts, elapsed, attempts as f64 / elapsed.max(0.0001));

    if !found.load(Ordering::Relaxed) {
        println!("Not found in wordlist");
    }
}

fn run_bruteforce(algo: &str, target_hash: &str, max_len: usize) {
    let charset: Vec<char> = "abcdefghijklmnopqrstuvwxyz0123456789".chars().collect();
    let found = AtomicBool::new(false);
    let counter = AtomicUsize::new(0);
    let start = Instant::now();

    for len in 1..=max_len {
        if found.load(Ordering::Relaxed) {
            break;
        }

        let total: u64 = (charset.len() as u64).pow(len as u32);
        println!("Trying length {} ({} combinations)...", len, total);

        let combos: Vec<String> = std::iter::repeat(charset.iter())
            .take(len)
            .multi_cartesian_product()
            .map(|combo| combo.into_iter().collect::<String>())
            .collect();

        let pb = make_progress_bar(combos.len() as u64);

        combos.par_iter().for_each(|word| {
            if found.load(Ordering::Relaxed) {
                return;
            }
            counter.fetch_add(1, Ordering::Relaxed);
            pb.inc(1);
            let computed = compute_hash(algo, word);
            if computed == target_hash {
                found.store(true, Ordering::Relaxed);
                pb.set_message(format!("Found: {}", word));
            }
        });

        pb.finish();
    }

    let elapsed = start.elapsed().as_secs_f64();
    let attempts = counter.load(Ordering::Relaxed);
    println!("Attempts: {} in {:.3}s ({:.0} hashes/sec)", attempts, elapsed, attempts as f64 / elapsed.max(0.0001));

    if !found.load(Ordering::Relaxed) {
        println!("Not found within given length range");
    }
}

fn main() {
    let cli = Cli::parse();
    let target_hash = cli.hash.to_lowercase();

    let algo = match detect_algo(&target_hash) {
        Some(a) => a,
        None => {
            eprintln!("Unsupported hash length: {}", target_hash.len());
            return;
        }
    };
    println!("Detected algorithm: {}", algo);

    if let Some(max_len) = cli.bruteforce {
        run_bruteforce(algo, &target_hash, max_len);
    } else if let Some(wordlist) = cli.wordlist {
        run_dictionary(algo, &target_hash, &wordlist);
    } else {
        eprintln!("Specify either --wordlist <path> or --bruteforce <max_length>");
    }
}