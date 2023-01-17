use crate::{resources::Resources, types::PublicIP};
use frame_support::pallet_prelude::DispatchResultWithPostInfo;

pub trait Tfgrid<AccountId, Name> {
    fn get_farm(farm_id: u32) -> Option<super::types::Farm<Name>>;
    fn is_farm_owner(farm_id: u32, who: AccountId) -> bool;
    fn is_twin_owner(twin_id: u32, who: AccountId) -> bool;
}

pub trait ChangeNode<Loc, If, Serial> {
    fn node_changed(
        node: Option<&super::types::Node<Loc, If, Serial>>,
        new_node: &super::types::Node<Loc, If, Serial>,
    );
    fn node_deleted(node: &super::types::Node<Loc, If, Serial>);
}

pub trait PublicIpModifier {
    fn ip_removed(ip: &PublicIP);
}

pub trait MintingHook<AccountId> {
    fn report_uptime(source: &AccountId, uptime: u64) -> DispatchResultWithPostInfo;
    fn report_nru(source: &AccountId, nru: u64, window: u64) -> DispatchResultWithPostInfo;
    fn report_used_resources(
        node_id: u32,
        resources: Resources,
        window: u64,
    ) -> DispatchResultWithPostInfo;
}
