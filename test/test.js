const {ApiPromise, RPCProvider, WsProvider, Keyring} = require('@polkadot/api');

const NODE_URL= process.env.NODE_URL;

async function expectOk(promise){
  try {
    await promise
  } catch(e){
    console.log('FAIL: Caught error', e.toString())
    process.exit(1)
  }
}

async function expectFail(promise, string){
  try {
    await promise
    console.log('FAIL: expected error', string, 'but not caught')
    process.exit(1)
  } catch(e){
    if(e.toString() != string){
      console.log('eq', e.toString() == string)
      console.log(e.toString(), typeof(e.toString()))
      console.log(string, typeof(string))
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

  async function sendTxAndWait(account, tx){
    return new Promise(async (resolve, reject) => {
      const unsub = await tx.signAndSend(account, (result) => {
        //if (result.status.isFinalized) {
          //console.log(`Transaction finalized at blockHash ${result.status.asFinalized}`);
        //}
        if (result.status.isInBlock) {
          //console.log(`Transaction included at blockHash ${result.status.asInBlock}`);
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
          //console.log('events', result.events.map(e => JSON.stringify(e.event)))
          unsub();
          if(!rejected){
            resolve(result)
          }
        }
      })
    })
  }

  const testAddress = '0x1111111111111111111111111111111111111111'

  // Alice can upload file because she is admin

  console.log("initialized")

  await expectOk(
    sendTxAndWait(
      alice,
      api.tx.coldStack.upload(
        testAddress,
        '0x11111111111111111111111111111111',
        '0x22222222222222222222222222222222',
        1
      )
    )
  )

  console.log("alice succeed to upload file")

  // But Bob cannot


  await expectFail(
    sendTxAndWait(
      bob,
      api.tx.coldStack.upload(
        testAddress,
        '0x11111111111111111111111111111111',
        '0x22222222222222222222222222222222',
        1
      )
    ),
    'bob upload - coldStack.Unauthorized'
  )

  console.log("bob failed to upload file")

  // Let's grant permission to Bob

  await expectOk(
    sendTxAndWait(
      alice,
      api.tx.coldStack.grantFilePermission(bob.address)
    )
  )

  console.log("alice granted file permission to bob")

  // Now Bob can upload too

  await expectOk(
    sendTxAndWait(
      bob,
      api.tx.coldStack.upload(
        testAddress,
        '0x11111111111111111111111111111111',
        '0x22222222222222222222222222222222',
        1
      )
    )
  )

  console.log("bob succeed to upload file")

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

  // Try to withdraw 2 from testAddress and get InsufficientFunds

  await expectFail(
    sendTxAndWait(
      alice,
      api.tx.coldStack.withdraw(testAddress, 2)
    ),
    'coldStack.InsufficientFunds'
  )

  console.log("alice failed to deposit 2 tokens")

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

  // Bob cannot give permissions to himself

  await expectFail(
    sendTxAndWait(
      bob,
      api.tx.coldStack.grantBillingPermission(bob.address)
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
      api.tx.coldStack.grantBillingPermission(bob.address)
    )
  )

  console.log("alice granted billing permission to bob")

  // And now he can deposit too


  await expectOk(
    sendTxAndWait(
      bob,
      api.tx.coldStack.deposit(testAddress, 1)
    )
  )

  console.log("bob succeed to deposit 1 token")

  console.log('Tests passed')

  process.exit(0)

})()
