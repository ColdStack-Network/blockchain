#!/bin/bash

set -e

if [ -z $BLOCKCHAIN_BIN ] ; then
  echo Please set BLOCKCHAIN_BIN env variable
  exit 1
fi

trap "kill -- -$$" EXIT

LOG=$(mktemp)

echo Redirecting node log to $LOG

$BLOCKCHAIN_BIN --dev --tmp >$LOG 2>&1 &

node --unhandled-rejections=strict test.js
