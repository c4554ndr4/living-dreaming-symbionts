#![no_main]
risc0_zkvm::guest::entry!(main);

pub fn main() {
    let bytes: Vec<u8> = risc0_zkvm::guest::env::read();
    let input = symbiont_core::parse_input(&bytes).expect("invalid evidence input");
    let journal = symbiont_core::evaluate(&input).expect("invalid metric evidence");
    risc0_zkvm::guest::env::commit_slice(&serde_json::to_vec(&journal).unwrap());
}
