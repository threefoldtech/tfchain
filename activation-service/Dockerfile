# @polkadot/api 15.x and @polkadot/util-crypto, pulled in by
# @threefold/tfchain_client, both declare engines node >= 18.
FROM node:22

# Create app directory
WORKDIR /usr/src/app

# Install app dependencies
COPY package.json yarn.lock ./

RUN yarn install && yarn cache clean

# If you are building your code for production
# RUN npm ci --only=production

# Bundle app source
COPY . .

EXPOSE 3000

CMD [ "yarn", "start-prod" ]