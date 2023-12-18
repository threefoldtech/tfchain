package stellar

import (
	"context"
	"encoding/base64"
	"encoding/hex"
	"fmt"
	"math/big"
	"strconv"
	"strings"
	"time"

	"github.com/pkg/errors"
	"github.com/rs/zerolog"
	"github.com/rs/zerolog/log"
	"github.com/stellar/go/amount"
	"github.com/stellar/go/clients/horizonclient"
	"github.com/stellar/go/keypair"
	"github.com/stellar/go/network"
	hProtocol "github.com/stellar/go/protocols/horizon"
	horizoneffects "github.com/stellar/go/protocols/horizon/effects"
	"github.com/stellar/go/protocols/horizon/operations"
	"github.com/stellar/go/txnbuild"
	substrate "github.com/threefoldtech/tfchain/clients/tfchain-client-go"
	"github.com/threefoldtech/tfchain_bridge/pkg"
)

const (
	TFTMainnet = "TFT:GBOVQKJYHXRR3DX6NOX2RRYFRCUMSADGDESTDNBDS6CDVLGVESRTAC47"
	TFTTest    = "TFT:GA47YZA3PKFUZMPLQ3B5F2E3CJIB57TGGU7SPCQT2WAEYKN766PWIMB3"

	stellarPrecision       = 1e7
	stellarPrecisionDigits = 7
)

// stellarWallet is the bridge wallet
// Payments will be funded and fees will be taken with this wallet
type StellarWallet struct {
	keypair        *keypair.Full
	config         *pkg.StellarConfig
	signatureCount int
	sequenceNumber int64
}

func NewStellarWallet(ctx context.Context, config *pkg.StellarConfig) (*StellarWallet, error) {
	kp, err := keypair.ParseFull(config.StellarSeed)

	if err != nil {
		return nil, err
	}

	w := &StellarWallet{
		keypair: kp,
		config:  config,
	}

	account, err := w.getAccountDetails(config.StellarBridgeAccount)
	if err != nil {
		return nil, err
	}
	log.Info().Msgf("required signature count %d", int(account.Thresholds.MedThreshold))
	// If no threshold is set (0) we can asume it's a "normal" account without options
	// set the signature count to 1
	if int(account.Thresholds.MedThreshold) == 0 {
		w.signatureCount = 1
	} else {
		w.signatureCount = int(account.Thresholds.MedThreshold)
	}

	w.sequenceNumber, err = account.GetSequenceNumber()
	if err != nil {
		return nil, err
	}
	log.Info().Msgf("account %s loaded with sequence number %d", account.AccountID, w.sequenceNumber)

	return w, nil
}

func (w *StellarWallet) CreatePaymentAndReturnSignature(ctx context.Context, target string, amount uint64, txID uint64) (string, uint64, error) {
	txnBuild, err := w.generatePaymentOperation(amount, target, 0)
	if err != nil {
		return "", 0, err
	}

	txn, err := w.createTransaction(ctx, txnBuild, true)
	if err != nil {
		return "", 0, err
	}

	signatures := txn.Signatures()

	return base64.StdEncoding.EncodeToString(signatures[0].Signature), uint64(txn.SequenceNumber()), nil
}

func (w *StellarWallet) CreatePaymentWithSignaturesAndSubmit(ctx context.Context, target string, amount uint64, txHash string, signatures []substrate.StellarSignature, sequenceNumber int64) error {
	ctx_with_trace_id := context.WithValue(ctx, "trace_id", txHash)

	txnBuild, err := w.generatePaymentOperation(amount, target, sequenceNumber)
	if err != nil {
		return err
	}

	txn, err := w.createTransaction(ctx, txnBuild, false)
	if err != nil {
		return err
	}

	if len(signatures) < w.signatureCount {
		return errors.New("not enough signatures, aborting")
	}

	requiredSignatures := signatures[:w.signatureCount]
	for _, sig := range requiredSignatures {
		log.Debug().Str("signature", string(sig.Signature)).Str("address", string(sig.StellarAddress)).Msg("adding signature")
		txn, err = txn.AddSignatureBase64(w.getNetworkPassPhrase(), string(sig.StellarAddress), string(sig.Signature))
		if err != nil {
			return err
		}
	}

	return w.submitTransaction(ctx_with_trace_id, txn)
}

func (w *StellarWallet) CreateRefundPaymentWithSignaturesAndSubmit(ctx context.Context, target string, amount uint64, txHash string, signatures []substrate.StellarSignature, sequenceNumber int64) error {
	ctx_with_trace_id := context.WithValue(ctx, "trace_id", txHash)
	txnBuild, err := w.generatePaymentOperation(amount, target, sequenceNumber)
	if err != nil {
		return err
	}

	parsedMessage, err := hex.DecodeString(txHash)
	if err != nil {
		return err
	}

	var memo [32]byte
	copy(memo[:], parsedMessage)

	txnBuild.Memo = txnbuild.MemoReturn(memo)

	txn, err := w.createTransaction(ctx, txnBuild, false)
	if err != nil {
		return err
	}

	if len(signatures) < w.signatureCount {
		return errors.New("not enough signatures, aborting")
	}

	requiredSignatures := signatures[:w.signatureCount]
	for _, sig := range requiredSignatures {
		log.Debug().Msgf("adding signature %s, account %s", string(sig.Signature), string(sig.StellarAddress))
		txn, err = txn.AddSignatureBase64(w.getNetworkPassPhrase(), string(sig.StellarAddress), string(sig.Signature))
		if err != nil {
			return err
		}
	}

	return w.submitTransaction(ctx_with_trace_id, txn)
}

func (w *StellarWallet) CreateRefundAndReturnSignature(ctx context.Context, target string, amount uint64, message string) (string, uint64, error) {
	txnBuild, err := w.generatePaymentOperation(amount, target, 0)
	if err != nil {
		return "", 0, err
	}

	parsedMessage, err := hex.DecodeString(message)
	if err != nil {
		return "", 0, err
	}

	var memo [32]byte
	copy(memo[:], parsedMessage)

	txnBuild.Memo = txnbuild.MemoReturn(memo)

	txn, err := w.createTransaction(ctx, txnBuild, true)
	if err != nil {
		return "", 0, err
	}

	signatures := txn.Signatures()

	return base64.StdEncoding.EncodeToString(signatures[0].Signature), uint64(txn.SequenceNumber()), nil
}

func (w *StellarWallet) CheckAccount(account string) error {
	acc, err := w.getAccountDetails(account)
	if err != nil {
		return err
	}

	asset := w.getAssetCodeAndIssuer()

	for _, balance := range acc.Balances {
		if balance.Code != asset[0] || balance.Issuer != asset[1] {
			continue
		}
		limit, err := strconv.ParseFloat(balance.Limit, 64)
		if err != nil {
			//probably an empty string.
			continue
		}
		if limit > 0 {
			//valid address
			return nil
		}
	}

	return fmt.Errorf("addess has no trustline")
}

func (w *StellarWallet) generatePaymentOperation(amount uint64, destination string, sequenceNumber int64) (txnbuild.TransactionParams, error) {
	// if amount is zero, do nothing
	if amount == 0 {
		return txnbuild.TransactionParams{}, errors.New("invalid amount")
	}

	sourceAccount, err := w.getAccountDetails(w.config.StellarBridgeAccount)
	if err != nil {
		return txnbuild.TransactionParams{}, errors.Wrap(err, "an error occurred while getting source account details")
	}

	asset := w.getAssetCodeAndIssuer()

	var paymentOperations []txnbuild.Operation
	paymentOP := txnbuild.Payment{
		Destination: destination,
		Amount:      big.NewRat(int64(amount), stellarPrecision).FloatString(stellarPrecisionDigits),
		Asset: txnbuild.CreditAsset{
			Code:   asset[0],
			Issuer: asset[1],
		},
		SourceAccount: sourceAccount.AccountID,
	}
	paymentOperations = append(paymentOperations, &paymentOP)

	if sequenceNumber == 0 {
		w.sequenceNumber = w.sequenceNumber + 1
	} else {
		w.sequenceNumber = int64(sequenceNumber)
	}

	txnBuild := txnbuild.TransactionParams{
		Operations:           paymentOperations,
		Timebounds:           txnbuild.NewInfiniteTimeout(),
		SourceAccount:        &txnbuild.SimpleAccount{AccountID: sourceAccount.AccountID, Sequence: w.sequenceNumber},
		BaseFee:              txnbuild.MinBaseFee * 1000,
		IncrementSequenceNum: false,
	}

	return txnBuild, nil
}

func (w *StellarWallet) createTransaction(ctx context.Context, txn txnbuild.TransactionParams, sign bool) (*txnbuild.Transaction, error) {
	tx, err := txnbuild.NewTransaction(txn)
	if err != nil {
		return nil, errors.Wrap(err, "an error occurred while building the transaction")
	}

	if sign {
		tx, err = tx.Sign(w.getNetworkPassPhrase(), w.keypair)
		if err != nil {
			if hError, ok := err.(*horizonclient.Error); ok {
				log.Error().Msgf("Error submitting tx %+v", hError.Problem.Extras)
			}
			return nil, errors.Wrap(err, "an error occurred while signing the transaction with keypair")
		}
	}

	return tx, nil
}

func (w *StellarWallet) submitTransaction(ctx context.Context, txn *txnbuild.Transaction) error {
	client, err := w.getHorizonClient()
	if err != nil {
		return errors.Wrap(err, "an error occurred while getting horizon client")
	}

	// Submit the transaction
	txResult, err := client.SubmitTransaction(txn)
	if err != nil {
		log.Info().Msg(err.Error())
		if hError, ok := err.(*horizonclient.Error); ok {
			if ok {
				log.Err(err).Msgf("error while submitting transaction %+v", hError.Problem.Extras)
			}
		}
		errSequence := w.resetAccountSequence()
		if errSequence != nil {
			return errSequence
		}
		return errors.Wrap(err, "an error occurred while submitting the transaction")
	}
	log.Info().
		Str("trace_id", fmt.Sprint(ctx.Value("trace_id"))).
		Str("event_action", "stellar_transaction_submitted").
		Str("event_kind", "event").
		Str("category", "vault").
		Dict("metadata", zerolog.Dict().
			Str("result_tx_id", txResult.ID)).
		Msgf("the transaction submitted to the Stellar network, and its unique identifier is %s", txResult.ID)
	return nil
}

func (w *StellarWallet) resetAccountSequence() error {
	log.Info().Msgf("resetting account sequence")
	account, err := w.getAccountDetails(w.config.StellarBridgeAccount)
	if err != nil {
		return err
	}

	w.sequenceNumber, err = account.GetSequenceNumber()
	if err != nil {
		return err
	}

	return nil
}

func (w *StellarWallet) GetKeypair() *keypair.Full {
	return w.keypair
}

type MintEventSubscription struct {
	Events []MintEvent
	Err    error
}

type MintEvent struct {
	Senders map[string]*big.Int
	Tx      hProtocol.Transaction
	Error   error
}

// getAccountDetails gets account details based an a Stellar address
func (w *StellarWallet) getAccountDetails(address string) (account hProtocol.Account, err error) {
	client, err := w.getHorizonClient()
	if err != nil {
		return hProtocol.Account{}, err
	}
	ar := horizonclient.AccountRequest{AccountID: address}
	account, err = client.AccountDetail(ar)
	if err != nil {
		return hProtocol.Account{}, errors.Wrapf(err, "failed to get account details for account: %s", address)
	}
	return account, nil
}

func (w *StellarWallet) StreamBridgeStellarTransactions(ctx context.Context, mintChan chan<- MintEventSubscription, cursor string) error {
	client, err := w.getHorizonClient()
	if err != nil {
		return err
	}

	opRequest := horizonclient.TransactionRequest{
		ForAccount: w.config.StellarBridgeAccount,
		Cursor:     cursor,
	}

	for {
		select {
		case <-ctx.Done():
			return ctx.Err()
		default:
			response, err := client.Transactions(opRequest)
			if err != nil {
				log.Logger.Warn().
					Err(err).
					Str("event_action", "fetch_transactions_failed").
					Str("event_kind", "alert").
					Str("category", "stellar_monitor").
					Dict("metadata", zerolog.Dict().
						Str("cursor", opRequest.Cursor)).
					Msg("encountered an error while retrieving transactions for bridge Stellar account, retrying in 5 sec")
				select {
				case <-ctx.Done():
					return ctx.Err()
				case <-time.After(5 * time.Second):
					continue
				}
			}

			log.Logger.Info().
				Str("event_action", "transactions_fetched").
				Str("event_kind", "event").
				Str("category", "stellar_monitor").
				Dict("metadata", zerolog.Dict().
					Str("cursor", opRequest.Cursor).
					Int("count", len(response.Embedded.Records))).
				Msg("stellar transactions fetched")

			for _, tx := range response.Embedded.Records {
				mintEvents, err := w.processTransaction(tx)
				if err != nil {
					return err
				}
				mintChan <- MintEventSubscription{
					Events: mintEvents,
				}
				opRequest.Cursor = tx.PagingToken()
			}

			if len(response.Embedded.Records) == 0 {
				select {
				case <-ctx.Done():
					return ctx.Err()
				case <-time.After(10 * time.Second):
				}
			}
		}
	}
}

func (w *StellarWallet) processTransaction(tx hProtocol.Transaction) ([]MintEvent, error) {
	logger := log.Logger.With().Str("trace_id", tx.ID).Logger()

	if !tx.Successful {
		return nil, nil
	}

	effects, err := w.getTransactionEffects(tx.Hash)
	if err != nil {
		return nil, errors.Wrapf(err, "failed to fetch transaction effects for transaction with id is %s", tx.ID)
	}

	asset := w.getAssetCodeAndIssuer()

	var mintEvents []MintEvent
	for _, effect := range effects.Embedded.Records {
		if effect.GetAccount() != w.config.StellarBridgeAccount {
			continue
		}

		if effect.GetType() != "account_credited" {
			continue
		}

		creditedEffect := effect.(horizoneffects.AccountCredited)
		if creditedEffect.Asset.Code != asset[0] && creditedEffect.Asset.Issuer != asset[1] {
			continue
		}

		ops, err := w.getOperationEffect(tx.Hash)
		if err != nil {
			continue
		}

		senders := make(map[string]*big.Int)
		for _, op := range ops.Embedded.Records {
			if op.GetType() != "payment" {
				return nil, nil
			}

			PaymentOperation := op.(operations.Payment)
			if PaymentOperation.To != w.config.StellarBridgeAccount {
				continue
			}

			parsedAmount, err := amount.ParseInt64(PaymentOperation.Amount)
			if err != nil {
				continue
			}

			depositedAmount := big.NewInt(int64(parsedAmount))
			logger.Info().
				Str("event_action", "payment_received").
				Str("event_kind", "event").
				Str("category", "vault").
				Dict("metadata", zerolog.Dict().
					Str("from", PaymentOperation.From).
					Str("amount", PaymentOperation.Amount)).
				Str("tx_hash", PaymentOperation.TransactionHash).
				Str("ledger_close_time", PaymentOperation.LedgerCloseTime.String()).
				Msg("a payment has received on bridge Stellar account")
			if _, ok := senders[PaymentOperation.From]; !ok {
				senders[PaymentOperation.From] = depositedAmount
			} else {
				senderAmount := senders[PaymentOperation.From]
				senderAmount = senderAmount.Add(senderAmount, depositedAmount)
				senders[PaymentOperation.From] = senderAmount
			}
		}

		mintEvents = append(mintEvents, MintEvent{
			Senders: senders,
			Tx:      tx,
			Error:   nil,
		})
	}

	return mintEvents, nil
}

func (w *StellarWallet) getTransactionEffects(txHash string) (effects horizoneffects.EffectsPage, err error) {
	client, err := w.getHorizonClient()
	if err != nil {
		return effects, err
	}

	effectsReq := horizonclient.EffectRequest{
		ForTransaction: txHash,
	}
	effects, err = client.Effects(effectsReq)
	if err != nil {
		return effects, err
	}

	return effects, nil
}

func (w *StellarWallet) getOperationEffect(txHash string) (ops operations.OperationsPage, err error) {
	client, err := w.getHorizonClient()
	if err != nil {
		return ops, err
	}

	opsRequest := horizonclient.OperationRequest{
		ForTransaction: txHash,
	}
	ops, err = client.Operations(opsRequest)
	if err != nil {
		return ops, err
	}

	return ops, nil
}

// getHorizonClient gets the horizon client based on the wallet's network
func (w *StellarWallet) getHorizonClient() (*horizonclient.Client, error) {
	if w.config.StellarHorizonUrl != "" {
		return &horizonclient.Client{HorizonURL: w.config.StellarHorizonUrl}, nil
	}

	switch w.config.StellarNetwork {
	case "testnet":
		return horizonclient.DefaultTestNetClient, nil
	case "production":
		return horizonclient.DefaultPublicNetClient, nil
	default:
		return nil, errors.New("network is not supported")
	}
}

// getNetworkPassPhrase gets the Stellar network passphrase based on the wallet's network
func (w *StellarWallet) getNetworkPassPhrase() string {
	switch w.config.StellarNetwork {
	case "testnet":
		return network.TestNetworkPassphrase
	case "production":
		return network.PublicNetworkPassphrase
	default:
		return network.TestNetworkPassphrase
	}
}

func (w *StellarWallet) getAssetCodeAndIssuer() []string {
	switch w.config.StellarNetwork {
	case "testnet":
		return strings.Split(TFTTest, ":")
	case "production":
		return strings.Split(TFTMainnet, ":")
	default:
		return strings.Split(TFTTest, ":")
	}
}

func (w *StellarWallet) StatBridgeAccount() (string, error) {
	acc, err := w.getAccountDetails(w.config.StellarBridgeAccount)
	if err != nil {
		return "", err
	}

	asset := w.getAssetCodeAndIssuer()

	for _, balance := range acc.Balances {
		if balance.Code == asset[0] || balance.Issuer == asset[1] {
			return balance.Balance, nil
		}
	}
	return "", nil
}
