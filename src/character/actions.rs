use super::stats::Stats;

pub trait Action {
    fn attack_roll_modifer(stat: &Stats, proficiency_bonus: isize) -> usize;
    fn damage_roll_modifer(stat: &Stats, proficiency_bonus: isize) -> usize;
}
