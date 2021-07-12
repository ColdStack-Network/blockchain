const assert = require('assert')
const fs = require('fs')
const {ApiPromise, WsProvider, Keyring} = require('@polkadot/api');
const {BN} = require('bn.js')
const crypto = require('crypto')

const codePath = process.argv[2]

async function expectOk(promise){
  try {
    return await promise
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

async function sendTxAndWait(api, account, tx){
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

async function withAPI(action){
  const wsProvider = new WsProvider()
  const api = await ApiPromise.create({ 
    provider: wsProvider,
    types: {
      Gateway: {
        address: 'Vec<u8>',
        seed_address: 'Option<Vec<u8>>'
      },
    }


  });
  try {
    await action(api)
  } finally {
    await api.disconnect()
  }
}

async function main(){

  await require('@polkadot/wasm-crypto').waitReady()
  const keyring = new Keyring({ type: 'sr25519' });

  const alice = keyring.addFromUri('//Alice')
  const ferdie = keyring.addFromUri('//Ferdie')


  await withAPI(async api => {

    assert.equal(api.tx.coldStack.test, null)

    const GATEWAY = '0x1111111111111111111111111111111111111111'
    const SEED_GATEWAY = '0x2222222222222222222222222222222222222222'

    await sendTxAndWait(
      api,
      alice,
      api.tx.coldStack.registerGatewayNode(
        GATEWAY,
        SEED_GATEWAY,
        'http://gateway_seed.test',
      )
    )

    const seed = await api.query.coldStack.gatewayNodeSeeds(GATEWAY)
    assert.equal(seed, SEED_GATEWAY)

    async function setCode(){
      const adminId = await api.query.sudo.key();
      const adminPair = keyring.getPair(adminId.toString());
      const code = fs.readFileSync(codePath).toString('hex');
      const tx = api.tx.system.setCode(`0x${code}`)
      console.log(`Upgrading from ${adminId}, ${code.length / 2} bytes`);
      return await sendTxAndWait(api, adminPair, api.tx.sudo.sudoUncheckedWeight(tx, 0))
    }

    console.log('transfer funds to alice account so it can pay for upgrade')
    await sendTxAndWait(api, ferdie, api.tx.balances.transfer(alice.address, 1000000000))

    console.log('upgrading runtime')
    await setCode()
    console.log('runtime upgraded')

    const oldSeedData = await api.query.coldStack.gatewayNodeSeeds(GATEWAY)
    assert.equal(oldSeedData.isSome, false)

    const gateway = await api.query.coldStack.gateways(GATEWAY)
    assert.equal(gateway.address.toString('hex'), GATEWAY)
    assert.equal(
      gateway.seed_address.isSome && gateway.seed_address.unwrap().toString('hex'),
      SEED_GATEWAY,
    )


    console.log('invoking new method')
    const PARAM = 42
    const result = await expectOk(
      sendTxAndWait(
        api,
        alice,
        api.tx.coldStack.test(PARAM),
      )
    )
    const testEvent = result.events.find(({event}) => api.events.coldStack.Test.is(event))
    assert.equal(testEvent.event.data[0].toNumber(), PARAM)

  })

  console.log('Test passed')


}

main().catch(e => console.log(e))
