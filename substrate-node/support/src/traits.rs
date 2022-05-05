pub trait Tfgrid<AccountId> {
    fn get_farm(farm_id: u32) -> super::types::Farm;
    fn is_farm_owner(farm_id: u32, who: AccountId) -> bool;
    fn is_twin_owner(twin_id: u32, who: AccountId) -> bool;
}

pub trait ChangeNode {
    fn node_changed(node: &super::types::Node, new_node: &super::types::Node);
}