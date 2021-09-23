# Processor chart

## Building the images

```
cd tfchain/graphql
docker build . -t tfchainprocessor:$(git describe --abbrev=0 --tags | sed 's/^v//')  -f docker/Dockerfile.processor
docker build . -t tfchainquerynode:$(git describe --abbrev=0 --tags | sed 's/^v//')-f docker/Dockerfile.query-node
```

## Create PersistentVolumeClaim

```sh
cd tfchain/graphql/processor-chart
kubectl apply -f pvc-db-processor.yaml
```

## Install chart with helm

```sh
cd tfchain/graphql/processor-chart
helm install processor .
```

If indexer cannot reach Database, you can set `db_url` to the db-service cluster ip.

```sh
kubectl get svc
```

## NOTES

take note of the IP assigned the db-service. Use this IP in `values.yaml` for the db url.
