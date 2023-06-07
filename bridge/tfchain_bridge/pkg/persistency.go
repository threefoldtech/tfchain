package pkg

import (
	"encoding/json"
	"os"
)

type Blockheight struct {
	LastHeight    uint32 `json:"lastHeight"`
	StellarCursor string `json:"stellarCursor"`
}

type ChainPersistency struct {
	location string
}

func InitPersist(location string) (*ChainPersistency, error) {
	return &ChainPersistency{
		location: location,
	}, nil
}

func (b *ChainPersistency) SaveHeight(height uint32) error {
	blockheight, err := b.GetHeight()
	if err != nil {
		return err
	}

	blockheight.LastHeight = height
	return b.Save(blockheight)
}

func (b *ChainPersistency) SaveStellarCursor(cursor string) error {
	blockheight, err := b.GetHeight()
	if err != nil {
		return err
	}

	blockheight.StellarCursor = cursor
	return b.Save(blockheight)
}

func (b *ChainPersistency) GetHeight() (*Blockheight, error) {
	var blockheight Blockheight
	file, err := os.ReadFile(b.location)
	if os.IsNotExist(err) {
		return &blockheight, nil
	}
	if err != nil {
		return nil, err
	}
	err = json.Unmarshal(file, &blockheight)
	if err != nil {
		return nil, err
	}

	return &blockheight, nil
}

func (b *ChainPersistency) Save(blockheight *Blockheight) error {
	updatedPersistency, err := json.Marshal(blockheight)
	if err != nil {
		return err
	}

	return os.WriteFile(b.location, updatedPersistency, 0644)
}
