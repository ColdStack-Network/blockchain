const {ApiPromise, RPCProvider, WsProvider, Keyring} = require('@polkadot/api');
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
    console.log('FAIL: expected error', string, 'but not caught')
    process.exit(1)
  } catch(e){
    if(e.toString() != string){
      console.log(`FAIL: expected ${string}, caught ${e.toString()}`)
      process.exit(1)
    }
  }
}

function assertEq(expected, actual){
  if(expected != actual){
    throw new Error('Expected ${expected} but actual is ${actual}')
  }
}

function assert(cond, message){
  if(!cond){
    throw new Error(message)
  }
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

  const FILE_CONTENTS = "loremipsum"
  const FILE_SIZE = FILE_CONTENTS.length
  const USER_ETH_ADDRESS = '0x3333333333333333333333333333333333333333'

  let uploadNumber = 0
  function upload(){
    const number = (uploadNumber++).toString()
    return api.tx.coldStack.upload(
      /*user_eth_address:*/   USER_ETH_ADDRESS,
      /*file_name_hash:*/     '0x' + crypto.createHash('sha256').update(number).digest('hex'),
      /*file_size_bytes:  */  FILE_SIZE,
      /*file_contents_hash:*/ '0x' + crypto.createHash('sha256').update(FILE_CONTENTS).digest('hex'),
      /*gateway_eth_address:*/'0x2222222222222222222222222222222222222222',
    )
  }

  const testAddress = '0x1111111111111111111111111111111111111111'

  assertEq((await api.query.coldStack.totalFileCount()).toNumber(), 0)
  assertEq((await api.query.coldStack.totalFileSize()).toNumber(), 0)


  // Total issuance is equal to locked funds
  const totalIssuance = (await api.query.coldStack.totalIssuance()).toNumber()
  assertEq(totalIssuance, (await api.query.coldStack.lockedFunds()).toNumber())

  // Alice can upload file because she is admin

  console.log("initialized")

  await expectOk(
    sendTxAndWait(
      alice,
      upload()
    )
  )

  assertEq((await api.query.coldStack.totalFileCount()).toNumber(), 1)
  assertEq((await api.query.coldStack.totalFileSize()).toNumber(), FILE_SIZE)

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
      api.tx.coldStack.grantFilePermission(bobEthAddress, bob.address)
    )
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

  assert((await api.query.coldStack.balances(testAddress)).eq(0), 'Unexpected balance')

  // Deposit 1 to testAddress

  await expectOk(
    sendTxAndWait(
      alice,
      api.tx.coldStack.deposit(testAddress, 1)
    )
  )

  console.log("alice succeed to deposit 1 token")

  // now testAddress has balance eq to 1

  assert((await api.query.coldStack.balances(testAddress)).eq(1), 'Unexpected balance')

  // And locked funds is equal to `totalIssuance - 1`

  assertEq(totalIssuance - 1, (await api.query.coldStack.lockedFunds()).toNumber())

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

  assert((await api.query.coldStack.balances(testAddress)).eq(0), 'Unexpected balance')

  // And locked funds eq to totalIssuance

  assertEq(totalIssuance, (await api.query.coldStack.lockedFunds()).toNumber())

  // Bob cannot give permissions to himself

  await expectFail(
    sendTxAndWait(
      bob,
      api.tx.coldStack.grantBillingPermission(bobEthAddress, bob.address)
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
      api.tx.coldStack.grantBillingPermission(bobEthAddress, bob.address)
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

  process.exit(0)

})()
