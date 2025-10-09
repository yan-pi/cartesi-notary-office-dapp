# Cargo Configuration Notes

## Default Target

This project is configured to build for `riscv64gc-unknown-linux-gnu` by default (for Cartesi RISC-V environment).

## Local Development & Testing

To run tests on your native machine (macOS/Linux), override the target:

```bash
# Run tests with native target
cargo test --lib --target aarch64-apple-darwin  # macOS ARM (M1/M2)
cargo test --lib --target x86_64-apple-darwin   # macOS Intel
cargo test --lib --target x86_64-unknown-linux-gnu  # Linux x64

# Run specific test suite
cargo test --test unit --target aarch64-apple-darwin
```

## Docker Builds

The default config ensures Docker builds work correctly:

```bash
# Build Docker image
cartesi build
# or
docker buildx build --load -t cartesi-notary-dapp .
```

## Binary Target

The binary is defined in `Cargo.toml` as:
```toml
[[bin]]
name = "dapp"
path = "src/main.rs"
```

This ensures Cargo builds the binary even when `src/lib.rs` exists.
