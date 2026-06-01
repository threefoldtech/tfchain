package stellar

import (
	"context"
	"encoding/base64"
	"encoding/hex"
	"fmt"
	"math/big"
	"net/http"
	"strconv"
	"strings"
	"time"

	"github.com/hashicorp/go-retryablehttp"
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
	"github.com/threefoldtech/tfchain/bridge/tfchain_bridge/pkg"
	substrate "github.com/threefoldtech/tfchain/clients/tfchain-client-go"
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

type TraceIdKey struct{}

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
	ctx_with_trace_id := context.WithValue(ctx, TraceIdKey{}, txHash)

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
	ctx_with_trace_id := context.WithValue(ctx, TraceIdKey{}, txHash)
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

	return fmt.Errorf("address has no trustline")
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

// terminalSubmitOperationCodes are Horizon payment operation result codes that
// represent a target-side failure the bridge can never resolve on its own:
// delivery requires a deliberate action by the destination owner (add a
// trustline, create the account, get issuer authorization) that the bridge has
// no visibility into. A refund hitting one of these is forfeited rather than
// retried forever.
//
// Deliberately excluded:
//   - op_line_full: the destination CAN receive the asset but is momentarily at
//     its trustline limit; it may succeed once the balance is spent down, so it
//     is postponed and retried rather than forfeited.
//   - op_underfunded / op_src_no_trust / op_src_not_authorized: bridge-side
//     (source) problems that must alert/halt, never forfeit a user's refund.
//   - op_malformed / op_no_issuer: bridge bug or global asset misconfiguration
//     affecting all transactions, not a per-target condition.
var terminalSubmitOperationCodes = map[string]struct{}{
	"op_no_trust":       {}, // destination has no trustline for the asset
	"op_no_destination": {}, // destination account does not exist
	"op_not_authorized": {}, // destination is not authorized to hold the asset
}

// isTerminalSubmitError reports whether a Stellar submission failure is a
// target-side error that the bridge cannot resolve by retrying, and so should
// be forfeited rather than postponed.
func isTerminalSubmitError(codes *hProtocol.TransactionResultCodes) bool {
	if codes == nil {
		return false
	}
	for _, op := range codes.OperationCodes {
		if _, ok := terminalSubmitOperationCodes[op]; ok {
			return true
		}
	}
	return false
}

// isInsufficientFundsError reports whether a Stellar submission failed because
// the bridge (source) account is out of funds. This is an operational
// condition that requires the bridge wallet to be refilled; it is not the
// target's fault and the transaction must not be forfeited.
func isInsufficientFundsError(codes *hProtocol.TransactionResultCodes) bool {
	if codes == nil {
		return false
	}
	for _, op := range codes.OperationCodes {
		if op == "op_underfunded" {
			return true
		}
	}
	return false
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
			log.Err(err).Msgf("error while submitting transaction %+v", hError.Problem.Extras)
			if codes, rcErr := hError.ResultCodes(); rcErr == nil {
				// A target-side failure (e.g. the destination removed its trustline)
				// can never succeed on retry. Surface it as a typed error so the
				// caller can quarantine the transaction instead of retrying forever.
				// The account sequence is irrelevant here, so don't reset it.
				if isTerminalSubmitError(codes) {
					return errors.Wrapf(pkg.ErrStellarTransactionUndeliverable, "operation result codes %v", codes.OperationCodes)
				}
				// The bridge account is out of funds. This is recoverable (the
				// transaction will be retried) but needs operator attention to refill
				// the bridge wallet, so raise an alert rather than only postponing.
				if isInsufficientFundsError(codes) {
					log.Warn().
						Str("trace_id", fmt.Sprint(ctx.Value(TraceIdKey{}))).
						Str("event_action", "bridge_account_underfunded").
						Str("event_kind", "alert").
						Str("category", "vault").
						Msg("the bridge account has insufficient funds to submit the transaction; it will be retried but the bridge wallet must be refilled")
				}
			}
		}
		errSequence := w.resetAccountSequence()
		if errSequence != nil {
			return errSequence
		}
		return errors.Wrap(err, "an error occurred while submitting the transaction")
	}
	log.Info().
		Str("trace_id", fmt.Sprint(ctx.Value(TraceIdKey{}))).
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
		// Skip the effect unless BOTH the asset code and issuer match the
		// bridge's TFT asset. Using && here meant a credit was only skipped
		// when both differed, so a credit with the right code but a wrong
		// issuer (or vice-versa) slipped through and could be minted.
		if creditedEffect.Code != asset[0] || creditedEffect.Issuer != asset[1] {
			continue
		}

		ops, err := w.getOperationEffect(tx.Hash)
		if err != nil {
			continue
		}

		senders := make(map[string]*big.Int)
		for _, op := range ops.Embedded.Records {
			// Skip non-payment operations individually. Previously this
			// returned from the whole function on the first non-payment op,
			// silently dropping any legitimate payment ops in the same
			// transaction (lost deposits / missing mints).
			if op.GetType() != "payment" {
				continue
			}

			PaymentOperation := op.(operations.Payment)
			if PaymentOperation.To != w.config.StellarBridgeAccount || PaymentOperation.From == w.config.StellarBridgeAccount {
				continue
			}

			// Validate the payment asset at the operation level too. The
			// account_credited effect check above gates entry into this loop,
			// but the per-payment amount must itself be TFT — otherwise a
			// non-TFT payment to the bridge in the same transaction would be
			// summed and minted as TFT.
			if PaymentOperation.Code != asset[0] || PaymentOperation.Issuer != asset[1] {
				logger.Warn().
					Str("event_action", "non_tft_payment_rejected").
					Str("event_kind", "alert").
					Str("from", PaymentOperation.From).
					Str("asset_code", PaymentOperation.Code).
					Str("asset_issuer", PaymentOperation.Issuer).
					Str("amount", PaymentOperation.Amount).
					Str("tx_hash", PaymentOperation.TransactionHash).
					Msg("non-TFT payment to bridge detected — skipping")
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
	var client *horizonclient.Client

	if w.config.StellarHorizonUrl != "" {
		client = &horizonclient.Client{HorizonURL: w.config.StellarHorizonUrl}
	}

	switch w.config.StellarNetwork {
	case "testnet":
		client = horizonclient.DefaultTestNetClient
	case "production":
		client = horizonclient.DefaultPublicNetClient
	default:
		return nil, errors.New("network is not supported")
	}

	// custom HTTP client with retry logic
	retryClient := retryablehttp.NewClient()
	retryClient.RetryMax = 3
	retryClient.RetryWaitMin = 2 * time.Second
	retryClient.RetryWaitMax = 5 * time.Second

	retryClient.CheckRetry = func(ctx context.Context, resp *http.Response, err error) (bool, error) {
		if ctx.Err() != nil {
			return false, ctx.Err()
		}

		if err != nil {
			return true, nil
		}

		if resp.StatusCode == 429 || (resp.StatusCode >= 500 && resp.StatusCode <= 599) {
			return true, nil
		}

		return false, nil
	}

	client.HTTP = retryClient.StandardClient()

	return client, nil
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
		// Match the TFT balance on BOTH code and issuer. Using || could
		// return an unrelated asset's balance that happened to share either
		// the code or the issuer.
		if balance.Code == asset[0] && balance.Issuer == asset[1] {
			return balance.Balance, nil
		}
	}
	return "", errors.New("source account does not have trustline")
}
