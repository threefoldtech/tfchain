FROM node:14 AS processor
WORKDIR /hydra-project
ADD package.json .
ADD package-lock.json .
RUN npm ci
ADD tsconfig.json .
ADD db db
ADD generated generated
ADD mappings mappings
ADD chain chain
ADD server-extension server-extension
RUN npx tsc
ADD manifest.yml .
ADD .env .
ENV HYDRA_NO_TS=true
CMD [ "npm", "run", "processor:start"]


FROM processor AS query-node
CMD ["node", "./generated/server/index.js"]