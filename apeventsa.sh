#!/bin/sh

export DATABASE="postgresql://apevents_app:password@127.0.0.1/apeventsa_dev"
export DOMAIN=apeventsa.tunnelto.dev
export EXTERNAL_BASE=https://apeventsa.tunnelto.dev
export PORT=8081

cargo run
