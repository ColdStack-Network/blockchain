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
parser.add_argument('--preserve-data', 
  help='Do not remove data, just remove container', 
  action='store_true'
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
    docker ps -a | grep coldstack/privatechain | awk '{print $1}' \
      | xargs -r  docker stop | xargs -r docker rm
  """)
  
  if not args.preserve_data:
    run_ssh(host, "sudo rm -rf /var/blockchain")
