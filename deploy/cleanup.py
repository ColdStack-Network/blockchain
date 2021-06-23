#!/usr/bin/env python3

import argparse
import subprocess
import json
from substrateinterface import Keypair, KeypairType

parser = argparse.ArgumentParser(description='Cleanup after unsuccessful deployment')
parser.add_argument('--node', 
  help='node to cleanup', 
  nargs='+',
  required=True,
)
args = parser.parse_args()

def run(command, input):
  subprocess.run(command, shell=True, check=True, text=True, input=input)

def run_ssh(host, input, sudo=False):
  print('run command on host', host)
  print(input)
  print('\n')
  if sudo:
    run('ssh ' + host + ' sudo bash -e' , input)
  else:
    run('ssh ' + host + ' bash -e' , input)

for host in args.node:
  run_ssh(host, """
    docker ps -aq --filter "ancestor=coldstack/privatechain" \
      | xargs -r  docker stop | xargs -r docker rm
    sudo rm -rf /var/blockchain
  """)
