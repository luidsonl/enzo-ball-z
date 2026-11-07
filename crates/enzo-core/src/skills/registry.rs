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

    pub const fn code(self) -> u8 {
        self.index() as u8
    }

    pub const fn from_code(code: u8) -> Option<Self> {
        match code {
            0 => Some(SkillId::Jump),
            1 => Some(SkillId::Dash),
            2 => Some(SkillId::Fly),
            3 => Some(SkillId::Melee),
            4 => Some(SkillId::Mogging),
            5 => Some(SkillId::NutellaShot),
            6 => Some(SkillId::Shout),
            _ => None,
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
