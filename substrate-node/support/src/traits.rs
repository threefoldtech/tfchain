pub trait Tfgrid<AccountId, Name, Ip, Gateway> {
    fn get_farm(farm_id: u32) -> Option<super::types::Farm<Name, Ip, Gateway>>;
    fn is_farm_owner(farm_id: u32, who: AccountId) -> bool;
    fn is_twin_owner(twin_id: u32, who: AccountId) -> bool;
}

pub trait ChangeNode {
    fn node_changed(node: Option<&super::types::Node>, new_node: &super::types::Node);
    fn node_deleted(node: &super::types::Node);
}