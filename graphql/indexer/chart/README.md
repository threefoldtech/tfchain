# Indexer stack chart

## Create types json configmap

```sh
kubectl create configmap indexer-config --from-file=./types.json
```

## Create PersistentVolumeClaims for the database

```sh
kubectl apply -f pvc-db.yaml
```

## Install chart with helm

```sh
helm install indexer .
```

If the indexer cannot reach the database, you can set `db_url` to the db-service cluster ip.

```sh
kubectl get svc
```

NOTE: take note of the IP assigned the db-service. Use this IP in `values.yaml` for the db_endpoint, ws_endpoint and indexer_status_service_url. Until DNS resolution works you can update these via 'helm upgrade'.
