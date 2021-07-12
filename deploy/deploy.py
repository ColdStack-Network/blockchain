#!/usr/bin/env python3

import argparse
import subprocess
import json

parser = argparse.ArgumentParser(description='Deploy blockchain')
parser.add_argument('--validator-node', 
  help='validator node ssh address. First node becomes boot node', 
  nargs='+'
)
parser.add_argument('--api-node', help='api node ssh address', nargs='+', default=[])
parser.add_argument('--boot-node-addr', help='first (boot) node ip address', required=True)
parser.add_argument('--secrets', help='secrets file', required=True)
parser.add_argument('--env', help='production or staging', choices=['production', 'staging'], required=True)
parser.add_argument('--tag', help='tag of docker image', required=True)
args = parser.parse_args()

print('Parsed CLI args', args)

def read_secrets_file():
  with open(args.secrets, 'r') as file:
    return json.loads(file.read())

def run(command, input=None):
  subprocess.run(command, shell=True, check=True, text=True, input=input)

def run_ssh(host, input, sudo=False):
  print('run command on host', host)
  print(input)
  print('\n')
  if sudo:
    run('ssh ' + host + ' sudo bash -e' , input)
  else:
    run('ssh ' + host + ' bash -e' , input)

def prepare_blockchain_dir(host):
  print('creating blockchain directory on host', host)
  run_ssh(host, 
    f"""
    mkdir -p /var/blockchain
    chown 1000.1000 /var/blockchain -R
    """,
    sudo=True
  )

def init_node(host):
  print('Initialize node', host)
  prepare_blockchain_dir(host)

def init_keystore(host):
  print('Initialize keystore', host)
  key_types = dict(aura = 'Sr25519', gran = 'Ed25519')
  for key_type in key_types:
    scheme = key_types[key_type]
    key_file_name = f"blockchain_deploy_key_{key_type}"
    with open(f"/tmp/{key_file_name}", 'w') as file:
      file.write(secrets['authorities'][0])

    print(f"Copy authority key file {key_type}", host)
    run(f"scp /tmp/{key_file_name} {host}:/tmp")

    print(f"Initializing key store for {key_type}", host)
    try:
      input = f"docker run \
        -v /var/blockchain:/data \
        -v /tmp:/keys \
        --rm \
        coldstack/privatechain:{args.tag} key insert \
        --chain /chainspec/{args.env}.json \
        --key-type {key_type}  \
        --scheme {scheme} \
        --suri /keys/{key_file_name} \
      "
      run_ssh(host, input)
    finally:
      print('Removing authority key file', host)
      run_ssh(host, f"rm /tmp/{key_file_name}")


def run_api_node(host):
  print('Run API node on host', host)
  init_node(host)
  input = f"docker run \
  -d \
  --restart unless-stopped \
  -p 30333:30333 \
  -p 9933:9933 \
  -p 9944:9944 \
  -v /var/blockchain:/data \
  coldstack/privatechain:{args.tag} \
  --name 'Coldstack Public {args.env}' \
  --pruning archive \
  --no-telemetry --no-prometheus \
  --chain /chainspec/{args.env}.json \
  --port 30333   \
  --ws-port 9944 \
  --rpc-external \
  --ws-external  \
  --rpc-port 9933 \
  --rpc-cors all \
  --bootnodes /ip4/{args.boot_node_addr}/tcp/30333/p2p/{secrets['peer_id']} \
  "
  run_ssh(host, input)


def run_validator_node(host, is_boot_node):
  print('Run validator node on host', host, 'is_boot_node =', is_boot_node)
  init_node(host)
  init_keystore(host)
  input = f"docker run \
  -d \
  --restart unless-stopped \
  -p 30333:30333 \
  -v /var/blockchain:/data \
  coldstack/privatechain:{args.tag} \
  --validator \
  --name 'Coldstack Validator {args.env}' \
  --pruning archive \
  --no-telemetry --no-prometheus \
  --chain /chainspec/{args.env}.json \
  --port 30333 \
  "
  if is_boot_node:
    input = f"{input} \
     --node-key {secrets['nodekey']} \
    "
  else:
    input = f"{input} \
     --bootnodes /ip4/{args.boot_node_addr}/tcp/30333/p2p/{secrets['peer_id']} \
    "
  run_ssh(host, input)


secrets = read_secrets_file()

for i, host in enumerate(args.validator_node):
  run_validator_node(host, is_boot_node = (i == 0))

for host in args.api_node:
  run_api_node(host)
