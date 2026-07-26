// TFT has 7 decimals on tfchain, so amounts on the wire are integers of 1e-7 TFT.
const TFT_DECIMALS = 7
const UNITS_PER_TFT = 10 ** TFT_DECIMALS

// `ACTIVATION_AMOUNT` is expressed in whole TFT, which is what readme.md has always
// documented ("ACTIVATION_AMOUNT=1", "currently 1 TFT") and what the Helm chart's
// `activation_amount: 1` means. Nothing ever read the variable though — the amount was
// hardcoded to 1000000 units, i.e. 0.1 TFT, so the documentation overstated it tenfold.
// The variable is honoured now; the default preserves the amount actually funded before.
const DEFAULT_ACTIVATION_TFT = 0.1

// Existing accounts that have dipped below this floor are topped back up by this much.
// Not configurable: it is a balance threshold rather than a funding policy.
const TOPUP_AMOUNT = 15000 // 0.0015 TFT

// Amounts are rounded to whole base units, so anything finer than 7 decimals is a
// typo rather than an intent. Tolerance guards against binary float representation:
// 0.1 * 1e7 is 1000000.0000000002, not 1000000.
const ROUNDING_TOLERANCE = 1e-6

/**
 * Parse an ACTIVATION_AMOUNT value, given in whole TFT, into base units.
 *
 * @param {string|undefined} raw - the raw env value; absent or empty means "use the default"
 * @returns {number} the amount in base units, ready to hand to balances.transfer
 * @throws {Error} if the value is not a positive number, or is finer than 7 decimals
 */
function parseActivationAmount (raw) {
  const tft = raw === undefined || raw === '' ? DEFAULT_ACTIVATION_TFT : Number(raw)

  if (!Number.isFinite(tft) || tft <= 0) {
    throw new Error(
      `ACTIVATION_AMOUNT must be a positive amount of TFT, got "${raw}"`
    )
  }

  const exact = tft * UNITS_PER_TFT
  const units = Math.round(exact)

  if (units < 1 || Math.abs(exact - units) > ROUNDING_TOLERANCE) {
    throw new Error(
      `ACTIVATION_AMOUNT cannot be expressed in TFT base units: "${raw}" TFT needs ` +
      `more than ${TFT_DECIMALS} decimals`
    )
  }

  return units
}

module.exports = {
  // A function rather than a value so a malformed setting surfaces from init(), and
  // is therefore logged and reported like every other startup failure, instead of
  // throwing while this module is still being required. Returns base units.
  activationAmount: () => parseActivationAmount(process.env.ACTIVATION_AMOUNT),
  topupAmount: TOPUP_AMOUNT,
  parseActivationAmount,
  DEFAULT_ACTIVATION_TFT,
  UNITS_PER_TFT
}
