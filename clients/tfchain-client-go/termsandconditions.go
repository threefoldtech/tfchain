package substrate

import (
	"github.com/centrifuge/go-substrate-rpc-client/v4/types"
	"github.com/pkg/errors"
)

type TermsAndConditions struct {
	Account      AccountID `json:"account_id"`
	Timestamp    types.U64 `json:"timestamp"`
	DocumentLink string    `json:"document_link"`
	DocumentHash string    `json:"document_hash"`
}

// AcceptTermsAndConditions accepts terms and conditions
func (s *Substrate) AcceptTermsAndConditions(identity Identity, documentLink string, documentHash string) error {
	cl, meta, err := s.GetClient()
	if err != nil {
		return err
	}

	c, err := types.NewCall(meta, "TfgridModule.user_accept_tc",
		documentLink, documentHash,
	)

	if err != nil {
		return errors.Wrap(err, "failed to create call")
	}

	_, err = s.Call(cl, meta, identity, c)
	if err != nil {
		return errors.Wrap(err, "failed to accept terms and conditions")
	}

	return nil
}

// SignedTermsAndConditions return list of signed terms and conditions for this account
func (s *Substrate) SignedTermsAndConditions(account AccountID) ([]TermsAndConditions, error) {
	cl, meta, err := s.GetClient()
	if err != nil {
		return nil, err
	}

	bytes, err := types.Encode(account)
	if err != nil {
		return nil, errors.Wrap(err, "substrate: encoding error building query arguments")
	}
	key, err := types.CreateStorageKey(meta, "TfgridModule", "UsersTermsAndConditions", bytes, nil)
	if err != nil {
		return nil, errors.Wrap(err, "failed to create substrate query key")
	}

	raw, err := cl.RPC.State.GetStorageRawLatest(key)
	if err != nil {
		return nil, errors.Wrap(err, "failed to lookup terms and conditions")
	}

	if len(*raw) == 0 {
		// no signatures for this account
		return nil, nil
	}

	var conditions []TermsAndConditions
	if err := types.Decode(*raw, &conditions); err != nil {
		return nil, errors.Wrap(err, "failed to load object")
	}

	return conditions, nil
}
