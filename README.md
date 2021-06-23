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

### Get list of gateway nodes:

```
const {u8aToString} = require('@polkadot/util/u8a/toString')

async function gatewayNodes(){
  const nodeEntries = await api.query.coldStack.gatewayNodeSeeds.entries()
  return Promise.all(nodeEntries.map(async ([k,v]) => {
    const nodeAddress = k.args.toString('hex')
    return {
      nodeAddress,
      seedAddress: v.isNone ? null : v.toString('hex'),
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

# Substrate Node Template

Repository is forked from [Substrate Node
Template](https://github.com/substrate-developer-hub/substrate-node-template).
