#!/bin/bash

set -e

if [ -z $BLOCKCHAIN_PORT ] ; then
  # set default
  BLOCKCHAIN_PORT=9944
fi

trap "kill -- -$$" EXIT

LOG=$(mktemp)

echo Redirecting node log to $LOG

./target/release/node-template --dev --tmp --ws-port=$BLOCKCHAIN_PORT >$LOG 2>&1 &

NODE_URL=ws://localhost:$BLOCKCHAIN_PORT node --unhandled-rejections=strict test/test.js
