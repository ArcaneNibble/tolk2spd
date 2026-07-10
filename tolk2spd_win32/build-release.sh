#!/bin/sh

set -e

cargo build --release
printf "Wine builtin DLL\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00" | dd of=target/x86_64-pc-windows-gnullvm/release/tolk.dll seek=64 bs=1 conv=notrunc
