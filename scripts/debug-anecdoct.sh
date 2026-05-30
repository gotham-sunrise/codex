#!/bin/bash

# Set "chatgpt.cliExecutable": "/Users/<USERNAME>/code/anecdoct/scripts/debug-anecdoct.sh" in VSCode settings to always get the 
# latest anecdoct-rs binary when debugging Anecdoct Extension.


set -euo pipefail

ANECDOCT_RS_DIR=$(realpath "$(dirname "$0")/../anecdoct-rs")
(cd "$ANECDOCT_RS_DIR" && cargo run --quiet --bin anecdoct -- "$@")