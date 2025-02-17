//! Oneshot execution of `validate_blocks` benchmark.
//! Can be useful to profile using flamegraph.
//!
//! ```bash
//! CARGO_PROFILE_RELEASE_DEBUG=true cargo flamegraph --root --release --example validate_blocks
//! ```

mod validate_blocks;

use validate_blocks::WsvValidateBlocks;

fn main() {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Failed building the Runtime");
    {
        let _guard = rt.enter();
        iroha_logger::test_logger();
    }
    iroha_logger::test_logger();
    iroha_logger::info!("Starting...");
    let bench = WsvValidateBlocks::setup(rt.handle()).expect("Failed to setup benchmark");
    WsvValidateBlocks::measure(bench).expect("Failed to execute bnechmark");
}
