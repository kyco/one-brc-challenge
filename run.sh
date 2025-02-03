#!/bin/bash
if [ "$#" -ne 1 ]; then
    echo "Usage: $0 <binary_name>"
    exit 1
fi

echo "Running cargo build --release..."
cargo build --release || exit 1

echo "Executing binary with timing..."
time ./target/release/"$1"