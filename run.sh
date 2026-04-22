#!/bin/bash
set -e

trap 'kill $(jobs -p) 2>/dev/null' EXIT

(cd backend && cargo run --release) &
(cd frontend && npm run prod) &

wait
