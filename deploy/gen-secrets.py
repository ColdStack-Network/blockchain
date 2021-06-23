#!/usr/bin/env python3

import argparse
import subprocess
import json
from substrateinterface import Keypair, KeypairType

parser = argparse.ArgumentParser(description='Generate blockchain secrets')
parser.add_argument('--file', help='secrets file', required=True)
args = parser.parse_args()

def generate_node_key():
  result = subprocess.run(
    'docker run --rm coldstack/privatechain key generate-node-key',
    shell=True, capture_output=True, check=True, text=True
  )
  nodekey = result.stdout.strip()
  peer_id = result.stderr.strip()
  return dict(nodekey=nodekey, peer_id=peer_id)

nodekey = generate_node_key()

with open(args.file, 'w') as file:
  file.write(  
    json.dumps(dict(
      authorities = [
        Keypair.generate_mnemonic(),
      ],
      sudo    = Keypair.generate_mnemonic(),
      admin   = Keypair.generate_mnemonic(),
      nodekey = nodekey['nodekey'],
      peer_id = nodekey['peer_id'],
    ), indent = 2)
  )
