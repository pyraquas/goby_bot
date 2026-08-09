#!/usr/bin/env bash
# Build goby_bot for the Raspberry Pi and push it over SSH.
# Run from WSL:  ./deploy.sh pi@192.168.1.209
#
# Ships .env.prod by default, since the Pi runs the production bot. Override
# with ENV_FILE=.env to put the dev bot there instead.
# The remote secrets file is written once and then left alone; FORCE_ENV=1
# overwrites it, which is what you want after switching tokens.
set -euo pipefail

TARGET_HOST="${1:?usage: ./deploy.sh user@host [remote-dir]}"
REMOTE_DIR="${2:-goby_bot}"
ENV_FILE="${ENV_FILE:-.env.prod}"
FORCE_ENV="${FORCE_ENV:-0}"
TRIPLE=aarch64-unknown-linux-gnu
BIN="target/$TRIPLE/release/goby_bot"

[ -f "$ENV_FILE" ] || { echo "missing $ENV_FILE" >&2; exit 1; }

cargo build --release --target "$TRIPLE"

ssh "$TARGET_HOST" "mkdir -p '$REMOTE_DIR'"

# The running bot holds its own binary open, so land next to it and swap.
scp "$BIN" "$TARGET_HOST:$REMOTE_DIR/goby_bot.new"
ssh "$TARGET_HOST" "mv '$REMOTE_DIR/goby_bot.new' '$REMOTE_DIR/goby_bot' && chmod +x '$REMOTE_DIR/goby_bot'"

if [ "$FORCE_ENV" != "1" ] && ssh "$TARGET_HOST" "test -f '$REMOTE_DIR/.env'"; then
    echo "remote .env already present, left untouched (FORCE_ENV=1 to replace)"
else
    scp "$ENV_FILE" "$TARGET_HOST:$REMOTE_DIR/.env"
    ssh "$TARGET_HOST" "chmod 600 '$REMOTE_DIR/.env'"
    echo "shipped $ENV_FILE as remote .env"
fi

echo "deployed to $TARGET_HOST:$REMOTE_DIR"
echo "run it with:  ssh $TARGET_HOST 'cd $REMOTE_DIR && ./goby_bot'"
