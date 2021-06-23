#!/usr/bin/env python3

import sys
import argparse
import datetime
from substrateinterface import SubstrateInterface

# seconds to wait for new block. 
# Normally new block should be released every 6 seconds
TIMEOUT_SEC = 30 

parser = argparse.ArgumentParser(
  description="""Checks if API blockchain node is OK. 
  Returns exit code 0 is everything is ok, not zero if not ok"""
)
parser.add_argument('--node-url', help='Node URL', required=True)
args = parser.parse_args()

substrate = SubstrateInterface(
  url=args.node_url,
  type_registry_preset='substrate-node-template',
)

def blockheight():
  return int(str(substrate.query(module='System', storage_function='Number')))

print('getting current blockheight')
current_blockheight = blockheight()
print('current blockheight is', current_blockheight)

current = datetime.datetime.now()

while (datetime.datetime.now() - current).seconds < TIMEOUT_SEC:
  height = blockheight()
  if height > current_blockheight:
    print(f'received new block number {height}, everything is ok')
    break
else:
  print(f'No new block for {TIMEOUT_SEC} seconds. Healthcheck failed')
  sys.exit(1)
