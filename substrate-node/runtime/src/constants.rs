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

/// Time and blocks.
pub mod time {
    use crate::BlockNumber;
    pub const MILLISECS_PER_BLOCK: u64 = 6000;
    pub const SLOT_DURATION: u64 = MILLISECS_PER_BLOCK;
    pub const EPOCH_DURATION_IN_BLOCKS: BlockNumber = 1 * HOURS;

    // These time units are defined in number of blocks.
    pub const MINUTES: BlockNumber = 60_000 / (MILLISECS_PER_BLOCK as BlockNumber);
    pub const HOURS: BlockNumber = MINUTES * 60;
    pub const DAYS: BlockNumber = HOURS * 24;

    // 1 in 4 blocks (on average, not counting collisions) will be primary babe blocks.
    pub const PRIMARY_PROBABILITY: (u64, u64) = (1, 4);
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
            let q = 10 * Balance::from(ExtrinsicBaseWeight::get());
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
    use crate::MaximumBlockWeight;
    use frame_support::weights::constants::ExtrinsicBaseWeight;
    use frame_support::weights::WeightToFee;
    use env_logger;

    #[test]
    // This function tests that the fee for `MaximumBlockWeight` of weight is correct
    fn full_block_fee_is_correct() {
        env_logger::try_init();
        // A full block should cost 23.3112 DOLLARS
        log::info!("MaxBlockWeight: {:?}", MaximumBlockWeight::get());
        log::info!("BaseWeight: {:?}", ExtrinsicBaseWeight::get());
        // we multiply by 100 to avoid loss of precision after division and devide by 100 at the end
        let precision = 100;
        let max_block_weight: u128 = (MaximumBlockWeight::get() as u128) * precision;
        let ext_base_weight: u128 = ExtrinsicBaseWeight::get() as u128;
        let x = WeightToFeeStruct::weight_to_fee(&MaximumBlockWeight::get());
        let cost_extrinsic:u128 = WeightToFeeStruct::weight_to_fee(&ExtrinsicBaseWeight::get());
        let y:u128 = (cost_extrinsic * (max_block_weight/ext_base_weight)) / precision;

        assert!(x.max(y) - x.min(y) < cost_extrinsic / 2);
    }

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
