const assert = require('assert')
const {ApiPromise, RPCProvider, WsProvider, Keyring} = require('@polkadot/api');
const {u8aToString} = require('@polkadot/util/u8a/toString')
const {BN} = require('bn.js')
const crypto = require('crypto')

const NODE_URL= process.env.NODE_URL;

console.log('NODE_URL', NODE_URL)

async function expectOk(promise){
  try {
    await promise
  } catch(e){
    console.log('FAIL: Caught error', e.toString())
    throw e
  }
}

async function expectFail(promise, string){
  try {
    await promise
  } catch(e){
    if(e.toString() != string){
      throw new Error(`FAIL: expected ${string}, caught ${e.toString()}`)
    }
    return
  }
  throw new Error(`FAIL: expected error "${string}" but not caught`)
}

(async () => {

  await require('@polkadot/wasm-crypto').waitReady()
  const keyring = new Keyring({ type: 'sr25519' });

  const wsProvider = new WsProvider(NODE_URL)
  const api = await ApiPromise.create({ provider: wsProvider });

  const alice = keyring.addFromUri('//Alice')
  const bob = keyring.addFromUri('//Bob')
  const bobEthAddress = '0x4444444444444444444444444444444444444444'

  async function sendTxAndWait(account, tx){
    return new Promise(async (resolve, reject) => {
      const unsub = await tx.signAndSend(account, (result) => {
        if (result.status.isInBlock) {
          let rejected = false
          result.events
          .filter(({event}) =>
            api.events.system.ExtrinsicFailed.is(event)
          )
          .forEach(({ event: { data: [error, info] } }) => {
            if (error.isModule) {
              const decoded = api.registry.findMetaError(error.asModule);
              const { documentation, method, section } = decoded;
              //console.log(`${section}.${method}: ${documentation.join(' ')}`);
              reject(`${section}.${method}`)
            } else {
              // Other, CannotLookup, BadOrigin, no extra info
              reject(error)
            }
            rejected = true
          })
          unsub();
          if(!rejected){
            resolve(result)
          }
        }
      })
    })
  }

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

  const FILE_CONTENTS = "loremipsum"
  const FILE_SIZE = FILE_CONTENTS.length
  const USER_ETH_ADDRESS = '0x3333333333333333333333333333333333333333'
  const GATEWAY_SEED_NODE = '0x2222222222222222222222222222222222222222'
  const GATEWAY_SEC_NODE =  '0x6666666666666666666666666666666666666666'

  console.log('register seed gateway node')

  await expectOk(
    sendTxAndWait(
      alice,
      api.tx.coldStack.registerGatewayNode(
        GATEWAY_SEED_NODE,
        null,
        'http://gateway_seed.test',
      )
    )
  )

  console.log('register non-seed gateway node')

  await expectOk(
    sendTxAndWait(
      alice,
      api.tx.coldStack.registerGatewayNode(
        GATEWAY_SEC_NODE,
        GATEWAY_SEED_NODE,
        'http://gateway_sec.test',
      )
    )
  )

  assert.deepEqual(
    await gatewayNodes(),
    [
      {
        nodeAddress: GATEWAY_SEC_NODE,
        seedAddress: GATEWAY_SEED_NODE,
        url: 'http://gateway_sec.test'
      },
      {
        nodeAddress: GATEWAY_SEED_NODE,
        seedAddress: null,
        url: 'http://gateway_seed.test'
      }
    ]
  )

  let uploadNumber = 0
  function upload(){
    const number = (uploadNumber++).toString()
    return api.tx.coldStack.upload(
      /*user_eth_address:*/   USER_ETH_ADDRESS,
      /*file_name_hash:*/     '0x' + crypto.createHash('sha256').update(number).digest('hex'),
      /*file_size_bytes:  */  FILE_SIZE,
      /*file_contents_hash:*/ '0x' + crypto.createHash('sha256').update(FILE_CONTENTS).digest('hex'),
      /*gateway_eth_address:*/GATEWAY_SEED_NODE,
    )
  }

  const testAddress = '0x1111111111111111111111111111111111111111'
  const testAddress2 = '0x5555555555555555555555555555555555555555'

  assert.equal((await api.query.coldStack.totalFileCount()).toNumber(), 0)
  assert.equal((await api.query.coldStack.totalFileSize()).toNumber(), 0)


  // Total issuance is equal to locked funds
  const totalIssuance = await api.query.coldStack.totalIssuance()
  assert.ok(totalIssuance.eq(await api.query.coldStack.lockedFunds()))

  // Alice can upload file because she is admin

  await expectOk(
    sendTxAndWait(
      alice,
      upload()
    )
  )

  assert.equal((await api.query.coldStack.totalFileCount()).toNumber(), 1)
  assert.equal((await api.query.coldStack.totalFileSize()).toNumber(), FILE_SIZE)

  console.log("alice succeed to upload file")

  // But Bob cannot


  await expectFail(
    sendTxAndWait(
      bob,
      upload()
    ),
    'coldStack.Unauthorized'
  )

  console.log("bob failed to upload file")

  // Let's grant permission to Bob

  await expectOk(
    sendTxAndWait(
      alice,
      api.tx.coldStack.grantFilePermission(bobEthAddress, bob.address, 'http://foo.bar')
    )
  )

  assert.equal(
    u8aToString(await api.query.coldStack.nodeURLs(bobEthAddress)), 
    'http://foo.bar'
  )

  console.log("alice granted file permission to bob")

  // Now Bob can upload too

  await expectOk(
    sendTxAndWait(
      bob,
      upload()
    )
  )

  console.log("bob succeed to upload file")

  await expectOk(
    sendTxAndWait(
      bob,
      api.tx.coldStack.delete(
      /*user_eth_address*/ USER_ETH_ADDRESS,
      /*file_name_hash :*/ '0x' + crypto.createHash('sha256').update("0").digest('hex'),
      )
    )
  )

  console.log("bob succeed to delete his file")

  console.log('now revoke bobs permission to upload file')

  await expectOk(
    sendTxAndWait(
      alice,
      api.tx.coldStack.revokeFilePermission(bobEthAddress)
    )
  )

  console.log('now bob cannot upload file')

  await expectFail(
    sendTxAndWait(
      bob,
      upload()
    ),
    'coldStack.Unauthorized'
  )

  // testAddress has zero balance

  assert.equal((await api.query.coldStack.balances(testAddress)).toNumber(), 0)

  // Deposit 1 to testAddress

  await expectOk(
    sendTxAndWait(
      alice,
      api.tx.coldStack.deposit(testAddress, 1)
    )
  )

  console.log("alice succeed to deposit 1 token")

  // now testAddress has balance eq to 1

  assert.equal((await api.query.coldStack.balances(testAddress)).toNumber(), 1)

  // And locked funds is equal to `totalIssuance - 1`

  assert.ok(totalIssuance.sub(new BN(1)).eq(await api.query.coldStack.lockedFunds()))

  // Try to withdraw 2 from testAddress and get InsufficientFunds

  await expectFail(
    sendTxAndWait(
      alice,
      api.tx.coldStack.withdraw(testAddress, 2)
    ),
    'coldStack.InsufficientFunds'
  )

  console.log("alice failed to withdraw 2 tokens")

  // Try to withdraw 1

  await expectOk(
    sendTxAndWait(
      alice,
      api.tx.coldStack.withdraw(testAddress, 1)
    )
  )

  console.log("alice succeeded to withdraw 1 token")

  // And get balance back to zero

  assert.equal((await api.query.coldStack.balances(testAddress)).toNumber(), 0)

  // And locked funds eq to totalIssuance

  assert.ok(totalIssuance.eq(await api.query.coldStack.lockedFunds()))


  console.log('Now deposit 10 to testAddress')

  await expectOk(
    sendTxAndWait(
      alice,
      api.tx.coldStack.deposit(testAddress, 10)
    )
  )

  console.log('And transfer to testAddress2')

  await expectOk(
    sendTxAndWait(
      alice,
      api.tx.coldStack.transfer(testAddress, testAddress2, 4)
    )
  )

  console.log('balances should change')

  assert.equal((await api.query.coldStack.balances(testAddress)).toNumber(), 6)
  assert.equal((await api.query.coldStack.balances(testAddress2)).toNumber(), 4)

  // Bob cannot give permissions to himself

  await expectFail(
    sendTxAndWait(
      bob,
      api.tx.coldStack.grantBillingPermission(bobEthAddress, bob.address, 'http://foo.bar')
    ),
    'coldStack.Unauthorized'
  )

  console.log("bob failed to grant permission to himself")

  // Bob cannot deposit until given permission

  await expectFail(
    sendTxAndWait(
      bob,
      api.tx.coldStack.deposit(testAddress, 1)
    ),
    'coldStack.Unauthorized'
  )

  console.log("bob failed to deposit 1 token")

  // Until we give him permission

  await expectOk(
    sendTxAndWait(
      alice,
      api.tx.coldStack.grantBillingPermission(bobEthAddress, bob.address, 'http://foo.bar')
    )
  )

  console.log("alice granted billing permission to bob")

  // And now he can deposit too

  await expectOk(
    sendTxAndWait(
      bob,
      api.tx.coldStack.deposit(testAddress, 50)
    )
  )

  console.log("bob succeed to deposit tokens")

  console.log('revoke bob billing permission')

  await expectOk(
    sendTxAndWait(
      alice,
      api.tx.coldStack.revokeBillingPermission(bobEthAddress)
    )
  )

  console.log('now bob cannot deposit')

  await expectFail(
    sendTxAndWait(
      bob,
      api.tx.coldStack.deposit(testAddress, 1)
    ),
    'coldStack.Unauthorized'
  )

  console.log('Tests passed')

  await api.disconnect()

})()
