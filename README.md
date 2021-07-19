# Coldstack Privatechain

General information:
https://coldstack.atlassian.net/wiki/spaces/CS/pages/66977793/Blockchain

## Build

To setup rust, see instructions below.

To build:

```
cargo build --release

```

## Build docker image

```
docker build -t privatechain .
```

## Push docker image

```
docker tag privatechain coldstack/privatechain:latest
docker push coldstack/privatechain
```

## Github workflows

Docker image built and pushed in a github worflow. See
.github/workflows/publish-docker.yml

### Tags

Tag name should correspond to `spec_version` (see [runtime
upgrades](#runtime-upgrades) for details).

## Run docker image

Run in dev mode

```
docker run coldstack/privatechain --dev --ws-external
```

Run with volume:

```
# Set permissions
chown 1000.1000 /your/host/directory -R

# Run
docker run -v /your/host/directory:/data coldstack/privatechain --dev --ws-external
```

Expose web service port:
```
docker run -p 9944:9944 coldstack/privatechain --dev --ws-external
```

## Run tests

First build and then

```
./test/test.sh
```

## API

Node.js client for blockchain is https://polkadot.js.org/docs/api/

You start by initializing `api` object (see docs). Then you can access storage
items and send transactions

### Common types

#### `ETHAddress`

Ethereum address. In blockchain runtime it is represented as `Vec<u8>`. It is
returned from api as `Uint8Array`. You can pass it to api as `Uint8Array`,
`Buffer` or hex-encoded string. Note that hex-encoded string must start with
leading `'0x'`. Length is 20 bytes.

#### `Hash`. 

32-bytes long hash. In blockchain runtime it is represented as `Vec<u8>`. It is
returned from api as `Uint8Array`. You can pass it to api as `Uint8Array`,
`Buffer` or hex-encoded string. Note that hex-encoded string must start with
leading `'0x'`.

#### `AccountId`

Built-in substrate type

#### `Option`

https://polkadot.js.org/docs/api/start/types.basics#working-with-optiontype

#### Number

https://polkadot.js.org/docs/api/start/types.basics#working-with-numbers

### Storage items

Most of data in blockchain is stored in storage maps. Storage map support the
following operations:

- get value by key
- get all entries

See docs https://polkadot.js.org/docs/api/start/api.query

All storage functions are async and return value wrapped to promise

#### `api.query.coldStack.key(): AccountId`

Admin account key. Constant value that is set in genesis.

#### `api.query.coldStack.totalIssuance(): number`

Constant value equal to total issuance of ColdStack token in Ethereum.

#### `api.query.coldStack.lockedFunds(): number`

Locked funds

#### `api.query.coldStack.balances(address: ETHAddress): number`

Balance of address

#### `api.query.coldStack.nodeURLs(address: ETHAddress): string`

URL of node for given address

#### `api.query.coldStack.filePermissionOwnersByETHAddress(address: ETHAddress): AccountId`

Substrate account of file node by eth address

#### `api.query.coldStack.filePermissionOwnersByAccountId(account: AccountId): ETHAddress`

eth address of file node by substrate account

#### `api.query.coldStack.billingPermissionOwnersByETHAddress(address: ETHAddress): AccountId`

Substrate account of billing node by eth address

#### `api.query.coldStack.billingPermissionOwnersByAccountId(account: AccountId): ETHAddress`

eth address of billing node by substrate account

#### `api.query.coldStack.gateways(address: ETHAddress): Gateway`

Get seed of gateway node by gateway node address

### Transactions

#### upload

```
api.tx.coldStack.upload(
  user_eth_address: ETHAddress,
  file_name_hash: Hash,
  file_size_bytes: number,
  file_contents_hash: Hash,
  gateway_eth_address: Hash,
)
```

#### download
```
api.tx.coldStack.download(
  user_eth_address: ETHAddress,
  file_name_hash: Hash,
  file_size_bytes: number,
  file_contents_hash: Hash,
  gateway_eth_address: Hash,
)
```

#### delete
```
api.tx.coldStack.delete(
  user_eth_address: ETHAddress,
  file_name_hash: Hash,
)
```

#### deposit
```
api.tx.coldStack.deposit(
  account: ETHAddress,
  value: number,
)
```

#### withdraw
```
api.tx.coldStack.withdraw(
  account: ETHAddress,
  value: number,
)
```

#### transfer
```
api.tx.coldStack.transfer(
  from: ETHAddress,
  to: ETHAddress,
  value: number,
)
```

#### grantFilePermission

```
api.tx.coldStack.grantFilePermission(
  eth_address: ETHAddress,
  account_id: AccountId,
  node_url: string,
)
```

#### grantBillingPermission

```
api.tx.coldStack.grantBillingPermission(
  eth_address: ETHAddress,
  account_id: AccountId,
  node_url: string,
)
```

#### revokeFilePermission

```
api.tx.coldStack.revokeFilePermission(
  eth_address: ETHAddress,
)
```

#### revokeBillingPermission

```
api.tx.coldStack.revokeBillingPermission(
  eth_address: ETHAddress,
)
```

#### registerGatewayNode

```
api.tx.coldStack.registerGatewayNode(
  node_eth_address: ETHAddress,
  seed_eth_address: Option<ETHAddress>,
  node_url: string,
)
```

### Get list of gateway nodes:

```
const {u8aToString} = require('@polkadot/util/u8a/toString')

const api = await ApiPromise.create({ 
  provider: wsProvider,
  types: {
    Gateway: {
      address: 'Vec<u8>',
      seedAddress: 'Option<Vec<u8>>',
      storage: 'u8',
    },
  },
})

async function gatewayNodes(){
  const nodeEntries = await api.query.coldStack.gateways.entries()
  return Promise.all(nodeEntries.map(async ([_, gateway]) => {
    const nodeAddress = gateway.address.toString('hex')
    return {
      nodeAddress,
      seedAddress: gateway.seedAddress.isNone ? null : gateway.seedAddress.toString('hex'),
      storage: gateway.storage.toNumber(),
      url: u8aToString(await api.query.coldStack.nodeURLs(nodeAddress)),
    }
  }))
}
```

Returns:

```
[
  {
    nodeAddress: '0x6666666666666666666666666666666666666666',
    seedAddress: '0x2222222222222222222222222222222222222222',
    url: 'http://gateway_sec.test'
  },
  {
    nodeAddress: '0x2222222222222222222222222222222222222222',
    seedAddress: null,
    url: 'http://gateway_seed.test'
  }
]
```

where

- `nodeAddress`: ETH address of gateway node
- `seedAddress`: ETH address of seed node for this gateway node. `null` if node is a seed node itself.
- `url`: URL of node

# Production deployment

[Production deployment](./prod.md)

# Runtime upgrades

Blockchain runtime (including ColdStack-specific code) could be upgraded by API
call, without redeploying blockchain nodes. See
https://substrate.dev/docs/en/tutorials/forkless-upgrade to learn mode about
runtime upgrades. Note that you must bump `spec_version` in
[./runtime/src/lib.rs](./runtime/src/lib.rs) to trigger runtime upgrade.

To see some examples of runtime upgrades please checkout branch
`runtime_upgrade_examples` (see
[README](https://github.com/ColdStack-Network/blockchain/blob/runtime_upgrade_examples/runtime_upgrade_examples/README.md)
for details)

# Substrate Node Template

Repository is forked from [Substrate Node
Template](https://github.com/substrate-developer-hub/substrate-node-template).
