use frame_support::pallet_prelude::DispatchResultWithPostInfo;

use crate::resources::Resources;
use crate::types::PublicIP;
pub trait Tfgrid<AccountId, Name> {
    fn get_farm(farm_id: u32) -> Option<super::types::Farm<Name>>;
    fn is_farm_owner(farm_id: u32, who: AccountId) -> bool;
    fn is_twin_owner(twin_id: u32, who: AccountId) -> bool;
    fn change_used_resources_on_node(
        node_id: u32,
        to_free: Resources,
        to_consume: Resources,
    ) -> DispatchResultWithPostInfo;
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
