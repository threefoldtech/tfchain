use crate::types::PublicIP;
pub trait Tfgrid<AccountId, Name> {
    fn is_farm_owner(farm_id: u32, who: AccountId) -> bool;
}

pub trait ChangeNode<Loc, If, Serial> {
    fn node_changed(
        node: Option<&super::types::Node<Loc, If, Serial>>,
        new_node: &super::types::Node<Loc, If, Serial>,
    );
    fn node_deleted(node: &super::types::Node<Loc, If, Serial>);
    fn node_power_state_changed(node: &super::types::Node<Loc, If, Serial>);
}

pub trait PublicIpModifier {
    fn ip_removed(ip: &PublicIP);
}
