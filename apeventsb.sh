#!/bin/sh

export DATABASE="postgresql://apevents_app:password@127.0.0.1/apeventsb_dev"
export DOMAIN=apeventsb.tunnelto.dev
export EXTERNAL_BASE=https://apeventsb.tunnelto.dev
export PORT=8082

cargo run
