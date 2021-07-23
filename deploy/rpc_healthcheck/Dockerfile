FROM node:14-slim

WORKDIR /rpc_healthcheck

COPY package.json package-lock.json ./

RUN npm install --only=prod

COPY index.js .

ENTRYPOINT ["node", "index.js"]
