## What is this

Submission for the Pirate Jam 15 by anti.negative, fabi337, hquil, InnocentusLime and Redderick

## Get Started

1. Clone the project repo

## Running the project

1. Make sure you have [Rust installed](https://www.rust-lang.org/learn/get-started)
    * You need **rustup**
    * Your Rust version must be 1.79 or higher
2. Install the wasm32 target
```sh
rustup target install wasm32-unknown-unknown
```
3. Install the tools required for web deploying
```sh
cargo install wasm-bindgen-cli --locked --version "0.2.92"
cargo install wasm-opt --locked --version "0.116.1"
cargo install --locked trunk
```
4. To run the project locally, run the following command
```sh
# For the intended experience
trunk serve --release
# For a faster build, but worse performance
trunk serve
```
5. The tool will print the address it's hosted on in the terminal