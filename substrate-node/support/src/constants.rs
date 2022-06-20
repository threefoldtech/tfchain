pub type BlockNumber = u32;
/// Time and blocks.
pub mod time {
    pub const MILLISECS_PER_BLOCK: u64 = 6000;
    pub const SLOT_DURATION: u64 = MILLISECS_PER_BLOCK;
    pub const EPOCH_DURATION_IN_BLOCKS: super::BlockNumber = 1 * HOURS;

    // These time units are defined in number of blocks.
    pub const MINUTES: super::BlockNumber = 60_000 / (MILLISECS_PER_BLOCK as super::BlockNumber);
    pub const HOURS: super::BlockNumber = MINUTES * 60;
    pub const DAYS: super::BlockNumber = HOURS * 24;
}