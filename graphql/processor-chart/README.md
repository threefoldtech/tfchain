# Processor chart

## Building the images

In the graphql folder (`..`):

```sh
docker build . -t tfchainprocessor:$(git describe --abbrev=0 --tags | sed 's/^v//') -f docker/Dockerfile.processor
docker build . -t tfchainquerynode:$(git describe --abbrev=0 --tags | sed 's/^v//') -f docker/Dockerfile.query-node
```

## Push images to the other k8s nodes

```sh
docker save tfchainprocessor:$(git describe --abbrev=0 --tags | sed 's/^v//') | ssh -C ubuntu@xx.xx.xx.xx docker load
docker save tfchainquerynode:$(git describe --abbrev=0 --tags | sed 's/^v//') | ssh -C ubuntu@xx.xx.xx.xx docker load
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

If the indexer cannot reach the database, you can set `db_url` to the db-service cluster ip.

```sh
kubectl get svc
```

## NOTES

take note of the IP assigned the db-service. Use this IP in `values.yaml` for the db url.
