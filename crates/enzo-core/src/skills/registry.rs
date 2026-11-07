use super::{dash, fly, jump, melee, mogging, nutella_shot, shout, SkillDef, SkillId};

impl SkillId {
    pub const ALL: [SkillId; 7] = [
        SkillId::Jump,
        SkillId::Dash,
        SkillId::Fly,
        SkillId::Melee,
        SkillId::Mogging,
        SkillId::NutellaShot,
        SkillId::Shout,
    ];

    pub const COUNT: usize = Self::ALL.len();

    pub const fn index(self) -> usize {
        match self {
            SkillId::Jump => 0,
            SkillId::Dash => 1,
            SkillId::Fly => 2,
            SkillId::Melee => 3,
            SkillId::Mogging => 4,
            SkillId::NutellaShot => 5,
            SkillId::Shout => 6,
        }
    }

    pub const fn def(self) -> &'static SkillDef {
        match self {
            SkillId::Jump => &jump::DEF,
            SkillId::Dash => &dash::DEF,
            SkillId::Fly => &fly::DEF,
            SkillId::Melee => &melee::DEF,
            SkillId::Mogging => &mogging::DEF,
            SkillId::NutellaShot => &nutella_shot::DEF,
            SkillId::Shout => &shout::DEF,
        }
    }
}
