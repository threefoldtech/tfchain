# Indexer stack chart

## Create types json configmap

```sh
kubectl create configmap indexer-config --from-file=./types.json
```

## Install chart with helm

```sh
helm install indexer .
```

If indexer cannot reach Database, you can set `db_url` to the db-service cluster ip.

```sh
kubectl get svc
```

take note of the IP assigned the db-service. Use this IP in `values.yaml` for the db url.