FROM node:14-alpine 

RUN mkdir -p /home/hydra-builder && chown -R node:node /home/hydra-builder

WORKDIR /home/hydra-builder

COPY ./mappings ./mappings
COPY ./*.yml ./
COPY ./*.json ./
COPY ./*.graphql ./
COPY ./.env ./
COPY ./package.json ./
COPY ./yarn.lock ./

# create temporary package so it can be depended on by mappings until a proper server is generated
# via `yarn codegen`
# Workaround for https://github.com/Joystream/hydra/issues/440
RUN mkdir -p ./generated/graphql-server
RUN echo '{"name": "query-node", "version": "0.0.1"}' > ./generated/graphql-server/package.json

RUN yarn
RUN yarn codegen

# Workaround for https://github.com/Joystream/hydra/issues/440
RUN sed -i 's#"version": "0.0.0"#"version": "0.0.1",\n"  main": "dist/model/index.js",\n  "types": "dist/model/index.d.ts"#' ./generated/graphql-server/package.json
RUN yarn # make sure generated package(s) are included

# load env settings
RUN set -a
RUN . ./.env
RUN set +a

#RUN yarn mappings:build

RUN yarn workspace query-node install
RUN yarn workspace query-node compile

# Workaround for https://github.com/Joystream/hydra/issues/441
RUN cd node_modules/@subsquid/hydra-typegen && yarn add ws
RUN yarn typegen

RUN yarn mappings:build
