FROM node:14 AS processor
WORKDIR /hydra-project
ADD package.json .
ADD package-lock.json .
RUN npm ci
ADD tsconfig.json .
ADD db db
ADD generated generated
ADD mappings mappings
ADD types types
ADD manifest.yml .
ADD .env .
CMD [ "npm", "run", "processor:start"]


FROM processor AS query-node
CMD ["npm", "run", "query-node:start"]