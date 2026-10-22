use landlord::mana::{ManaCost, Source};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CardSpec {
    pub id: u32,
    pub cost: ManaCost,
    pub printed_mana_value: u16,
    pub legendary: bool,
    pub keep_priority: i16,
    pub kind: Kind,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Kind {
    Land(LandSpec),
    Spell(SpellSpec),
    Ignored,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LandSpec {
    pub source: Source,
    pub enters: Enters,
    pub restriction: Restriction,
    /// WUBRG basic land type bits, separate from mana-producing colors.
    pub basic_types: u8,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Enters {
    Untapped,
    Tapped,
    OtherLandsAtLeast(u8),
    OtherLandsAtMost(u8),
    HasBasicType(u8),
    PayLife(u8),
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Restriction {
    Any,
    ArtifactSpell,
    EquipmentSpell,
    CreatureSpell,
}
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SpellSpec {
    pub permanent: bool,
    pub artifact: bool,
    pub equipment: bool,
    pub creature: bool,
    pub haste: bool,
    pub equip_cost: Option<ManaCost>,
    pub tutor: Option<TutorSpec>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TutorSpec {
    pub timing: TutorTiming,
    pub destination: Destination,
    pub max_mana_value: Option<u16>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TutorTiming {
    SpellResolves,
    EnterBattlefield,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Destination {
    Hand,
    Battlefield,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TurnSnapshot {
    pub turn: u8,
    pub life: u8,
    pub lands: usize,
    pub equipment: usize,
    pub attached: usize,
    /// Equipment attached to creatures eligible to attack this turn. No combat
    /// outcome, opponent interaction, power/toughness, or damage is simulated.
    pub attack_ready: usize,
    pub legendary_bricks: usize,
    pub spells_cast: usize,
    pub tutors_cast: usize,
    pub searches_resolved: usize,
    pub decked: bool,
}
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Goal {
    pub equipment: usize,
    pub attached: usize,
    pub attack_ready: usize,
}
impl Goal {
    pub fn met(&self, snapshot: &TurnSnapshot) -> bool {
        !snapshot.decked
            && snapshot.equipment >= self.equipment
            && snapshot.attached >= self.attached
            && snapshot.attack_ready >= self.attack_ready
    }
}
