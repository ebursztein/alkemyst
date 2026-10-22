use landlord::mana::{ManaCost, Source};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CardSpec {
    pub id: u32,
    pub cost: ManaCost,
    pub printed_mana_value: u16,
    pub legendary: bool,
    pub keep_priority: i16,
    pub tags: Tags,
    pub adventure: Option<Adventure>,
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
    pub conditional: Vec<ManaAbility>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Enters {
    Untapped,
    Tapped,
    OtherLandsAtLeast(u8),
    OtherLandsAtMost(u8),
    HasBasicType(u8),
    PayLife(u8),
    ControlsEquipment,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Restriction {
    Any,
    ArtifactSpell,
    EquipmentSpell,
    CreatureSpell,
    EquipmentOrEquip,
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
    pub mana_ability: Option<ManaAbility>,
    pub worthy_only: bool,
    pub equip_reduction_per_color: bool,
    pub story_free_equip: bool,
    pub target_equip_reduction: u16,
    pub attach_on_enter: bool,
    pub auto_attach_min_value: Option<u16>,
    pub entry_draw: Option<EntryDraw>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TutorSpec {
    pub scope: SearchScope,
    pub types: SearchTypes,
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
    pub enduring_story: bool,
    pub lands: usize,
    pub creatures: usize,
    pub adventure_exile: usize,
    pub cards_drawn: usize,
    pub cards_discarded: usize,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManaAbility {
    pub source: Source,
    pub restriction: Restriction,
    pub condition: Condition,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Condition {
    Always,
    HasBasicType(u8),
    ControlsEquipment,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Tags {
    pub colors: u8,
    pub dwarf: bool,
    pub saga: bool,
    pub villain: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchScope {
    Library,
    Top(u8),
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchTypes {
    Equipment,
    EquipmentOrDwarf,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntryDraw {
    pub allow_self: bool,
    pub once_per_turn: bool,
    pub discard: DiscardPolicy,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiscardPolicy {
    None,
    Bricks,
    LowestPriority,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Adventure {
    pub cost: ManaCost,
    pub tutor: TutorSpec,
}
