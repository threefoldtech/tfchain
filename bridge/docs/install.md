# Install

## From Source

You'll need `go` [installed](https://golang.org/doc/install) and the required
environment variables set, which can be done with the following commands:

```sh
echo export GOPATH=\"\$HOME/go\" >> ~/.bash_profile
echo export PATH=\"\$PATH:\$GOPATH/bin\" >> ~/.bash_profile
```

### Get Source Code

```sh
git clone https://github.com/threefoldtech/tfchain.git
cd bridge/tfchain_bridge
```

### Compile

```sh
go build .
```
