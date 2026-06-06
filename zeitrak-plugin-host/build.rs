use std::process::Command;

fn main() {
    let plugin_dir =
        concat!(env!("CARGO_MANIFEST_DIR"), "/../examples/plugins/leave-requests");
    println!("cargo:rerun-if-changed={plugin_dir}/src");
    println!("cargo:rerun-if-changed={plugin_dir}/Cargo.toml");

    let status = Command::new("cargo")
        .args(["build", "--release", "--target", "wasm32-unknown-unknown"])
        .current_dir(plugin_dir)
        .status();

    match status {
        Ok(s) if s.success() => {}
        Ok(s) => {
            println!(
                "cargo:warning=leave-requests WASM build exited with {s} — \
                 integration tests that require the WASM will be skipped"
            );
        }
        Err(e) => {
            println!(
                "cargo:warning=could not invoke cargo for leave-requests WASM build: {e} — \
                 integration tests that require the WASM will be skipped"
            );
        }
    }
}
