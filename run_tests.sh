#!/bin/bash

set -e

cargo test
cargo test --features envelope
