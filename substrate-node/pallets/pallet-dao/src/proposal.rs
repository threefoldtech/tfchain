use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_std::vec::Vec;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug, Encode, Decode, Default, TypeInfo)]
pub struct DaoProposal<ProposalIndex> {
    pub index: ProposalIndex,
    pub description: Vec<u8>,
    pub link: Vec<u8>,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug, Encode, Decode, Default, TypeInfo)]
pub struct DaoVotes<ProposalIndex, BlockNumber, AccountId> {
    pub index: ProposalIndex,
    pub threshold: u32,
    pub ayes: Vec<VoteWeight>,
    pub nays: Vec<VoteWeight>,
    pub end: BlockNumber,
    pub vetos: Vec<AccountId>,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug, Encode, Decode, Default, TypeInfo)]
pub struct VoteWeight {
    pub farm_id: u32,
    pub weight: u64,
}
