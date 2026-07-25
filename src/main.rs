use md5::Md5;
use sha2::{Sha256, Digest};
use rayon::prelude::*;
use itertools::Itertools;
use std::env;
use std::fs;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::Instant;

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

fn run_dictionary(algo: &str, target_hash: &str, wordlist_path: &str) {
    let content = fs::read_to_string(wordlist_path).expect("Failed to open wordlist file");
    let words: Vec<&str> = content.lines().collect();

    let found = AtomicBool::new(false);
    let counter = AtomicUsize::new(0);
    let start = Instant::now();

    words.par_iter().for_each(|word| {
        if found.load(Ordering::Relaxed) {
            return;
        }
        counter.fetch_add(1, Ordering::Relaxed);
        let computed = compute_hash(algo, word);
        if computed == target_hash {
            found.store(true, Ordering::Relaxed);
            println!("Found: {}", word);
        }
    });

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

        println!("Trying length {}...", len);

        let combos: Vec<String> = std::iter::repeat(charset.iter())
            .take(len)
            .multi_cartesian_product()
            .map(|combo| combo.into_iter().collect::<String>())
            .collect();

        combos.par_iter().for_each(|word| {
            if found.load(Ordering::Relaxed) {
                return;
            }
            counter.fetch_add(1, Ordering::Relaxed);
            let computed = compute_hash(algo, word);
            if computed == target_hash {
                found.store(true, Ordering::Relaxed);
                println!("Found: {}", word);
            }
        });
    }

    let elapsed = start.elapsed().as_secs_f64();
    let attempts = counter.load(Ordering::Relaxed);
    println!("Attempts: {} in {:.3}s ({:.0} hashes/sec)", attempts, elapsed, attempts as f64 / elapsed.max(0.0001));

    if !found.load(Ordering::Relaxed) {
        println!("Not found within given length range");
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        eprintln!("Usage:");
        eprintln!("  Dictionary mode:  {} <hash> <wordlist_path>", args[0]);
        eprintln!("  Bruteforce mode:  {} <hash> --bruteforce <max_length>", args[0]);
        return;
    }

    let target_hash = args[1].to_lowercase();
    let algo = match detect_algo(&target_hash) {
        Some(a) => a,
        None => {
            eprintln!("Unsupported hash length: {}", target_hash.len());
            return;
        }
    };
    println!("Detected algorithm: {}", algo);

    if args[2] == "--bruteforce" {
        let max_len: usize = args.get(3)
            .expect("Provide max length after --bruteforce")
            .parse()
            .expect("Max length must be a number");
        run_bruteforce(algo, &target_hash, max_len);
    } else {
        run_dictionary(algo, &target_hash, &args[2]);
    }
}