package substrate

import (
	"bytes"
	"encoding/hex"

	"github.com/centrifuge/go-substrate-rpc-client/v4/scale"
)

func Encode(value interface{}) ([]byte, error) {
	var bytes bytes.Buffer
	enc := scale.NewEncoder(&bytes)

	if err := enc.Encode(value); err != nil {
		return nil, err
	}

	return bytes.Bytes(), nil
}

func Decode(data []byte, value interface{}) error {
	buffer := bytes.NewBuffer(data)
	enc := scale.NewDecoder(buffer)
	return enc.Decode(value)
}

func HexEncodeToString(b []byte) string {
	return "0x" + hex.EncodeToString(b)
}
