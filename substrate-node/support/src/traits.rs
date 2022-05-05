pub trait Tfgrid<AccountId> {
    fn get_farm(farm_id: u32) -> super::farms::Farm;
}