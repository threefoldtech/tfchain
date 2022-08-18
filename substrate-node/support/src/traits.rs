pub trait Tfgrid<AccountId, Name, PublicIP> {
    fn get_farm(farm_id: u32) -> Option<super::types::Farm<Name, PublicIP>>;
    fn is_farm_owner(farm_id: u32, who: AccountId) -> bool;
    fn is_twin_owner(twin_id: u32, who: AccountId) -> bool;
}

pub trait ChangeNode<PubConfig, If> {
    fn node_changed(node: Option<&super::types::Node<PubConfig, If>>, new_node: &super::types::Node<PubConfig, If>);
    fn node_deleted(node: &super::types::Node<PubConfig, If>);
}