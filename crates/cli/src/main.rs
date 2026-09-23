use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
#[cfg(feature = "zk")]
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::{
    fs::{File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
};
use symbiont_core::{evaluate, expectation, parse_input, MAX_INPUT_BYTES};

#[cfg(feature = "zk")]
mod proof;

#[derive(Parser)]
#[command(
    version,
    about = "Check agent outcome metrics and verify receipts over supplied evidence"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Preview deterministic results. This does not produce a cryptographic proof.
    Evaluate { input: PathBuf },
    /// Save the policy and evidence commitment to keep independently of the prover.
    Prepare { input: PathBuf, expected: PathBuf },
    /// Prove locally and save a receipt. Requires --features zk at build time.
    Prove { input: PathBuf, receipt: PathBuf },
    /// Verify against the compiled program and your independently retained expectation.
    Verify {
        receipt: PathBuf,
        #[arg(long)]
        expect: PathBuf,
        /// Also fail with exit code 2 if the valid proof's metric outcome is not met.
        #[arg(long)]
        require_met: bool,
    },
    /// Print the identity of the compiled proof guest (requires --features zk).
    ImageId,
}

fn read_bytes(path: &Path, max: usize) -> Result<Vec<u8>> {
    let mut bytes = Vec::new();
    File::open(path)
        .with_context(|| format!("opening {}", path.display()))?
        .take(max as u64 + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() > max {
        bail!("{} exceeds the {} byte limit", path.display(), max);
    }
    Ok(bytes)
}

#[cfg(feature = "zk")]
fn read_json<T: DeserializeOwned>(path: &Path, max: usize) -> Result<T> {
    serde_json::from_slice(&read_bytes(path, max)?)
        .with_context(|| format!("parsing {}", path.display()))
}

fn save_new(path: &Path, value: &impl Serialize) -> Result<()> {
    let bytes = serde_json::to_vec(value)?;
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(path).with_context(|| {
        format!(
            "creating {} (existing files are never overwritten)",
            path.display()
        )
    })?;
    file.write_all(&bytes)?;
    file.write_all(b"\n")?;
    file.sync_all()?;
    Ok(())
}

fn main() -> Result<()> {
    match Cli::parse().command {
        Command::Evaluate { input } => {
            let input =
                parse_input(&read_bytes(&input, MAX_INPUT_BYTES)?).map_err(anyhow::Error::msg)?;
            let journal = evaluate(&input).map_err(anyhow::Error::msg)?;
            eprintln!("Preview only: supplied evidence has not been authenticated; no proof was generated.");
            println!("{}", serde_json::to_string_pretty(&journal)?);
        }
        Command::Prepare { input, expected } => {
            let input =
                parse_input(&read_bytes(&input, MAX_INPUT_BYTES)?).map_err(anyhow::Error::msg)?;
            save_new(&expected, &expectation(&input).map_err(anyhow::Error::msg)?)?;
            eprintln!("Saved expectation. Review the policy and retain this file independently of the prover.");
        }
        Command::Prove { input, receipt } => {
            #[cfg(feature = "zk")]
            {
                proof::prove(&input, &receipt)?;
            }
            #[cfg(not(feature = "zk"))]
            {
                let _ = (input, receipt);
                bail!("rebuild with --features zk to generate real proofs");
            }
        }
        Command::Verify {
            receipt,
            expect,
            require_met,
        } => {
            #[cfg(feature = "zk")]
            {
                let journal = proof::verify(&receipt, &expect)?;
                println!("{}", serde_json::to_string_pretty(&journal)?);
                eprintln!("Cryptographic receipt verified against the compiled program and expected policy/evidence.");
                if require_met && journal.outcome != symbiont_core::Outcome::Met {
                    std::process::exit(2);
                }
            }
            #[cfg(not(feature = "zk"))]
            {
                let _ = (receipt, expect, require_met);
                bail!("rebuild with --features zk to verify receipts");
            }
        }
        Command::ImageId => {
            #[cfg(feature = "zk")]
            {
                println!(
                    "{}",
                    risc0_zkvm::sha::Digest::from(symbiont_methods::SYMBIONT_GUEST_ID)
                );
            }
            #[cfg(not(feature = "zk"))]
            {
                bail!("rebuild with --features zk to compile the proof guest");
            }
        }
    }
    Ok(())
}
