# gateway
![Status](https://img.shields.io/badge/Status-Beta-yellowgreen?style=flat-square)
![License](https://img.shields.io/badge/License-AGPL--3.0-blue?style=flat-square)

This project emulates `mec-gw.ops.dice.se`, the web gateway component of the Mirror's Edge Catalyst server stack. The game client communicates with this server via JSON-RPC over HTTP to handle its asynchronous multiplayer features. Following EA's server shutdown, this project replaces the dead endpoints to restore the game's "Social Play" functionality

### Supported Features
- **Beat LEs:** Handles creating, placing, and loading user Beat LEs across the map
- **Time Trials:** Saves and serves custom checkpoints, replay files, and leaderboard times
- **Dashes & Leaderboards:** Saves, tracks and returns player rankings across leaderboards
- **Customization:** Handles progression stats, Player Tags, Runner Kits, and Echoes
- **Achievements:** Allows getting all of the in-game achievements

## Prerequisites
- Rust toolchain (2024 edition)
- Discord application for authentication
- PostgreSQL (tested with version 17)
- S3-compatible object storage (tested with SeaweedFS 4.13)

## Configuration
Configure the gateway using the following environment variables in your `.env` file:

| Variable | Example | Description |
|---|---|---|
| `PORT` | `3000` | Port number the gateway listens on |
| `DATABASE_URL` | `postgres://user:password@localhost:5432/gateway` | PostgreSQL connection string |
| `GATEWAY_CLIENT_ID` | `MirrorsEdgeCatalyst-SERVER-PC` | Client identifier for gateway session generation |
| `GATEWAY_SECRET` | `5f80a...ab235` | Hex-encoded key for payload decryption |
| `UGC_BASE_URL` | `http://localhost:3000/checkpoints` | Base URL for user-generated content redirects |
| `S3_BUCKET` | `replays` | S3 bucket name for uploads |
| `S3_ENDPOINT` | `http://localhost:8333/` | HTTP URL of the S3-compatible API |
| `AWS_ACCESS_KEY_ID` | `your-access-key` | Access key for S3 authorization |
| `AWS_SECRET_ACCESS_KEY` | `your-secret-key` | Secret key for S3 authorization |
| `AWS_REGION` | `us-east-1` | AWS region code |
| `DISCORD_CLIENT_ID` | `1234567890123456789` | Discord application client ID for OAuth |
| `RUST_LOG` | `gateway=debug` | Logger filter configuration |
| `LOG_FORMAT` | `pretty` | Log output format |

## Running
> Upon startup, the server automatically synchronizes the database schema using SeaORM registry synchronization

### Via cargo:
```bash
cargo run --release
```

### Via Docker:
```bash
docker build -t pamplona-gateway .
# Match the port mapping to the PORT defined in your .env
docker run -p 3000:3000 --env-file .env pamplona-gateway
```

## Connecting
> To play on the private server, the game client needs to be pointed towards the custom redirector instance

### The easy way
Use an existing launcher - [BeatLink](https://github.com/synthic/BeatLink) to skip manual client patching entirely

### The hard way
The game's executable must be patched to disable SSL verification and redirect traffic from EA's servers to the custom instance. This can be done via:
- editing the [assembly instructions](https://github.com/synthic/beatrevival-launcher/blob/master/main.cpp)
- using a [runtime hook](https://github.com/ploxxxy/catalyst-mitm)

## Meta
### Credits
- [WarrantyVoider](https://github.com/zeroKilo) - sharing tons of info publicly, help with dumping and reversing the game
- [Doomaholic](https://github.com/synthic) - guidance on early server architecture and patching the executable
- [av1d](https://github.com/av1d) and [wagwan piffting blud](https://github.com/wagwan-piffting-blud) - created mec-scrape, for downloading the original server data
- [wagwan piffting blud](https://github.com/wagwan-piffting-blud) - prototyped [Rust port](https://gitea.wagspuzzle.space/wagwan-piffting-blud/pamplona-future-rust/) of [pamplona-future](https://github.com/ploxxxy/pamplona-future), helping identify a [critical roadblock](https://youtu.be/uDY9DJ6B0xM)
- The [Beat Revival](https://beatrevival.me) team - creating the initiative, capturing packets, community updates, scraping the servers


### Acknowledgements
Some parts of this codebase were assisted or generated using AI tools, then manually reviewed, refactored, and tested for accuracy

### Legal Disclaimer
This is a free, open-source research and preservation project. It is not affiliated with, endorsed by, or connected to Electronic Arts (EA), DICE, or any of their associated entities

* **No Copyrighted Material:** This repository does not contain, distribute, or host any copyrighted assets, binaries, or game files from *Mirror's Edge Catalyst*. It is a clean-room implementation based entirely on network analysis
* **Non-Commercial:** This software is distributed strictly free of charge for historical preservation and educational purposes. No monetization, donations, or commercial transactions are accepted
* **Trademarks:** All product names, logos, and brands are property of their respective owners. Their use here is solely for compatibility identification purposes

### License
This project is licensed under the GNU Affero General Public License v3.0 - see the [LICENSE](LICENSE) file for details