#!/bin/bash

set -e

# cd to repo root
cd $(dirname "${BASH_SOURCE[0]}")/..


# Parse CLI
EXAMPLE=$1
if [ -z $EXAMPLE ] || [ ! -d $EXAMPLE ] ; then 
  echo Usage: test.sh PATH-TO-EXAMPLE
  exit 1
fi


# Create temp directory
mkdir -p tmp_runtime_upgrade_examples


# Build base version to be upgraded
BASE_BIN=tmp_runtime_upgrade_examples/node-template.base
if [ ! -f $BASE_BIN ] ; then
  echo Build base version
  cargo build
  cp ./target/debug/node-template $BASE_BIN
  cp ./target/debug/wbuild/node-template-runtime/node_template_runtime.compact.wasm tmp_runtime_upgrade_examples/wasm.base
else
  echo Base version already exists, skipping
fi


# Build modified version
#BASE_MODIFIED=tmp_runtime_upgrade_examples/node-template.$(basename $EXAMPLE)
WASM=tmp_runtime_upgrade_examples/wasm.$(basename $EXAMPLE)
if [ ! -f $WASM ] ; then
  echo Build modified version
  cp $EXAMPLE/lib.rs pallets/template/src/lib.rs
  cp runtime_upgrade_examples/runtime-lib.rs runtime/src/lib.rs
  cargo build -p node-template-runtime
  #cp ./target/debug/node-template $BASE_MODIFIED
  cp ./target/debug/wbuild/node-template-runtime/node_template_runtime.compact.wasm $WASM
else
  echo Modified version already exists, skipping
fi


echo Starting blockchain
trap "kill -- -$$" EXIT
LOG=$(mktemp)
echo Redirecting node log to $LOG
$BASE_BIN --dev --tmp >$LOG 2>&1 &
sleep 10

cd runtime_upgrade_examples/
node --unhandled-rejections=strict $(basename $EXAMPLE)/test.js ../$WASM
