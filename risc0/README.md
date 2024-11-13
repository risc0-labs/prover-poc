# RISC Zero prover-poc

## Getting Started

Navigate to [prover-poc/risc0](risc0/) and run the prorgam.

```
BONSAI_API_KEY=<API_KEY> BONSAI_API_URL=<API_URL> RISC0_DEV_MODE=<true/false> RUST_LOG=info cargo run --release
```

You can enable `RISC0_DEV_MODE` to enable proving, or disable it to just execute the `prover-poc`. 

### Performance 

As of October 2024:

```
executor: execution time: 7.705603125s
number of segments: 292
total cycles: 306184192
user cycles: 280402266
```

With Bonsai, proving takes about 366 seconds - nearly 24Mhz.  

