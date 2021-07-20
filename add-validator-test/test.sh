#!/bin/bash

set -e

trap "kill -- -$$" EXIT

PROFILE=debug

rm -rf tmp && mkdir tmp

./target/$PROFILE/node-template --chain dev \
  --alice --port 30331 --base-path tmp/alice              > tmp/alice-log 2>&1 &

./target/$PROFILE/node-template --chain dev \
  --bob   --port 30332 --base-path tmp/bob --ws-port 9944 > tmp/bob-log   2>&1 &

cd $(dirname "${BASH_SOURCE[0]}")
node --unhandled-rejections=strict test.js

sleep infinity

# tmp/bob-log must eventually produce messages like this:


# 2021-07-20 16:13:30 🙌 Starting consensus session on top of parent 0x5c57b5e67d980c3d9a78d03994d18fa729e8c8db97d13160c752d33f3313e11a
# 2021-07-20 16:13:30 🎁 Prepared block for proposing at 10 [hash: 0xefa7ccc070ca5126533b714a8ca88840a7e5ba113c005f6ddd4fae6b823a208a; parent_hash: 0x5c57…e11a; extrinsics (1): [0xdd9a…8943]]
# 2021-07-20 16:13:30 🔖 Pre-sealed block for proposal at 10. Hash now 0xb4119ff7d5f3b444f23e8389f9681b28cc8c153850610bd13f72c15bab6a071d, previously 0xefa7ccc070ca5126533b714a8ca88840a7e5ba113c005f6ddd4fae6b823a208a.
