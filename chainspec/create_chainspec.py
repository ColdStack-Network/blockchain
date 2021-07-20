#!/usr/bin/env python3

import subprocess
import argparse
import json
import os.path
from substrateinterface import Keypair, KeypairType

parser = argparse.ArgumentParser(description='Populate chainspec')
parser.add_argument('--chainspec', help='chainspec file name', required=True)
parser.add_argument('--rawchainspec', help='raw chainspec file name', required=True)
parser.add_argument('--secrets', help='directory with secrets', required=True)
parser.add_argument('--name', help='network name', required=True)
parser.add_argument('--id', help='network id', required=True)
args = parser.parse_args()

def run(command):
  print('excuting', command)
  subprocess.run(command, shell=True, check=True)

def create_chainspec():
  run('docker run --rm coldstack/privatechain build-spec \
    --disable-default-bootnode --chain local > ' + args.chainspec)

def create_raw_chainspec():
  volumepath = os.path.abspath(os.path.dirname(args.chainspec))
  run(f'docker run --rm -v {volumepath}:/chainspec \
    coldstack/privatechain build-spec \
    --chain=/chainspec/{os.path.basename(args.chainspec)} --raw --disable-default-bootnode > ' +
    args.rawchainspec
  )

def gen_keys(mnemonic):
  types = dict(ed25519 = KeypairType.ED25519, sr25519 = KeypairType.SR25519)
  result = dict(mnemonic=mnemonic)
  for type in types:
    keypair = Keypair.create_from_mnemonic(mnemonic, crypto_type=types[type])
    result[type] = dict(
      addr = keypair.ss58_address,
      pk = keypair.public_key,
    )
  return result

def read_chainspec():
  with open(args.chainspec,'r') as file:
    return json.loads(file.read())

def gen_chainspec():
  chainspec = read_chainspec()

  keys = [gen_keys(mnemonic) for mnemonic in secrets['authorities']]
  sudokey = gen_keys(secrets['sudo'])
  adminkey = gen_keys(secrets['admin'])

  chainspec['name'] = args.name
  chainspec['id'] = args.id

  runtime = chainspec['genesis']['runtime']

  runtime['validatorSet']['validators'] = [k['sr25519']['addr'] for k in keys]
  runtime['session']['keys'] = [
    [ 
      k['sr25519']['addr'],
      k['sr25519']['addr'],
      {
        "aura":    k['sr25519']['addr'],
        "grandpa": k['ed25519']['addr'],
      },
    ]
    for k in keys
  ]

  runtime['balances']['balances'] = [
    [sudokey['sr25519']['addr'],  1152921504606846976],
  ]
  for k in keys:
    runtime['balances']['balances'].append(
      [k['sr25519']['addr'],  1152921504606846976],
    )


  runtime['sudo']['key'] = sudokey['sr25519']['addr']
  runtime['coldStack']['key'] = adminkey['sr25519']['addr']

  return chainspec

if __name__ == '__main__':
  create_chainspec()

  with open(args.secrets, 'r') as file:
    secrets = json.loads(file.read())

  spec = json.dumps(gen_chainspec(), indent=2)
  with open(args.chainspec, 'w') as file:
    file.write(spec)

  create_raw_chainspec()
