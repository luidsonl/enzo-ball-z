//! Internacionalização da interface.
//!
//! O jogo não usa arquivos de tradução em runtime: todas as strings ficam no
//! binário para o cliente continuar sem IO. `Text` é a chave estável; o match
//! exaustivo garante que uma tradução nova não esqueça um idioma.
//!
//! Regra do projeto: o core nunca formata texto de UI. Ele expõe dados
//! (`Attribute`, `SkillId`, `ItemKind`, `EquipSlot`) and the client chooses the label.

use enzo_core::entities::{EquipSlot, ItemKind};
use enzo_core::input::Attribute;
use enzo_core::skills::SkillId;

pub const LANGUAGES: [Language; 2] = [Language::PtBr, Language::English];

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Language {
    #[default]
    PtBr,
    English,
}

impl Language {
    /// Código curto usado no `localStorage`.
    pub fn code(self) -> &'static str {
        match self {
            Self::PtBr => "pt-BR",
            Self::English => "en",
        }
    }

    pub fn from_code(code: &str) -> Option<Self> {
        match code {
            "pt-BR" | "pt" => Some(Self::PtBr),
            "en" | "en-US" | "en-GB" => Some(Self::English),
            _ => None,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::PtBr => "Português (BR)",
            Self::English => "English",
        }
    }
}

/// Chaves de texto da UI. Adicione novas variantes aqui e o compilador cobra a
/// tradução nos dois idiomas.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Text {
    // Menu inicial
    MenuSubtitle,
    Play,
    Resume,
    ManageSaves,
    Options,
    Back,
    ChooseSlot,
    EmptySlot,
    Load,
    NewGame,
    Delete,
    DeleteConfirm,
    Confirm,
    Cancel,
    Language,
    InterfaceSize,
    MobileControls,
    MobileControlsOn,
    MobileControlsOff,
    MobileControlsAuto,
    SlotLabel,
    SlotDeleted,
    LevelLoadFailed,
    LevelLabel,
    PointsLabel,
    MenuHint,
    MainTitle,
    SavesTitle,
    OptionsTitle,

    // HUD fixo
    LevelShort,
    Health,
    Aura,
    Xp,
    Points,
    Status,
    Inventory,
    Equipment,
    Skills,
    MenuButton,
    DragHint,
    Close,
    MoggingReady,
    MoggingCooldown,
    SavedPrefix,
    SaveFailed,

    // Janela de status / level-up
    StatusTitle,
    AttributeSpeed,
    AttributeStrength,
    AttributeHealth,
    AttributeAura,
    EffectSpeed,
    EffectStrength,
    EffectHealth,
    EffectAura,
    CurrentValue,
    LevelUpTitle,
    PointsToSpend,
    SpendHint,
    BackToGame,

    // Janela de inventário
    InventoryTitle,
    InventoryEmpty,

    // Janela de equipamentos
    EquipmentTitle,
    WeaponSlot,
    ArmorSlot,
    AccessorySlot,
    NoItem,

    // Janela de skills
    SkillsTitle,
    Movements,
    Attacks,
    NoMoves,
    MoveWalk,
    MoveJump,
    MoveDash,
    MoveFly,
    MoveSixSeven,
    MoveMelee,
    MoveMogging,
    SkillNutellaShot,
    SkillShout,
    SkillEmpty,
    SkillNoAura,
    SkillCooldown,
    SkillLocked,

    // Itens
    ItemSword,
    ItemBow,
    ItemArmor,
    ItemAmulet,
    ItemPotion,
    ItemNutellaGun,

    // Overlays de fase
    PhaseBanner,
    ControlsLine1,
    ControlsLine2,
    DeathTitle,
    RespawnIn,
    ProgressionKept,
    RestartNow,
    VictoryTitle,
    ProgressSaved,
    PlayAgain,
    ReturnToMenu,

    // Efeitos do mundo
    EffectMogging,
    EffectBlocked,
    EffectLevel,
    EffectNoAura,
    EffectDeath,
    EffectVictory,

    // Motivos exibidos no toast de save
    SaveReasonDeath,
    SaveReasonAttribute,
    SaveReasonProgress,
}

/// Traduções ativas. É `Copy` e barato de passar para o HUD.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct I18n {
    language: Language,
}

impl I18n {
    pub fn new(language: Language) -> Self {
        Self { language }
    }

    pub fn language(self) -> Language {
        self.language
    }

    pub fn set_language(&mut self, language: Language) {
        self.language = language;
    }

    pub fn t(self, text: Text) -> &'static str {
        match (self.language, text) {
            // Menu inicial
            (Language::PtBr, Text::MenuSubtitle) => "Corra, lute e farme aura",
            (Language::English, Text::MenuSubtitle) => "Run, fight and farm aura",
            (Language::PtBr, Text::Play) => "Jogar",
            (Language::English, Text::Play) => "Play",
            (Language::PtBr, Text::Resume) => "Continuar",
            (Language::English, Text::Resume) => "Resume",
            (Language::PtBr, Text::ManageSaves) => "Gerenciar saves",
            (Language::English, Text::ManageSaves) => "Manage saves",
            (Language::PtBr, Text::Options) => "Opções",
            (Language::English, Text::Options) => "Options",
            (Language::PtBr, Text::Back) => "Voltar",
            (Language::English, Text::Back) => "Back",
            (Language::PtBr, Text::ChooseSlot) => "Escolha um slot para jogar",
            (Language::English, Text::ChooseSlot) => "Choose a slot to play",
            (Language::PtBr, Text::EmptySlot) => "Slot vazio",
            (Language::English, Text::EmptySlot) => "Empty slot",
            (Language::PtBr, Text::Load) => "Carregar",
            (Language::English, Text::Load) => "Load",
            (Language::PtBr, Text::NewGame) => "Novo jogo",
            (Language::English, Text::NewGame) => "New game",
            (Language::PtBr, Text::Delete) => "Apagar",
            (Language::English, Text::Delete) => "Delete",
            (Language::PtBr, Text::DeleteConfirm) => "Apagar este save?",
            (Language::English, Text::DeleteConfirm) => "Delete this save?",
            (Language::PtBr, Text::Confirm) => "Confirmar",
            (Language::English, Text::Confirm) => "Confirm",
            (Language::PtBr, Text::Cancel) => "Cancelar",
            (Language::English, Text::Cancel) => "Cancel",
            (Language::PtBr, Text::Language) => "Idioma",
            (Language::English, Text::Language) => "Language",
            (Language::PtBr, Text::InterfaceSize) => "Tamanho da interface",
            (Language::English, Text::InterfaceSize) => "Interface size",
            (Language::PtBr, Text::MobileControls) => "Controles mobile",
            (Language::English, Text::MobileControls) => "Mobile controls",
            (Language::PtBr, Text::MobileControlsOn) => "Ligado",
            (Language::English, Text::MobileControlsOn) => "On",
            (Language::PtBr, Text::MobileControlsOff) => "Desligado",
            (Language::English, Text::MobileControlsOff) => "Off",
            (Language::PtBr, Text::MobileControlsAuto) => "Automático",
            (Language::English, Text::MobileControlsAuto) => "Auto",
            (Language::PtBr, Text::SlotLabel) => "Slot",
            (Language::English, Text::SlotLabel) => "Slot",
            (Language::PtBr, Text::SlotDeleted) => "Slot apagado",
            (Language::English, Text::SlotDeleted) => "Slot deleted",
            (Language::PtBr, Text::LevelLoadFailed) => "falha ao carregar a fase",
            (Language::English, Text::LevelLoadFailed) => "failed to load the stage",
            (Language::PtBr, Text::LevelLabel) => "Nível",
            (Language::English, Text::LevelLabel) => "Level",
            (Language::PtBr, Text::PointsLabel) => "pontos",
            (Language::English, Text::PointsLabel) => "points",
            (Language::PtBr, Text::MenuHint) => "Use o mouse para navegar",
            (Language::English, Text::MenuHint) => "Use the mouse to navigate",
            (Language::PtBr, Text::MainTitle) => "ENZO BALL Z",
            (Language::English, Text::MainTitle) => "ENZO BALL Z",
            (Language::PtBr, Text::SavesTitle) => "SAVES",
            (Language::English, Text::SavesTitle) => "SAVES",
            (Language::PtBr, Text::OptionsTitle) => "OPÇÕES",
            (Language::English, Text::OptionsTitle) => "OPTIONS",

            // HUD fixo
            (Language::PtBr, Text::LevelShort) => "Nv.",
            (Language::English, Text::LevelShort) => "Lv.",
            (Language::PtBr, Text::Health) => "VIDA",
            (Language::English, Text::Health) => "HEALTH",
            (Language::PtBr, Text::Aura) => "AURA",
            (Language::English, Text::Aura) => "AURA",
            (Language::PtBr, Text::Xp) => "XP",
            (Language::English, Text::Xp) => "XP",
            (Language::PtBr, Text::Points) => "PONTOS",
            (Language::English, Text::Points) => "POINTS",
            (Language::PtBr, Text::Status) => "Status",
            (Language::English, Text::Status) => "Status",
            (Language::PtBr, Text::Inventory) => "Inventário",
            (Language::English, Text::Inventory) => "Inventory",
            (Language::PtBr, Text::Equipment) => "Equipamentos",
            (Language::English, Text::Equipment) => "Equipment",
            (Language::PtBr, Text::Skills) => "Habilidades",
            (Language::English, Text::Skills) => "Skills",
            (Language::PtBr, Text::MenuButton) => "MENU",
            (Language::English, Text::MenuButton) => "MENU",
            (Language::PtBr, Text::DragHint) => "Arraste pelo título",
            (Language::English, Text::DragHint) => "Drag by the title",
            (Language::PtBr, Text::Close) => "Fechar",
            (Language::English, Text::Close) => "Close",
            (Language::PtBr, Text::MoggingReady) => "MOGGING PRONTO (25 aura)",
            (Language::English, Text::MoggingReady) => "MOGGING READY (25 aura)",
            (Language::PtBr, Text::MoggingCooldown) => "MOGGING",
            (Language::English, Text::MoggingCooldown) => "MOGGING",
            (Language::PtBr, Text::SavedPrefix) => "SALVO",
            (Language::English, Text::SavedPrefix) => "SAVED",
            (Language::PtBr, Text::SaveFailed) => "falha ao salvar",
            (Language::English, Text::SaveFailed) => "save failed",

            // Status / level-up
            (Language::PtBr, Text::StatusTitle) => "STATUS DO PERSONAGEM",
            (Language::English, Text::StatusTitle) => "CHARACTER STATUS",
            (Language::PtBr, Text::AttributeSpeed) => "Velocidade",
            (Language::English, Text::AttributeSpeed) => "Speed",
            (Language::PtBr, Text::AttributeStrength) => "Força",
            (Language::English, Text::AttributeStrength) => "Strength",
            (Language::PtBr, Text::AttributeHealth) => "Vida",
            (Language::English, Text::AttributeHealth) => "Health",
            (Language::PtBr, Text::AttributeAura) => "Aura",
            (Language::English, Text::AttributeAura) => "Aura",
            (Language::PtBr, Text::EffectSpeed) => "+10 px/s de caminhada",
            (Language::English, Text::EffectSpeed) => "+10 px/s walk speed",
            (Language::PtBr, Text::EffectStrength) => "+2 dano físico",
            (Language::English, Text::EffectStrength) => "+2 physical damage",
            (Language::PtBr, Text::EffectHealth) => "+10 vida máxima",
            (Language::English, Text::EffectHealth) => "+10 max health",
            (Language::PtBr, Text::EffectAura) => "+10 aura / +2 Mogging",
            (Language::English, Text::EffectAura) => "+10 aura / +2 Mogging",
            (Language::PtBr, Text::CurrentValue) => "atual",
            (Language::English, Text::CurrentValue) => "current",
            (Language::PtBr, Text::LevelUpTitle) => "SUBIU DE NÍVEL!",
            (Language::English, Text::LevelUpTitle) => "LEVEL UP!",
            (Language::PtBr, Text::PointsToSpend) => "ponto(s) para distribuir",
            (Language::English, Text::PointsToSpend) => "point(s) to spend",
            (Language::PtBr, Text::SpendHint) => "Clique em um atributo para investir",
            (Language::English, Text::SpendHint) => "Click an attribute to spend",
            (Language::PtBr, Text::BackToGame) => "Voltar ao jogo",
            (Language::English, Text::BackToGame) => "Back to game",

            // Inventário
            (Language::PtBr, Text::InventoryTitle) => "INVENTÁRIO",
            (Language::English, Text::InventoryTitle) => "INVENTORY",
            (Language::PtBr, Text::InventoryEmpty) => "Nenhum item coletado ainda.",
            (Language::English, Text::InventoryEmpty) => "No items collected yet.",

            // Equipamentos
            (Language::PtBr, Text::EquipmentTitle) => "EQUIPAMENTOS",
            (Language::English, Text::EquipmentTitle) => "EQUIPMENT",
            (Language::PtBr, Text::WeaponSlot) => "Arma",
            (Language::English, Text::WeaponSlot) => "Weapon",
            (Language::PtBr, Text::ArmorSlot) => "Armadura",
            (Language::English, Text::ArmorSlot) => "Armor",
            (Language::PtBr, Text::AccessorySlot) => "Acessório",
            (Language::English, Text::AccessorySlot) => "Accessory",
            (Language::PtBr, Text::NoItem) => "Nenhum item equipado",
            (Language::English, Text::NoItem) => "No item equipped",

            // Skills
            (Language::PtBr, Text::SkillsTitle) => "HABILIDADES",
            (Language::English, Text::SkillsTitle) => "SKILLS",
            (Language::PtBr, Text::Movements) => "Movimentos",
            (Language::English, Text::Movements) => "Movements",
            (Language::PtBr, Text::Attacks) => "Ataques",
            (Language::English, Text::Attacks) => "Attacks",
            (Language::PtBr, Text::NoMoves) => "Nenhum movimento atribuído.",
            (Language::English, Text::NoMoves) => "No moves assigned.",
            (Language::PtBr, Text::MoveWalk) => "Caminhada",
            (Language::English, Text::MoveWalk) => "Walk",
            (Language::PtBr, Text::MoveJump) => "Pulo",
            (Language::English, Text::MoveJump) => "Jump",
            (Language::PtBr, Text::MoveDash) => "Investida",
            (Language::English, Text::MoveDash) => "Dash",
            (Language::PtBr, Text::MoveFly) => "Voo",
            (Language::English, Text::MoveFly) => "Fly",
            (Language::PtBr, Text::MoveSixSeven) => "Farm de aura",
            (Language::English, Text::MoveSixSeven) => "Aura farming",
            (Language::PtBr, Text::MoveMelee) => "Ataque físico",
            (Language::English, Text::MoveMelee) => "Melee attack",
            (Language::PtBr, Text::MoveMogging) => "Mogging",
            (Language::English, Text::MoveMogging) => "Mogging",
            (Language::PtBr, Text::SkillNutellaShot) => "Tiro de Nutela",
            (Language::English, Text::SkillNutellaShot) => "Nutella Shot",
            (Language::PtBr, Text::SkillShout) => "Grito",
            (Language::English, Text::SkillShout) => "Shout",
            (Language::PtBr, Text::SkillEmpty) => "Vazio",
            (Language::English, Text::SkillEmpty) => "Empty",
            (Language::PtBr, Text::SkillNoAura) => "Sem aura",
            (Language::English, Text::SkillNoAura) => "No aura",
            (Language::PtBr, Text::SkillCooldown) => "Recarga",
            (Language::English, Text::SkillCooldown) => "Cooldown",
            (Language::PtBr, Text::SkillLocked) => "Bloqueada",
            (Language::English, Text::SkillLocked) => "Locked",

            // Itens
            (Language::PtBr, Text::ItemSword) => "Espada",
            (Language::English, Text::ItemSword) => "Sword",
            (Language::PtBr, Text::ItemBow) => "Arco",
            (Language::English, Text::ItemBow) => "Bow",
            (Language::PtBr, Text::ItemArmor) => "Armadura",
            (Language::English, Text::ItemArmor) => "Armor",
            (Language::PtBr, Text::ItemAmulet) => "Amuleto",
            (Language::English, Text::ItemAmulet) => "Amulet",
            (Language::PtBr, Text::ItemPotion) => "Poção",
            (Language::English, Text::ItemPotion) => "Potion",
            (Language::PtBr, Text::ItemNutellaGun) => "Arma de Nutela",
            (Language::English, Text::ItemNutellaGun) => "Nutella Gun",

            // Overlays
            (Language::PtBr, Text::PhaseBanner) => "FASE 1  ·  CHEGUE À BANDEIRA",
            (Language::English, Text::PhaseBanner) => "STAGE 1  ·  REACH THE FLAG",
            (Language::PtBr, Text::ControlsLine1) => {
                "A/D ou ←/→ mover   ·   ESPAÇO pular   ·   SHIFT dash   ·   J atacar   ·   K Mogging"
            }
            (Language::English, Text::ControlsLine1) => {
                "A/D or ←/→ move   ·   SPACE jump   ·   SHIFT dash   ·   J attack   ·   K Mogging"
            }
            (Language::PtBr, Text::ControlsLine2) => {
                "Clique nas bolinhas para abrir as janelas   ·   1-4 distribuir pontos   ·   ENTER confirmar   ·   R reiniciar"
            }
            (Language::English, Text::ControlsLine2) => {
                "Click the dots to open windows   ·   1-4 spend points   ·   ENTER confirm   ·   R restart"
            }
            (Language::PtBr, Text::DeathTitle) => "VOCÊ CAIU!",
            (Language::English, Text::DeathTitle) => "YOU FELL!",
            (Language::PtBr, Text::RespawnIn) => "Renascendo em",
            (Language::English, Text::RespawnIn) => "Respawning in",
            (Language::PtBr, Text::ProgressionKept) => "A progressão foi mantida.",
            (Language::English, Text::ProgressionKept) => "Your progression was kept.",
            (Language::PtBr, Text::RestartNow) => "Reiniciar agora",
            (Language::English, Text::RestartNow) => "Restart now",
            (Language::PtBr, Text::VictoryTitle) => "FASE CONCLUÍDA!",
            (Language::English, Text::VictoryTitle) => "STAGE CLEARED!",
            (Language::PtBr, Text::ProgressSaved) => "Progresso salvo no navegador.",
            (Language::English, Text::ProgressSaved) => "Progress saved in the browser.",
            (Language::PtBr, Text::PlayAgain) => "Jogar de novo",
            (Language::English, Text::PlayAgain) => "Play again",
            (Language::PtBr, Text::ReturnToMenu) => "Voltar ao menu",
            (Language::English, Text::ReturnToMenu) => "Return to menu",

            // Efeitos
            (Language::PtBr, Text::EffectMogging) => "MOGGING!",
            (Language::English, Text::EffectMogging) => "MOGGING!",
            (Language::PtBr, Text::EffectBlocked) => "BLOQUEADO",
            (Language::English, Text::EffectBlocked) => "BLOCKED",
            (Language::PtBr, Text::EffectLevel) => "NÍVEL",
            (Language::English, Text::EffectLevel) => "LEVEL",
            (Language::PtBr, Text::EffectNoAura) => "Sem aura!",
            (Language::English, Text::EffectNoAura) => "No aura!",
            (Language::PtBr, Text::EffectDeath) => "VOCÊ CAIU!",
            (Language::English, Text::EffectDeath) => "YOU FELL!",
            (Language::PtBr, Text::EffectVictory) => "FASE CONCLUÍDA!",
            (Language::English, Text::EffectVictory) => "STAGE CLEARED!",

            // Save
            (Language::PtBr, Text::SaveReasonDeath) => "morte",
            (Language::English, Text::SaveReasonDeath) => "death",
            (Language::PtBr, Text::SaveReasonAttribute) => "atributo",
            (Language::English, Text::SaveReasonAttribute) => "attribute",
            (Language::PtBr, Text::SaveReasonProgress) => "progresso",
            (Language::English, Text::SaveReasonProgress) => "progress",
        }
    }

    pub fn attribute(self, attribute: Attribute) -> &'static str {
        match attribute {
            Attribute::Speed => self.t(Text::AttributeSpeed),
            Attribute::Strength => self.t(Text::AttributeStrength),
            Attribute::Health => self.t(Text::AttributeHealth),
            Attribute::Aura => self.t(Text::AttributeAura),
        }
    }

    pub fn item(self, kind: ItemKind) -> &'static str {
        match kind {
            ItemKind::Sword => self.t(Text::ItemSword),
            ItemKind::Bow => self.t(Text::ItemBow),
            ItemKind::Armor => self.t(Text::ItemArmor),
            ItemKind::Amulet => self.t(Text::ItemAmulet),
            ItemKind::Potion => self.t(Text::ItemPotion),
            ItemKind::NutellaGun => self.t(Text::ItemNutellaGun),
        }
    }

    pub fn skill(self, id: SkillId) -> &'static str {
        match id {
            SkillId::Jump => self.t(Text::MoveJump),
            SkillId::Dash => self.t(Text::MoveDash),
            SkillId::Fly => self.t(Text::MoveFly),
            SkillId::Melee => self.t(Text::MoveMelee),
            SkillId::Mogging => self.t(Text::MoveMogging),
            SkillId::NutellaShot => self.t(Text::SkillNutellaShot),
            SkillId::Shout => self.t(Text::SkillShout),
        }
    }

    /// Compat: mapeia nomes antigos usados por HUD legado.
    pub fn movement_label(self, _label: &str) -> &'static str {
        ""
    }

    pub fn equip_slot(self, slot: EquipSlot) -> &'static str {
        match slot {
            EquipSlot::Weapon => self.t(Text::WeaponSlot),
            EquipSlot::Armor => self.t(Text::ArmorSlot),
            EquipSlot::Accessory => self.t(Text::AccessorySlot),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// List of keys for coverage testing. Keep in sync with the enum.
    const ALL_TEXT: [Text; 109] = [
        Text::MenuSubtitle,
        Text::Play,
        Text::Resume,
        Text::ManageSaves,
        Text::Options,
        Text::Back,
        Text::ChooseSlot,
        Text::EmptySlot,
        Text::Load,
        Text::NewGame,
        Text::Delete,
        Text::DeleteConfirm,
        Text::Confirm,
        Text::Cancel,
        Text::Language,
        Text::InterfaceSize,
        Text::MobileControls,
        Text::MobileControlsOn,
        Text::MobileControlsOff,
        Text::MobileControlsAuto,
        Text::SlotLabel,
        Text::SlotDeleted,
        Text::LevelLoadFailed,
        Text::LevelLabel,
        Text::PointsLabel,
        Text::MenuHint,
        Text::MainTitle,
        Text::SavesTitle,
        Text::OptionsTitle,
        Text::LevelShort,
        Text::Health,
        Text::Aura,
        Text::Xp,
        Text::Points,
        Text::Status,
        Text::Inventory,
        Text::Equipment,
        Text::Skills,
        Text::MenuButton,
        Text::DragHint,
        Text::Close,
        Text::MoggingReady,
        Text::MoggingCooldown,
        Text::SavedPrefix,
        Text::SaveFailed,
        Text::StatusTitle,
        Text::AttributeSpeed,
        Text::AttributeStrength,
        Text::AttributeHealth,
        Text::AttributeAura,
        Text::EffectSpeed,
        Text::EffectStrength,
        Text::EffectHealth,
        Text::EffectAura,
        Text::CurrentValue,
        Text::LevelUpTitle,
        Text::PointsToSpend,
        Text::SpendHint,
        Text::BackToGame,
        Text::InventoryTitle,
        Text::InventoryEmpty,
        Text::EquipmentTitle,
        Text::WeaponSlot,
        Text::ArmorSlot,
        Text::AccessorySlot,
        Text::NoItem,
        Text::SkillsTitle,
        Text::Movements,
        Text::Attacks,
        Text::NoMoves,
        Text::MoveWalk,
        Text::MoveJump,
        Text::MoveDash,
        Text::MoveFly,
        Text::MoveSixSeven,
        Text::MoveMelee,
        Text::MoveMogging,
        Text::SkillNutellaShot,
        Text::SkillShout,
        Text::SkillEmpty,
        Text::SkillNoAura,
        Text::SkillCooldown,
        Text::SkillLocked,
        Text::ItemSword,
        Text::ItemBow,
        Text::ItemArmor,
        Text::ItemAmulet,
        Text::ItemPotion,
        Text::ItemNutellaGun,
        Text::PhaseBanner,
        Text::ControlsLine1,
        Text::ControlsLine2,
        Text::DeathTitle,
        Text::RespawnIn,
        Text::ProgressionKept,
        Text::RestartNow,
        Text::VictoryTitle,
        Text::ProgressSaved,
        Text::PlayAgain,
        Text::ReturnToMenu,
        Text::EffectMogging,
        Text::EffectBlocked,
        Text::EffectLevel,
        Text::EffectNoAura,
        Text::EffectDeath,
        Text::EffectVictory,
        Text::SaveReasonDeath,
        Text::SaveReasonAttribute,
        Text::SaveReasonProgress,
    ];

    #[test]
    fn every_key_has_a_non_empty_translation_in_both_languages() {
        for language in LANGUAGES {
            let i18n = I18n::new(language);
            for key in ALL_TEXT {
                assert!(!i18n.t(key).trim().is_empty(), "{language:?} {key:?}");
            }
        }
    }

    #[test]
    fn switching_language_changes_labels() {
        let mut i18n = I18n::new(Language::PtBr);
        assert_eq!(i18n.t(Text::Play), "Jogar");
        i18n.set_language(Language::English);
        assert_eq!(i18n.t(Text::Play), "Play");
        assert_eq!(i18n.attribute(Attribute::Speed), "Speed");
        assert_eq!(i18n.item(ItemKind::Sword), "Sword");
        assert_eq!(i18n.skill(SkillId::Mogging), "Mogging");
        assert_eq!(i18n.equip_slot(EquipSlot::Weapon), "Weapon");
    }

    #[test]
    fn language_codes_roundtrip() {
        for language in LANGUAGES {
            assert_eq!(Language::from_code(language.code()), Some(language));
            assert!(!language.label().is_empty());
        }
        assert_eq!(Language::from_code("xx"), None);
    }
}
