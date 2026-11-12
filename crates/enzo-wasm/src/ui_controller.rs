//! UI controller: orchestrates menu/HUD/game UI state and DOM actions.
//!
//! The DOM/HTML layer interacts with Rust only through [`crate::ui_bridge`] exports.
//! This controller owns the translation between semantic UI actions and game/menu
//! state changes, and publishes serializable [`UiState`] for rendering.

use enzo_core::entities::EquipSlot;
use enzo_core::input::{Attribute, InputAction};
use enzo_core::progression::{xp_to_next, SaveData};
use enzo_core::skills::SkillCategory;
use enzo_core::snapshot::{EntitySnapshot, WorldSnapshot};
use macroquad::prelude::KeyCode;
use serde::Serialize;

use crate::i18n::{I18n, Text};
use crate::input::KeyBindings;
use crate::menu::{Menu, MenuScreen};
use crate::save::{self, SLOT_COUNT};
use crate::ui_bridge::UiAction;

// --- Serializable UI state (read by DOM) ---

#[derive(Debug, Clone, Serialize)]
pub struct UiLabels {
    pub play: String,
    pub resume: String,
    pub options: String,
    pub back: String,
    pub saves_title: String,
    pub options_title: String,
    pub choose_slot: String,
    pub slot: String,
    pub empty_slot: String,
    pub load: String,
    pub new_game: String,
    pub delete: String,
    pub delete_confirm: String,
    pub confirm: String,
    pub cancel: String,
    pub language: String,
    pub interface_size: String,
    pub mobile_controls: String,
    pub on: String,
    pub off: String,
    pub auto: String,
    pub level: String,
    pub points: String,
    pub menu: String,
    pub restart: String,
    pub level_up: String,
    pub victory: String,
    pub game_over: String,
    pub status: String,
    pub equipment: String,
    pub inventory: String,
    pub skills: String,
    pub status_title: String,
    pub equipment_title: String,
    pub inventory_title: String,
    pub skills_title: String,
    pub inventory_empty: String,
    pub no_item: String,
    pub assign: String,
    pub clear: String,
    pub equipped: String,
    pub locked: String,
    pub close: String,
    pub customize: String,
    pub character_editor: String,
    pub character: String,
    pub skin_tone: String,
    pub hair_style: String,
    pub hair_color: String,
    pub face: String,
    pub outfit_top: String,
    pub outfit_bottom: String,
    pub shoes: String,
    pub main_clothing_color: String,
    pub equip: String,
    pub unequip: String,
}

impl UiLabels {
    pub fn from_i18n(i18n: I18n) -> Self {
        Self {
            play: i18n.t(Text::Play).to_owned(),
            resume: i18n.t(Text::Resume).to_owned(),
            options: i18n.t(Text::Options).to_owned(),
            back: i18n.t(Text::Back).to_owned(),
            saves_title: i18n.t(Text::SavesTitle).to_owned(),
            options_title: i18n.t(Text::OptionsTitle).to_owned(),
            choose_slot: i18n.t(Text::ChooseSlot).to_owned(),
            slot: i18n.t(Text::SlotLabel).to_owned(),
            empty_slot: i18n.t(Text::EmptySlot).to_owned(),
            load: i18n.t(Text::Load).to_owned(),
            new_game: i18n.t(Text::NewGame).to_owned(),
            delete: i18n.t(Text::Delete).to_owned(),
            delete_confirm: i18n.t(Text::DeleteConfirm).to_owned(),
            confirm: i18n.t(Text::Confirm).to_owned(),
            cancel: i18n.t(Text::Cancel).to_owned(),
            language: i18n.t(Text::Language).to_owned(),
            interface_size: i18n.t(Text::InterfaceSize).to_owned(),
            mobile_controls: i18n.t(Text::MobileControls).to_owned(),
            on: i18n.t(Text::MobileControlsOn).to_owned(),
            off: i18n.t(Text::MobileControlsOff).to_owned(),
            auto: i18n.t(Text::MobileControlsAuto).to_owned(),
            level: i18n.t(Text::LevelLabel).to_owned(),
            points: i18n.t(Text::PointsLabel).to_owned(),
            menu: i18n.t(Text::MenuButton).to_owned(),
            restart: i18n.t(Text::RestartNow).to_owned(),
            level_up: i18n.t(Text::LevelUpTitle).to_owned(),
            victory: i18n.t(Text::VictoryTitle).to_owned(),
            game_over: if i18n.language().code() == "en" {
                "Game Over".to_owned()
            } else {
                "Fim de jogo".to_owned()
            },
            status: i18n.t(Text::Status).to_owned(),
            equipment: i18n.t(Text::Equipment).to_owned(),
            inventory: i18n.t(Text::Inventory).to_owned(),
            skills: i18n.t(Text::Skills).to_owned(),
            status_title: i18n.t(Text::StatusTitle).to_owned(),
            equipment_title: i18n.t(Text::EquipmentTitle).to_owned(),
            inventory_title: i18n.t(Text::InventoryTitle).to_owned(),
            skills_title: i18n.t(Text::SkillsTitle).to_owned(),
            inventory_empty: i18n.t(Text::InventoryEmpty).to_owned(),
            no_item: i18n.t(Text::NoItem).to_owned(),
            assign: if i18n.language().code() == "en" {
                "Assign".to_owned()
            } else {
                "Alocar".to_owned()
            },
            clear: if i18n.language().code() == "en" {
                "Clear".to_owned()
            } else {
                "Limpar".to_owned()
            },
            equipped: if i18n.language().code() == "en" {
                "Equipped".to_owned()
            } else {
                "Equipado".to_owned()
            },
            locked: i18n.t(Text::SkillLocked).to_owned(),
            close: if i18n.language().code() == "en" {
                "Close".to_owned()
            } else {
                "Fechar".to_owned()
            },
            customize: if i18n.language().code() == "en" {
                "Customize".to_owned()
            } else {
                "Personalizar".to_owned()
            },
            character_editor: if i18n.language().code() == "en" {
                "Character Editor".to_owned()
            } else {
                "Editor de Personagem".to_owned()
            },
            character: if i18n.language().code() == "en" {
                "Character".to_owned()
            } else {
                "Personagem".to_owned()
            },
            skin_tone: if i18n.language().code() == "en" {
                "Skin tone".to_owned()
            } else {
                "Tom de pele".to_owned()
            },
            hair_style: if i18n.language().code() == "en" {
                "Hair style".to_owned()
            } else {
                "Estilo de cabelo".to_owned()
            },
            hair_color: if i18n.language().code() == "en" {
                "Hair color".to_owned()
            } else {
                "Cor do cabelo".to_owned()
            },
            face: if i18n.language().code() == "en" {
                "Face".to_owned()
            } else {
                "Rosto".to_owned()
            },
            outfit_top: if i18n.language().code() == "en" {
                "Shirt".to_owned()
            } else {
                "Camisa".to_owned()
            },
            outfit_bottom: if i18n.language().code() == "en" {
                "Pants".to_owned()
            } else {
                "Calça".to_owned()
            },
            shoes: if i18n.language().code() == "en" {
                "Shoes".to_owned()
            } else {
                "Tênis".to_owned()
            },
            main_clothing_color: if i18n.language().code() == "en" {
                "Main clothing color".to_owned()
            } else {
                "Cor principal da roupa".to_owned()
            },
            equip: if i18n.language().code() == "en" {
                "Equip".to_owned()
            } else {
                "Equipar".to_owned()
            },
            unequip: if i18n.language().code() == "en" {
                "Unequip".to_owned()
            } else {
                "Desequipar".to_owned()
            },
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct UiSettings {
    pub language: String,
    pub mobile_controls: String,
    pub interface_scale: f32,
}

impl UiSettings {
    pub fn current() -> Self {
        Self {
            language: save::load_language().code().to_owned(),
            mobile_controls: save::load_mobile_controls_mode().code().to_owned(),
            interface_scale: save::load_interface_scale(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct UiSlotSummary {
    pub slot: u8,
    pub occupied: bool,
    pub level: Option<u32>,
    pub xp: Option<f32>,
    pub xp_to_next: Option<f32>,
    pub points: Option<u32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct UiMenuState {
    pub screen: String,
    pub has_background_game: bool,
    pub pending_delete: Option<u8>,
    pub slots: Vec<UiSlotSummary>,
    pub toast: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct UiSkillSlotState {
    pub slot: u8,
    pub skill: Option<u8>,
    pub skill_name: Option<String>,
    pub available: bool,
    pub cooldown: f32,
    pub cooldown_max: f32,
    pub aura_cost: f32,
    pub active: bool,
    pub lock_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct UiCatalogEntry {
    pub skill: u8,
    pub name: String,
    pub category: String,
    pub available: bool,
    pub lock_reason: Option<String>,
    pub equipped_slot: Option<u8>,
    pub aura_cost: f32,
    pub cooldown_max: f32,
}

#[derive(Debug, Clone, Serialize)]
pub struct UiEquipmentSlot {
    pub slot: String,
    pub slot_code: u8,
    pub slot_label: String,
    pub item: Option<String>,
    pub item_code: Option<u8>,
}

#[derive(Debug, Clone, Serialize)]
pub struct UiInventoryItem {
    pub code: u8,
    pub kind: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct UiAttributes {
    pub speed: u32,
    pub strength: u32,
    pub health: u32,
    pub aura: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct UiVisualProfile {
    pub skin_tone: u8,
    pub hair_style: u8,
    pub hair_color: u8,
    pub face: u8,
    pub outfit_top: u8,
    pub outfit_bottom: u8,
    pub shoes: u8,
}

#[derive(Debug, Clone, Serialize)]
pub struct UiPlayerState {
    pub character: String,
    pub character_code: u8,
    pub visual_profile: UiVisualProfile,
    pub health: f32,
    pub max_health: f32,
    pub aura: f32,
    pub max_aura: f32,
    pub level: u32,
    pub xp: f32,
    pub xp_to_next: f32,
    pub points: u32,
    pub pending_level_up: bool,
    pub attributes: UiAttributes,
    pub slots: Vec<UiSkillSlotState>,
    pub catalog: Vec<UiCatalogEntry>,
    pub equipment_slots: Vec<UiEquipmentSlot>,
    pub inventory: Vec<UiInventoryItem>,
}

#[derive(Debug, Clone, Serialize)]
pub struct UiControlBinding {
    pub action: String,
    pub label: String,
    pub keys: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct UiState {
    pub scene: String,
    pub phase: Option<String>,
    pub player: Option<UiPlayerState>,
    pub menu: Option<UiMenuState>,
    pub game_toast: Option<String>,
    pub settings: UiSettings,
    pub labels: UiLabels,
    pub controls: Vec<UiControlBinding>,
}

// --- State building ---

fn slot_summary(slot: usize, data: Option<&SaveData>) -> UiSlotSummary {
    match data {
        Some(data) => UiSlotSummary {
            slot: slot as u8,
            occupied: true,
            level: Some(data.level),
            xp: Some(data.xp),
            xp_to_next: Some(xp_to_next(data.level)),
            points: Some(data.points),
        },
        None => UiSlotSummary {
            slot: slot as u8,
            occupied: false,
            level: None,
            xp: None,
            xp_to_next: None,
            points: None,
        },
    }
}

fn menu_state(menu: &Menu) -> UiMenuState {
    let screen = match menu.current() {
        MenuScreen::Main => "main",
        MenuScreen::Saves => "saves",
        MenuScreen::Options => "options",
        MenuScreen::Customize => "customize",
    };
    let mut slots = Vec::with_capacity(SLOT_COUNT);
    for i in 0..SLOT_COUNT {
        slots.push(slot_summary(i, menu.slot(i)));
    }
    UiMenuState {
        screen: screen.to_owned(),
        has_background_game: menu.has_background_game,
        pending_delete: menu.pending_delete_slot().map(|s| s as u8),
        slots,
        toast: menu.toast_text().map(|s: &str| s.to_owned()),
    }
}

fn category_label(category: SkillCategory) -> &'static str {
    match category {
        SkillCategory::Movement => "Movement",
        SkillCategory::Attack => "Attack",
        SkillCategory::Utility => "Utility",
    }
}

fn equip_slot_code(slot: EquipSlot) -> &'static str {
    match slot {
        EquipSlot::Weapon => "weapon",
        EquipSlot::Armor => "armor",
        EquipSlot::Accessory => "accessory",
        EquipSlot::Head => "head",
        EquipSlot::Feet => "feet",
    }
}

fn draft_player_state(
    character: enzo_core::entities::PlayableCharacter,
    visual_profile: enzo_core::entities::VisualProfile,
) -> UiPlayerState {
    UiPlayerState {
        character: format!("{:?}", character),
        character_code: character.code(),
        visual_profile: UiVisualProfile {
            skin_tone: visual_profile.skin_tone,
            hair_style: visual_profile.hair_style,
            hair_color: visual_profile.hair_color,
            face: visual_profile.face,
            outfit_top: visual_profile.outfit_top,
            outfit_bottom: visual_profile.outfit_bottom,
            shoes: visual_profile.shoes,
        },
        health: 100.0,
        max_health: 100.0,
        aura: 100.0,
        max_aura: 100.0,
        level: 1,
        xp: 0.0,
        xp_to_next: 0.0,
        points: 0,
        pending_level_up: false,
        attributes: UiAttributes {
            speed: 0,
            strength: 0,
            health: 0,
            aura: 0,
        },
        slots: Vec::new(),
        catalog: Vec::new(),
        equipment_slots: Vec::new(),
        inventory: Vec::new(),
    }
}

fn player_state(entity: &EntitySnapshot, i18n: I18n) -> Option<UiPlayerState> {
    let p = entity.player.as_ref()?;
    let slots = p
        .slots
        .iter()
        .enumerate()
        .map(|(i, slot)| UiSkillSlotState {
            slot: i as u8,
            skill: slot.skill.map(|s| s.code()),
            skill_name: slot.skill.map(|s| i18n.skill(s).to_owned()),
            available: slot.available,
            cooldown: slot.cooldown,
            cooldown_max: slot.cooldown_max,
            aura_cost: slot.aura_cost,
            active: slot.active,
            lock_reason: slot.lock.as_ref().map(|l| format!("{:?}", l)),
        })
        .collect();

    let catalog = p
        .catalog
        .iter()
        .map(|entry| UiCatalogEntry {
            skill: entry.skill.code(),
            name: i18n.skill(entry.skill).to_owned(),
            category: category_label(entry.skill.def().category).to_owned(),
            available: entry.available,
            lock_reason: entry.lock.as_ref().map(|l| format!("{:?}", l)),
            equipped_slot: entry.equipped_slot.map(|s| s.index() as u8),
            aura_cost: entry.aura_cost,
            cooldown_max: entry.cooldown_max,
        })
        .collect();

    let equipment_slots = EquipSlot::ALL
        .into_iter()
        .map(|slot| {
            let eq = p.equipped[slot.index()].as_ref();
            UiEquipmentSlot {
                slot: equip_slot_code(slot).to_owned(),
                slot_code: slot.code(),
                slot_label: i18n.equip_slot(slot).to_owned(),
                item: eq.map(|e| i18n.item(e.kind).to_owned()),
                item_code: eq.map(|e| e.kind.code()),
            }
        })
        .collect();

    let inventory = p
        .inventory
        .iter()
        .map(|kind| UiInventoryItem {
            code: kind.code(),
            kind: format!("{:?}", kind),
            name: i18n.item(*kind).to_owned(),
        })
        .collect();

    Some(UiPlayerState {
        character: format!("{:?}", p.character),
        character_code: p.character.code(),
        visual_profile: UiVisualProfile {
            skin_tone: p.visual_profile.skin_tone,
            hair_style: p.visual_profile.hair_style,
            hair_color: p.visual_profile.hair_color,
            face: p.visual_profile.face,
            outfit_top: p.visual_profile.outfit_top,
            outfit_bottom: p.visual_profile.outfit_bottom,
            shoes: p.visual_profile.shoes,
        },
        health: entity.health,
        max_health: entity.max_health,
        aura: p.aura,
        max_aura: p.max_aura,
        level: p.level,
        xp: p.xp,
        xp_to_next: p.xp_to_next,
        points: p.points,
        pending_level_up: p.pending_level_up,
        attributes: UiAttributes {
            speed: p.attributes.speed,
            strength: p.attributes.strength,
            health: p.attributes.health,
            aura: p.attributes.aura,
        },
        slots,
        catalog,
        equipment_slots,
        inventory,
    })
}

pub fn build_state(
    scene: &str,
    snapshot: Option<&WorldSnapshot>,
    menu: Option<&Menu>,
    i18n: I18n,
    game_toast: Option<&str>,
    bindings: Option<&KeyBindings>,
) -> UiState {
    let (phase, mut player) = if let Some(snapshot) = snapshot {
        (
            Some(format!("{:?}", snapshot.phase)),
            snapshot.player().and_then(|e| player_state(e, i18n)),
        )
    } else {
        (None, None)
    };
    if player.is_none() {
        if let Some(menu) = menu {
            player = Some(draft_player_state(
                menu.draft_character,
                menu.draft_visual_profile,
            ));
        }
    }
    let default_bindings;
    let bindings = match bindings {
        Some(b) => b,
        None => {
            default_bindings = crate::input::defaults::default_bindings();
            &default_bindings
        }
    };
    UiState {
        scene: scene.to_owned(),
        phase,
        player,
        menu: menu.map(menu_state),
        game_toast: game_toast.map(|s| s.to_owned()),
        settings: UiSettings::current(),
        labels: UiLabels::from_i18n(i18n),
        controls: control_bindings(bindings, i18n),
    }
}

pub fn publish(
    scene: &str,
    snapshot: Option<&WorldSnapshot>,
    menu: Option<&Menu>,
    i18n: I18n,
    game_toast: Option<&str>,
    bindings: Option<&KeyBindings>,
) {
    let state = build_state(scene, snapshot, menu, i18n, game_toast, bindings);
    if let Ok(json) = serde_json::to_string(&state) {
        crate::ui_bridge::set_ui_state_json(json);
    }
}

fn control_bindings(bindings: &KeyBindings, i18n: I18n) -> Vec<UiControlBinding> {
    let is_en = i18n.language().code() == "en";
    let mut out = Vec::new();
    for action in InputAction::ALL {
        // Attribute allocation is performed via the in-game UI, not keyboard buttons.
        if action.attribute().is_some() {
            continue;
        }
        let keys: Vec<String> = bindings
            .keys(action)
            .iter()
            .map(|k| key_label(*k))
            .collect();
        out.push(UiControlBinding {
            action: format!("{:?}", action),
            label: action_label(action, is_en).to_owned(),
            keys,
        });
    }
    out
}

fn action_label(action: InputAction, is_en: bool) -> &'static str {
    match (action, is_en) {
        (InputAction::MoveLeft, true) => "Move left",
        (InputAction::MoveLeft, false) => "Mover para esquerda",
        (InputAction::MoveRight, true) => "Move right",
        (InputAction::MoveRight, false) => "Mover para direita",
        (InputAction::MoveUp, true) => "Move up / climb / fly up",
        (InputAction::MoveUp, false) => "Mover para cima / escalar / voar",
        (InputAction::MoveDown, true) => "Move down / crouch",
        (InputAction::MoveDown, false) => "Mover para baixo / agachar",
        (InputAction::Slot1, true) => "Action slot 1 (Jump by default)",
        (InputAction::Slot1, false) => "Slot de ação 1 (Pulo por padrão)",
        (InputAction::Slot2, true) => "Action slot 2",
        (InputAction::Slot2, false) => "Slot de ação 2",
        (InputAction::Slot3, true) => "Action slot 3",
        (InputAction::Slot3, false) => "Slot de ação 3",
        (InputAction::Slot4, true) => "Action slot 4",
        (InputAction::Slot4, false) => "Slot de ação 4",
        (InputAction::Six, true) => "Aura farming: 6 (alternate with 7)",
        (InputAction::Six, false) => "Farmar aura: 6 (alterne com 7)",
        (InputAction::Seven, true) => "Aura farming: 7 (alternate with 6)",
        (InputAction::Seven, false) => "Farmar aura: 7 (alterne com 6)",
        (InputAction::AllocateSpeed, true) => "Allocate: Speed",
        (InputAction::AllocateSpeed, false) => "Distribuir: Velocidade",
        (InputAction::AllocateStrength, true) => "Allocate: Strength",
        (InputAction::AllocateStrength, false) => "Distribuir: Força",
        (InputAction::AllocateHealth, true) => "Allocate: Health",
        (InputAction::AllocateHealth, false) => "Distribuir: Vida",
        (InputAction::AllocateAura, true) => "Allocate: Aura",
        (InputAction::AllocateAura, false) => "Distribuir: Aura",
        (InputAction::AllocateEgo, true) => "Allocate: Ego",
        (InputAction::AllocateEgo, false) => "Distribuir: Ego",
        (InputAction::Confirm, true) => "Confirm",
        (InputAction::Confirm, false) => "Confirmar",
        (InputAction::Restart, true) => "Restart",
        (InputAction::Restart, false) => "Reiniciar",
        (InputAction::OpenMenu, true) => "Open menu / pause",
        (InputAction::OpenMenu, false) => "Abrir menu / pausar",
        (InputAction::OpenSkills, true) => "Open skills",
        (InputAction::OpenSkills, false) => "Abrir habilidades",
    }
}

fn key_label(key: KeyCode) -> String {
    match key {
        KeyCode::Space => "Space".to_owned(),
        KeyCode::Escape => "Esc".to_owned(),
        KeyCode::Enter | KeyCode::KpEnter => "Enter".to_owned(),
        KeyCode::Left => "←".to_owned(),
        KeyCode::Right => "→".to_owned(),
        KeyCode::Up => "↑".to_owned(),
        KeyCode::Down => "↓".to_owned(),
        KeyCode::LeftShift | KeyCode::RightShift => "Shift".to_owned(),
        KeyCode::LeftControl | KeyCode::RightControl => "Ctrl".to_owned(),
        KeyCode::LeftAlt | KeyCode::RightAlt => "Alt".to_owned(),
        KeyCode::Key0 | KeyCode::Kp0 => "0".to_owned(),
        KeyCode::Key1 | KeyCode::Kp1 => "1".to_owned(),
        KeyCode::Key2 | KeyCode::Kp2 => "2".to_owned(),
        KeyCode::Key3 | KeyCode::Kp3 => "3".to_owned(),
        KeyCode::Key4 | KeyCode::Kp4 => "4".to_owned(),
        KeyCode::Key5 | KeyCode::Kp5 => "5".to_owned(),
        KeyCode::Key6 | KeyCode::Kp6 => "6".to_owned(),
        KeyCode::Key7 | KeyCode::Kp7 => "7".to_owned(),
        KeyCode::Key8 | KeyCode::Kp8 => "8".to_owned(),
        KeyCode::Key9 | KeyCode::Kp9 => "9".to_owned(),
        KeyCode::A => "A".to_owned(),
        KeyCode::B => "B".to_owned(),
        KeyCode::C => "C".to_owned(),
        KeyCode::D => "D".to_owned(),
        KeyCode::E => "E".to_owned(),
        KeyCode::F => "F".to_owned(),
        KeyCode::G => "G".to_owned(),
        KeyCode::H => "H".to_owned(),
        KeyCode::I => "I".to_owned(),
        KeyCode::J => "J".to_owned(),
        KeyCode::K => "K".to_owned(),
        KeyCode::L => "L".to_owned(),
        KeyCode::M => "M".to_owned(),
        KeyCode::N => "N".to_owned(),
        KeyCode::O => "O".to_owned(),
        KeyCode::P => "P".to_owned(),
        KeyCode::Q => "Q".to_owned(),
        KeyCode::R => "R".to_owned(),
        KeyCode::S => "S".to_owned(),
        KeyCode::T => "T".to_owned(),
        KeyCode::U => "U".to_owned(),
        KeyCode::V => "V".to_owned(),
        KeyCode::W => "W".to_owned(),
        KeyCode::X => "X".to_owned(),
        KeyCode::Y => "Y".to_owned(),
        KeyCode::Z => "Z".to_owned(),
        other => format!("{other:?}"),
    }
}

// --- Action handling (semantic, no game rules in DOM) ---

/// Apply menu-only UI actions. Returns `true` if the action was handled as a menu action.
pub fn apply_menu_action(
    menu: &mut Menu,
    action: UiAction,
    i18n: &mut I18n,
) -> Option<crate::menu::MenuAction> {
    match action {
        UiAction::MenuBack => {
            if menu.pending_delete_slot().is_some() {
                menu.apply_ui_action(UiAction::MenuCancelDelete, i18n);
            } else {
                menu.apply_ui_action(UiAction::MenuBack, i18n);
            }
            None
        }
        UiAction::MenuOpenSaves
        | UiAction::MenuOpenOptions
        | UiAction::MenuOpenCustomize
        | UiAction::SetPlayableCharacter { .. }
        | UiAction::SetVisualProfileField { .. }
        | UiAction::MenuStart { .. }
        | UiAction::MenuResume
        | UiAction::MenuRequestDelete { .. }
        | UiAction::MenuCancelDelete
        | UiAction::MenuConfirmDelete
        | UiAction::SetLanguage { .. }
        | UiAction::SetMobileControls { .. }
        | UiAction::SetInterfaceScale { .. } => menu.apply_ui_action(action, i18n),
        _ => None,
    }
}

pub fn attribute_from_code(code: u8) -> Option<Attribute> {
    Attribute::ALL.get(code as usize).copied()
}
