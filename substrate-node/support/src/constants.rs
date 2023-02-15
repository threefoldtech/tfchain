pub type BlockNumber = u32;
/// Time and blocks.
pub mod time {
    /// This determines the average expected block time that we are targeting.
    /// Blocks will be produced at a minimum duration defined by `SLOT_DURATION`.
    /// `SLOT_DURATION` is picked up by `pallet_timestamp` which is in turn picked
    /// up by `pallet_aura` to implement `fn slot_duration()`.
    ///
    /// Change this to adjust the block time.
    pub const MILLISECS_PER_BLOCK: u64 = 6000;
    pub const SLOT_DURATION: u64 = MILLISECS_PER_BLOCK;
    pub const SECS_PER_HOUR: u64 = 3600;
    pub const EPOCH_DURATION_IN_BLOCKS: super::BlockNumber = 1 * HOURS;

    // These time units are defined in number of blocks.
    pub const MINUTES: super::BlockNumber = 60_000 / (MILLISECS_PER_BLOCK as super::BlockNumber);
    pub const HOURS: super::BlockNumber = MINUTES * 60;
    pub const DAYS: super::BlockNumber = HOURS * 24;
}
