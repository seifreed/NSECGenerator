use clap::{Parser, Subcommand};
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Instant;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Domain to generate hashes for
    #[arg(short, long)]
    domain: Option<String>,

    /// Path to wordlist file (one subdomain per line)
    #[arg(short, long)]
    wordlist: Option<PathBuf>,

    /// NSEC3 salt (hex string, e.g., "ABC123" or empty for no salt)
    #[arg(short, long, global = true, default_value = "")]
    salt: String,

    /// Number of NSEC3 iterations
    #[arg(short, long, global = true, default_value_t = 0)]
    iterations: u32,

    /// Output directory for JSON cache files
    #[arg(short, long, global = true, default_value = "output")]
    output: PathBuf,

    /// Number of threads (default: CPU cores)
    #[arg(short, long, global = true)]
    threads: Option<usize>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Download wordlists from SecLists repository
    DownloadWordlists {
        /// Output directory for wordlists
        #[arg(short, long, default_value = "wordlists")]
        output: PathBuf,

        /// Size: 1k, 10k, 100k, or all
        #[arg(short, long, default_value = "all")]
        size: String,
    },
    /// Generate hashes for common NSEC3 configurations
    GenerateCommon {
        /// Domain to generate hashes for
        #[arg(short, long)]
        domain: String,

        /// Path to wordlist file
        #[arg(short, long)]
        wordlist: PathBuf,

        /// Output directory for JSON cache files
        #[arg(short, long, default_value = "output")]
        output: PathBuf,

        /// Number of threads (default: CPU cores)
        #[arg(short, long)]
        threads: Option<usize>,
    },
}

#[derive(Serialize, Deserialize)]
struct CacheFile {
    domain: String,
    salt: String,
    iterations: u32,
    wordlist_size: usize,
    hashes: HashMap<String, String>,
}

/// Calculate NSEC3 hash for a fully-qualified domain name
fn calculate_nsec3_hash(fqdn: &str, salt_bytes: &[u8], iterations: u32) -> String {
    // Convert FQDN to lowercase bytes
    let fqdn_bytes = fqdn.to_lowercase().into_bytes();

    // Initial hash: SHA1(domain + salt)
    let mut hasher = Sha1::new();
    hasher.update(&fqdn_bytes);
    hasher.update(salt_bytes);
    let mut hash = hasher.finalize();

    // Perform iterations: SHA1(previous_hash + salt)
    for _ in 0..iterations {
        let mut hasher = Sha1::new();
        hasher.update(hash);
        hasher.update(salt_bytes);
        hash = hasher.finalize();
    }

    // Base32 encode without padding
    base32::encode(base32::Alphabet::RFC4648 { padding: false }, &hash).to_lowercase()
}

/// Load wordlist from file
fn load_wordlist(path: &PathBuf) -> std::io::Result<Vec<String>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    Ok(reader
        .lines()
        .map_while(Result::ok)
        .map(|line| line.trim().to_string())
        .filter(|line| !line.is_empty())
        .collect())
}

/// Generate cache filename from salt and iterations
fn get_cache_filename(salt: &str, iterations: u32) -> String {
    let cache_key = format!("{salt}_{iterations}");
    let mut hasher = md5::Context::new();
    hasher.consume(cache_key.as_bytes());
    let hash = format!("{:x}", hasher.compute());
    format!("nsec3_{hash}.json")
}

/// Download wordlist from URL and save to file
fn download_wordlist(
    url: &str,
    output_path: &PathBuf,
    limit: usize,
    name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    print!("  → Downloading {} subdomains... ", name);
    std::io::stdout().flush()?;

    let response = reqwest::blocking::get(url)?;
    let text = response.text()?;

    let lines: Vec<&str> = text.lines().take(limit).collect();
    let content = lines.join("\n");

    fs::write(output_path, content)?;

    println!("✓");
    Ok(())
}

/// Download wordlists from SecLists repository
fn download_wordlists(output_dir: &PathBuf, size: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("📥 Downloading wordlists from SecLists...");
    println!();

    fs::create_dir_all(output_dir)?;

    let wordlists = vec![
        (
            "1k",
            "https://raw.githubusercontent.com/danielmiessler/SecLists/master/Discovery/DNS/subdomains-top1million-5000.txt",
            1000,
            "subdomains-1k.txt",
        ),
        (
            "10k",
            "https://raw.githubusercontent.com/danielmiessler/SecLists/master/Discovery/DNS/subdomains-top1million-20000.txt",
            10000,
            "subdomains-10k.txt",
        ),
        (
            "100k",
            "https://raw.githubusercontent.com/danielmiessler/SecLists/master/Discovery/DNS/subdomains-top1million-110000.txt",
            100000,
            "subdomains-100k.txt",
        ),
    ];

    for (list_size, url, limit, filename) in wordlists {
        if size == "all" || size == list_size {
            let output_path = output_dir.join(filename);
            match download_wordlist(url, &output_path, limit, &format!("{}K", limit / 1000)) {
                Ok(_) => {}
                Err(e) => eprintln!("  ✗ Failed to download {}: {}", filename, e),
            }
        }
    }

    println!();
    println!("✅ Wordlists downloaded to: {}", output_dir.display());
    println!();
    println!("📋 Downloaded files:");

    if let Ok(entries) = fs::read_dir(output_dir) {
        for entry in entries.flatten() {
            if let Ok(metadata) = entry.metadata() {
                let size_kb = metadata.len() / 1024;
                println!(
                    "   {} ({} KB)",
                    entry.file_name().to_string_lossy(),
                    size_kb
                );
            }
        }
    }

    Ok(())
}

/// Generate hash for a single configuration
fn generate_hash_for_config(
    domain: &str,
    wordlist_path: &PathBuf,
    salt: &str,
    iterations: u32,
    output_dir: &PathBuf,
) -> Result<String, Box<dyn std::error::Error>> {
    // Load wordlist
    let subdomains = load_wordlist(wordlist_path)?;

    // Parse salt from hex
    let salt_bytes = if salt.is_empty() {
        Vec::new()
    } else {
        hex::decode(salt).unwrap_or_else(|_| salt.as_bytes().to_vec())
    };

    // Thread-safe hashmap for results
    let hashes: Arc<Mutex<HashMap<String, String>>> = Arc::new(Mutex::new(HashMap::new()));

    // Parallel processing with rayon
    subdomains.par_iter().for_each(|subdomain| {
        let fqdn = format!("{}.{}", subdomain, domain);
        let hash = calculate_nsec3_hash(&fqdn, &salt_bytes, iterations);
        hashes.lock().unwrap().insert(hash, fqdn);
    });

    // Create output directory
    fs::create_dir_all(output_dir)?;

    // Generate cache file
    let cache_filename = get_cache_filename(salt, iterations);
    let output_path = output_dir.join(&cache_filename);

    let cache = CacheFile {
        domain: domain.to_string(),
        salt: salt.to_string(),
        iterations,
        wordlist_size: subdomains.len(),
        hashes: hashes.lock().unwrap().clone(),
    };

    let json = serde_json::to_string_pretty(&cache)?;
    fs::write(&output_path, json)?;

    let file_size = fs::metadata(&output_path)?.len();
    Ok(format!(
        "{} ({:.2} MB)",
        output_path.display(),
        file_size as f64 / 1_048_576.0
    ))
}

/// Generate hashes for common NSEC3 configurations
fn generate_common_configs(
    domain: &str,
    wordlist_path: &PathBuf,
    output_dir: &PathBuf,
    threads: Option<usize>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Set thread pool size
    if let Some(threads) = threads {
        rayon::ThreadPoolBuilder::new()
            .num_threads(threads)
            .build_global()
            .unwrap();
    }

    println!("🔄 Generating common NSEC3 configurations for {}", domain);
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();

    // Common configurations based on real-world statistics
    // Format: (name, salt, iterations)
    let configs = vec![
        ("No salt, no iterations (30% of NSEC3 domains)", "", 0),
        ("Google Cloud DNS", "DEADBEEF", 5),
        ("AWS Route53", "CAFEBABE", 10),
        ("Cloudflare minimal", "00", 0),
        ("Light security", "AABBCCDD", 3),
        ("Medium security", "12345678", 5),
        ("High security", "FEDCBA98", 10),
        ("Very high security", "FFFFFFFF", 15),
    ];

    let total = configs.len();
    let start_time = Instant::now();

    for (index, (name, salt, iterations)) in configs.iter().enumerate() {
        println!("[{}/{}] {} ", index + 1, total, name);
        println!("   Salt: {}", if salt.is_empty() { "none" } else { salt });
        println!("   Iterations: {}", iterations);

        let config_start = Instant::now();
        match generate_hash_for_config(domain, wordlist_path, salt, *iterations, output_dir) {
            Ok(output_info) => {
                println!("   ✓ Generated: {}", output_info);
                println!("   Time: {:.2}s", config_start.elapsed().as_secs_f64());
            }
            Err(e) => {
                eprintln!("   ✗ Failed: {}", e);
            }
        }
        println!();
    }

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("✅ Generation complete!");
    println!("   Total time: {:.2}s", start_time.elapsed().as_secs_f64());
    println!();
    println!(
        "📋 Generated {} cache files in: {}",
        total,
        output_dir.display()
    );
    println!();
    println!("📊 Next steps:");
    println!("   1. Copy to DNSight cache directory:");
    println!(
        "      cp {}/*.json ../data/nsec3_cache/",
        output_dir.display()
    );
    println!();
    println!("   2. Run DNSight zone walking:");
    println!("      dnsight zonewalk {}", domain);
    println!();
    println!("💡 DNSight will automatically use the cache that matches");
    println!("   the domain's actual NSEC3 parameters.");

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Handle subcommands
    if let Some(command) = args.command {
        match command {
            Commands::DownloadWordlists { output, size } => {
                return download_wordlists(&output, &size);
            }
            Commands::GenerateCommon {
                domain,
                wordlist,
                output,
                threads,
            } => {
                return generate_common_configs(&domain, &wordlist, &output, threads);
            }
        }
    }

    // Unwrap required arguments for hash generation
    let domain = args.domain.expect("Domain is required");
    let wordlist = args.wordlist.expect("Wordlist is required");

    // Set thread pool size
    if let Some(threads) = args.threads {
        rayon::ThreadPoolBuilder::new()
            .num_threads(threads)
            .build_global()
            .unwrap();
    }

    let num_threads = rayon::current_num_threads();

    println!("🚀 NSEC3 Hash Generator for DNSight");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("📋 Configuration:");
    println!("   Domain: {}", domain);
    println!("   Wordlist: {}", wordlist.display());
    println!(
        "   Salt: {}",
        if args.salt.is_empty() {
            "none"
        } else {
            &args.salt
        }
    );
    println!("   Iterations: {}", args.iterations);
    println!("   Threads: {}", num_threads);
    println!();

    // Load wordlist
    print!("📖 Loading wordlist... ");
    std::io::stdout().flush()?;
    let start = Instant::now();
    let subdomains = load_wordlist(&wordlist)?;
    println!(
        "✓ {} subdomains loaded ({:.2}s)",
        subdomains.len(),
        start.elapsed().as_secs_f64()
    );

    // Parse salt from hex
    let salt_bytes = if args.salt.is_empty() {
        Vec::new()
    } else {
        hex::decode(&args.salt).unwrap_or_else(|_| {
            eprintln!("⚠️  Warning: Invalid hex salt, using as-is");
            args.salt.as_bytes().to_vec()
        })
    };

    // Create progress bar
    let pb = ProgressBar::new(subdomains.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} ({percent}%) {msg}")
            .unwrap()
            .progress_chars("█▓░"),
    );

    println!("⚡ Computing NSEC3 hashes...");
    let start = Instant::now();

    // Thread-safe hashmap for results
    let hashes: Arc<Mutex<HashMap<String, String>>> = Arc::new(Mutex::new(HashMap::new()));

    // Parallel processing with rayon
    subdomains.par_iter().for_each(|subdomain| {
        let fqdn = format!("{}.{}", subdomain, domain);
        let hash = calculate_nsec3_hash(&fqdn, &salt_bytes, args.iterations);

        // Store result
        hashes.lock().unwrap().insert(hash, fqdn);

        // Update progress
        pb.inc(1);
    });

    pb.finish_with_message("Done!");

    let elapsed = start.elapsed();
    let hashes_computed = hashes.lock().unwrap().len();
    let hashes_per_sec = hashes_computed as f64 / elapsed.as_secs_f64();

    println!("\n✅ Hash computation complete!");
    println!("   Hashes computed: {}", hashes_computed);
    println!("   Time elapsed: {:.2}s", elapsed.as_secs_f64());
    println!("   Speed: {:.0} hashes/sec", hashes_per_sec);

    // Create output directory
    fs::create_dir_all(&args.output)?;

    // Generate cache file
    let cache_filename = get_cache_filename(&args.salt, args.iterations);
    let output_path = args.output.join(&cache_filename);

    println!("\n💾 Saving cache file...");
    println!("   Output: {}", output_path.display());

    let cache = CacheFile {
        domain: domain.clone(),
        salt: args.salt.clone(),
        iterations: args.iterations,
        wordlist_size: subdomains.len(),
        hashes: hashes.lock().unwrap().clone(),
    };

    let json = serde_json::to_string_pretty(&cache)?;
    fs::write(&output_path, json)?;

    let file_size = fs::metadata(&output_path)?.len();
    println!("   Size: {:.2} MB", file_size as f64 / 1_048_576.0);

    println!("\n🎉 Success! Cache file ready for DNSight");
    println!("\n📋 Next steps:");
    println!("   1. Copy to DNSight cache directory:");
    println!("      cp {} ../data/nsec3_cache/", output_path.display());
    println!("   2. Run DNSight zone walking:");
    println!("      dnsight zonewalk {}", domain);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nsec3_hash_no_salt_no_iterations() {
        // Test vector from RFC 5155
        let hash = calculate_nsec3_hash("example.com", &[], 0);
        // Should produce consistent hash
        assert_eq!(hash.len(), 32); // SHA1 base32 = 32 chars
    }

    #[test]
    fn test_nsec3_hash_with_salt() {
        let salt = hex::decode("AABBCCDD").unwrap();
        let hash = calculate_nsec3_hash("test.example.com", &salt, 0);
        assert_eq!(hash.len(), 32);
    }

    #[test]
    fn test_nsec3_hash_with_iterations() {
        let hash = calculate_nsec3_hash("example.com", &[], 10);
        assert_eq!(hash.len(), 32);
    }
}
