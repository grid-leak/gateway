# gateway
This project features a JSON-RPC 2.0–compliant HTTP API/Gateway built for Mirror’s Edge Catalyst. The Gateway handles most of the social features of the game: followers, player inventory, progress tracking, in-game challenges (billboards, dashes, time trials, etc.), and more.

![banner](https://github.com/user-attachments/assets/0edcb839-c66c-4f55-8503-e2dabb4628e5)

## Getting Started
> [!NOTE]
> If you're using VS Code it's highly suggested you install the [`rust-analyzer`](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer) extenstion

### Requirements

- [Rust](https://www.rust-lang.org/tools/install)

### Installation
Once you have Rust installed, clone the repository and run `cargo check` from the root of the project. This will install all the necessary crates.
Once that completes, run `cargo run` to launch the server.

### Connecting
Add [catalyst-mitm](https://github.com/ploxxxy/catalyst-mitm) to the installation directory of the game to redirect all traffic to localhost. Note that you will need to set up a running Blaze instance to point to the Gateway, and a Redirector instance to point to the Blaze. When launched, the game should connect to your new server.

## Useful Resources
1. [jsonrpsee docs](https://docs.rs/jsonrpsee/latest/jsonrpsee/index.html)
2. [jsonrpsee_proc_macros docs](https://docs.rs/jsonrpsee-proc-macros/latest/jsonrpsee_proc_macros/attr.rpc.html)
3. [serde docs](https://docs.rs/serde/latest/serde/index.html)
4. [tokio docs](https://docs.rs/tokio/latest/tokio/index.html)