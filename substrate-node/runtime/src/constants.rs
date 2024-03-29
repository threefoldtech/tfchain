// Copyright 2020 Parity Technologies (UK) Ltd.
// This file is part of Polkadot.

// Polkadot is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Polkadot is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Polkadot.  If not, see <http://www.gnu.org/licenses/>.

/// Money matters.
pub mod currency {
    use crate::Balance;

    pub const DOTS: Balance = 1_000_000_0;
    pub const DOLLARS: Balance = DOTS;
    pub const CENTS: Balance = DOLLARS / 100;
    pub const MILLICENTS: Balance = CENTS / 1_000;

    pub const fn deposit(items: u32, bytes: u32) -> Balance {
        items as Balance * 1 * DOLLARS + (bytes as Balance) * 5 * MILLICENTS
    }
}

/// Fee-related.
pub mod fee {
    use crate::Balance;
    use frame_support::weights::constants::ExtrinsicBaseWeight;
    use frame_support::weights::{
        WeightToFeeCoefficient, WeightToFeeCoefficients, WeightToFeePolynomial,
    };
    use smallvec::smallvec;
    pub use sp_runtime::{PerThing, Perbill};

    /// The block saturation level. Fees will be updates based on this value.
    pub const TARGET_BLOCK_FULLNESS: Perbill = Perbill::from_percent(25);

    /// Handles converting a weight scalar to a fee value, based on the scale and granularity of the
    /// node's balance type.
    ///
    /// This should typically create a mapping between the following ranges:
    ///   - [0, system::MaximumBlockWeight]
    ///   - [Balance::min, Balance::max]
    ///
    /// Yet, it can be used for any other sort of change to weight-fee. Some examples being:
    ///   - Setting it to `0` will essentially disable the weight fee.
    ///   - Setting it to `1` will cause the literal `#[weight = x]` values to be charged.
    pub struct WeightToFeeStruct;
    impl WeightToFeePolynomial for WeightToFeeStruct {
        type Balance = Balance;
        fn polynomial() -> WeightToFeeCoefficients<Self::Balance> {
            // in Westend, extrinsic base weight (smallest non-zero weight) is mapped to 1/10 CENT:
            let p = super::currency::CENTS;
            let q = 10 * Balance::from(ExtrinsicBaseWeight::get().ref_time());
            smallvec![WeightToFeeCoefficient {
                degree: 1,
                negative: false,
                coeff_frac: PerThing::from_rational(p % q, q),
                coeff_integer: p / q,
            }]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::currency::{CENTS, MILLICENTS};
    use super::fee::WeightToFeeStruct;
    use frame_support::weights::constants::ExtrinsicBaseWeight;
    use frame_support::weights::WeightToFee;

    #[test]
    // This function tests that the fee for `ExtrinsicBaseWeight` of weight is correct
    fn extrinsic_base_fee_is_correct() {
        // `ExtrinsicBaseWeight` should cost 1/10 of a CENT
        log::info!("Base: {}", ExtrinsicBaseWeight::get());
        let x = WeightToFeeStruct::weight_to_fee(&ExtrinsicBaseWeight::get());
        let y = CENTS / 10;
        assert!(x.max(y) - x.min(y) < MILLICENTS);
    }
}
