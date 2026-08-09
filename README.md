# goby_bot
Discord bot for "La poissonnerie" discord server

## Building for the Raspberry Pi 4

The Pi runs a 64-bit OS, so the target is `aarch64-unknown-linux-gnu`. Windows has
no linker for it, so the build happens in WSL.

One-time setup, inside the WSL distribution:

```sh
sudo apt update
sudo apt install -y build-essential gcc-aarch64-linux-gnu
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
. "$HOME/.cargo/env"
rustup target add aarch64-unknown-linux-gnu
```

Build (from the project directory, `/mnt/c/Users/<you>/Documents/Projects/goby_bot`):

```sh
cargo build --release --target aarch64-unknown-linux-gnu
```

The binary lands in `target/aarch64-unknown-linux-gnu/release/goby_bot`.

The linker is wired up in `.cargo/config.toml`. TLS comes from `rustls`, so there is
no OpenSSL to cross-build.

### Deploying

Copy the binary and the `.env` next to each other on the Pi:

```sh
scp target/aarch64-unknown-linux-gnu/release/goby_bot .env pi@raspberrypi.local:~/goby_bot/
```

The bot reads `.env` from the working directory and writes its state to `./data`
(override with `GOBY_DATA_DIR`), so start it from its own directory.
