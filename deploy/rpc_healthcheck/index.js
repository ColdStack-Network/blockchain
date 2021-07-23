import express from 'express'
import {ApiPromise, WsProvider} from '@polkadot/api'

if(!process.env.NODE_URL){
  console.error('env variable NODE_URL is not defined')
  process.exit(1)
}

const wsProvider = new WsProvider(process.env.NODE_URL)
const api = await ApiPromise.create({ provider: wsProvider })

const app = express()
app.use(express.json())

app.listen(process.env.PORT)

async function healthcheck(_, response){
  try {
    const number = await api.query.system.number()
    response.status(200).send('ok, blocknumber is ' + number)
  } catch(e) {
    response.status(502).send(e.toString())
  }
}

app.get('/healthcheck', healthcheck)
app.get('/', healthcheck)
