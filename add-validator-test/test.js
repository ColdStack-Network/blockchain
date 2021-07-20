const assert = require('assert')
const {ApiPromise, RPCProvider, WsProvider, Keyring} = require('@polkadot/api');

const NODE_URL= process.env.NODE_URL;

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
  const keyring = new Keyring({ type: 'sr25519' })

  const wsProvider = new WsProvider(NODE_URL)
  const api = await ApiPromise.create({provider: wsProvider})

  const alice = keyring.addFromUri('//Alice')
  const bob = keyring.addFromUri('//Bob')

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

  console.log('rotating node keys')
  const keys = await api.rpc.author.rotateKeys()

  console.log('setting keys from //Bob account')
  await sendTxAndWait(bob, api.tx.session.setKeys(keys, '0x'))

  console.log('adding //Bob as a validator')
  await sendTxAndWait(alice, 
    api.tx.sudo.sudo(api.tx.validatorSet.addValidator(bob.address))
  )

  await api.disconnect()

  console.log("Finished, check Bob's node logs")

})()
