FROM node:14 AS processor
WORKDIR /hydra-project
ADD package.json .
ADD yarn.lock .
ADD tsconfig.json .
ADD db db
ADD generated generated
ADD mappings mappings
ADD chain chain
# RUN npx tsc
ADD manifest.yml .
ADD .env .
ADD init-countries.js .
ENV HYDRA_NO_TS=true
CMD [ "yarn", "processor:start"]


FROM processor AS query-node
CMD ["node", "./generated/server/index.js"]