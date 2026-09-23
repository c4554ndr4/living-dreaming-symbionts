use super::*;
use risc0_zkvm::{
    ExecutorEnv, InnerReceipt, LocalProver, Prover, ProverOpts, Receipt, VerifierContext,
};
use symbiont_core::{check_expectation, Expectation, Journal};
use symbiont_methods::{SYMBIONT_GUEST_ELF, SYMBIONT_GUEST_ID};

const MAX_RECEIPT_BYTES: usize = 64 * 1_048_576;

fn reject_dev_environment() -> Result<()> {
    if std::env::var("RISC0_DEV_MODE")
        .is_ok_and(|value| matches!(value.to_lowercase().as_str(), "1" | "true" | "yes"))
    {
        bail!("RISC0_DEV_MODE is enabled; unset it to generate or verify real proofs");
    }
    Ok(())
}

fn check_receipt(receipt: &Receipt, expected: &Expectation) -> Result<Journal> {
    // Both a build-time feature and an explicit check protect this path from ambient dev mode.
    if matches!(receipt.inner, InnerReceipt::Fake(_)) {
        bail!("development receipts are never accepted");
    }
    reject_dev_environment()?;
    receipt
        .verify_with_context(
            &VerifierContext::default().with_dev_mode(false),
            SYMBIONT_GUEST_ID,
        )
        .context("cryptographic receipt verification failed")?;
    let journal: Journal =
        serde_json::from_slice(&receipt.journal.bytes).context("invalid verified journal")?;
    check_expectation(&journal, expected).map_err(anyhow::Error::msg)?;
    Ok(journal)
}

pub fn prove(input_path: &Path, receipt_path: &Path) -> Result<()> {
    reject_dev_environment()?;
    if receipt_path.exists() {
        bail!("receipt path already exists; choose a new output path");
    }
    let bytes = read_bytes(input_path, MAX_INPUT_BYTES)?;
    let input = parse_input(&bytes).map_err(anyhow::Error::msg)?;
    let expected = expectation(&input).map_err(anyhow::Error::msg)?;
    let env = ExecutorEnv::builder()
        .write(&bytes)?
        .session_limit(Some(32 * 1024 * 1024))
        .build()?;
    eprintln!("Generating a real proof locally; raw evidence stays on this machine.");
    let receipt = LocalProver::new("local")
        .prove_with_ctx(
            env,
            &VerifierContext::default().with_dev_mode(false),
            SYMBIONT_GUEST_ELF,
            &ProverOpts::default().with_dev_mode(false),
        )?
        .receipt;
    let journal = check_receipt(&receipt, &expected)?;
    save_new(receipt_path, &receipt)?;
    println!("{}", serde_json::to_string_pretty(&journal)?);
    Ok(())
}

pub fn verify(receipt_path: &Path, expectation_path: &Path) -> Result<Journal> {
    let expected = read_json(expectation_path, MAX_INPUT_BYTES)?;
    let receipt = read_json(receipt_path, MAX_RECEIPT_BYTES)?;
    check_receipt(&receipt, &expected)
}

#[cfg(test)]
mod tests {
    use super::*;
    use risc0_zkvm::{FakeReceipt, ReceiptClaim};

    #[test]
    fn fake_receipts_are_rejected() {
        let input = parse_input(include_bytes!("../../../examples/market-yes.json")).unwrap();
        let bytes = serde_json::to_vec(&evaluate(&input).unwrap()).unwrap();
        let fake = Receipt::new(
            InnerReceipt::Fake(FakeReceipt::new(ReceiptClaim::ok(
                SYMBIONT_GUEST_ID,
                bytes.clone(),
            ))),
            bytes,
        );
        assert!(check_receipt(&fake, &expectation(&input).unwrap())
            .unwrap_err()
            .to_string()
            .contains("development"));
    }

    #[test]
    #[ignore = "set SYMBIONT_TEST_RECEIPT to a real receipt produced by the current guest"]
    fn real_receipt_rejects_a_different_program() {
        let path =
            std::env::var("SYMBIONT_TEST_RECEIPT").expect("provide a real synthetic receipt path");
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join(path);
        let receipt: Receipt = read_json(&path, MAX_RECEIPT_BYTES).unwrap();
        assert!(!matches!(receipt.inner, InnerReceipt::Fake(_)));
        let context = VerifierContext::default().with_dev_mode(false);
        receipt
            .verify_with_context(&context, SYMBIONT_GUEST_ID)
            .unwrap();
        let mut wrong_id = SYMBIONT_GUEST_ID;
        wrong_id[0] ^= 1;
        assert!(receipt.verify_with_context(&context, wrong_id).is_err());
    }
}
