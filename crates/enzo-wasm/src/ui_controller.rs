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
    pub outfit_top_detail1: String,
    pub outfit_top_detail2: String,
    pub shoes: String,
    pub main_clothing_color: String,
    pub equip: String,
    pub unequip: String,
    pub skill_details: String,
    pub skill_level: String,
    pub skill_tier: String,
    pub skill_damage: String,
    pub skill_no_damage: String,
    pub skill_aura_cost: String,
    pub skill_cooldown: String,
    // Sandbox/debug
    pub sandbox: String,
    pub sandbox_debug: String,
    pub tab_character: String,
    pub tab_blocks: String,
    pub tab_enemies: String,
    pub tab_items: String,
    pub sigma_mode: String,
    pub level_up_debug: String,
    pub grant_levels: String,
    pub set_level: String,
    pub reset_attrs: String,
    pub die: String,
    pub reset_sandbox: String,
    pub place: String,
    pub remove: String,
    pub spawn: String,
}

impl UiLabels {
    pub fn from_i18n(i18n: I18n) -> Self {
        let en = i18n.language().code() == "en";
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
            game_over: if en {
                "Game Over".to_owned()
            } else {
                "Sobrou nada pro beta".to_owned()
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
            assign: if en {
                "Assign".to_owned()
            } else {
                "Alocar".to_owned()
            },
            clear: if en {
                "Clear".to_owned()
            } else {
                "Limpar".to_owned()
            },
            equipped: if en {
                "Equipped".to_owned()
            } else {
                "Equipado".to_owned()
            },
            locked: i18n.t(Text::SkillLocked).to_owned(),
            close: if en {
                "Close".to_owned()
            } else {
                "Fechar".to_owned()
            },
            customize: if en {
                "Customize".to_owned()
            } else {
                "Personalizar".to_owned()
            },
            character_editor: if en {
                "Character Editor".to_owned()
            } else {
                "Editor de Personagem".to_owned()
            },
            character: if en {
                "Character".to_owned()
            } else {
                "Personagem".to_owned()
            },
            skin_tone: if en {
                "Skin tone".to_owned()
            } else {
                "Tom de pele".to_owned()
            },
            hair_style: if en {
                "Hair style".to_owned()
            } else {
                "Estilo de cabelo".to_owned()
            },
            hair_color: if en {
                "Hair color".to_owned()
            } else {
                "Cor do cabelo".to_owned()
            },
            face: if en {
                "Face".to_owned()
            } else {
                "Rosto".to_owned()
            },
            outfit_top: if en {
                "Shirt color".to_owned()
            } else {
                "Cor da camiseta".to_owned()
            },
            outfit_bottom: if en {
                "Pants color".to_owned()
            } else {
                "Cor da calça".to_owned()
            },
            outfit_top_detail1: if en {
                "Outfit detail 1".to_owned()
            } else {
                "Detalhe da roupa 1".to_owned()
            },
            outfit_top_detail2: if en {
                "Outfit detail 2".to_owned()
            } else {
                "Detalhe da roupa 2".to_owned()
            },
            shoes: if en {
                "Details color".to_owned()
            } else {
                "Cor dos detalhes".to_owned()
            },
            main_clothing_color: if en {
                "Clothing colors".to_owned()
            } else {
                "Cores das roupas".to_owned()
            },
            equip: if en {
                "Equip".to_owned()
            } else {
                "Equipar".to_owned()
            },
            unequip: if en {
                "Unequip".to_owned()
            } else {
                "Desequipar".to_owned()
            },
            skill_details: i18n.t(Text::SkillDetails).to_owned(),
            skill_level: i18n.t(Text::SkillLevel).to_owned(),
            skill_tier: i18n.t(Text::SkillTier).to_owned(),
            skill_damage: i18n.t(Text::SkillDamage).to_owned(),
            skill_no_damage: i18n.t(Text::SkillNoDamage).to_owned(),
            skill_aura_cost: i18n.t(Text::SkillAuraCost).to_owned(),
            skill_cooldown: i18n.t(Text::SkillCooldown).to_owned(),
            sandbox: "Sandbox".to_owned(),
            sandbox_debug: "Debug".to_owned(),
            tab_character: if en {
                "Character".to_owned()
            } else {
                "Personagem".to_owned()
            },
            tab_blocks: if en {
                "Blocks".to_owned()
            } else {
                "Blocos".to_owned()
            },
            tab_enemies: if en {
                "Enemies".to_owned()
            } else {
                "Inimigos".to_owned()
            },
            tab_items: if en {
                "Items".to_owned()
            } else {
                "Itens".to_owned()
            },
            sigma_mode: if en {
                "Sigma mode".to_owned()
            } else {
                "Modo sigma".to_owned()
            },
            level_up_debug: if en {
                "Level up".to_owned()
            } else {
                "Subir nível".to_owned()
            },
            grant_levels: if en {
                "Grant +5 levels".to_owned()
            } else {
                "Dar +5 níveis".to_owned()
            },
            set_level: if en {
                "Set level".to_owned()
            } else {
                "Definir nível".to_owned()
            },
            reset_attrs: if en {
                "Reset attributes".to_owned()
            } else {
                "Resetar atributos".to_owned()
            },
            die: if en {
                "Die".to_owned()
            } else {
                "Morrer".to_owned()
            },
            reset_sandbox: if en {
                "Reset sandbox".to_owned()
            } else {
                "Resetar sandbox".to_owned()
            },
            place: if en {
                "Place".to_owned()
            } else {
                "Colocar".to_owned()
            },
            remove: if en {
                "Remove".to_owned()
            } else {
                "Remover".to_owned()
            },
            spawn: if en {
                "Spawn".to_owned()
            } else {
                "Spawnar".to_owned()
            },
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct UiSettings {
    pub language: String,
    pub mobile_controls: String,
    pub interface_scale: f32,
    pub interface_scale_auto: bool,
}

impl UiSettings {
    pub fn current() -> Self {
        Self {
            language: save::load_language().code().to_owned(),
            mobile_controls: save::load_mobile_controls_mode().code().to_owned(),
            interface_scale: crate::ui::interface_scale(),
            interface_scale_auto: save::load_interface_scale_auto(),
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
    pub sandbox_unlocked: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct UiSkillSlotState {
    pub slot: u8,
    pub skill: Option<u8>,
    pub skill_name: Option<String>,
    pub skill_icon: Option<String>,
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
    pub icon: String,
    pub category: String,
    pub description: String,
    pub available: bool,
    pub lock_reason: Option<String>,
    pub equipped_slot: Option<u8>,
    pub aura_cost: f32,
    pub cooldown_max: f32,
    pub effective_level: u8,
    pub active_tier: Option<u8>,
    pub damage_preview: Vec<f32>,
    pub damage_calc: Option<String>,
    pub range_calc: Option<String>,
    pub knockback_calc: Option<String>,
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
    pub ego: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct UiVisualProfile {
    pub skin_tone: u8,
    pub hair_style: u8,
    pub hair_color: u8,
    pub face: u8,
    pub outfit_top: u8,
    pub outfit_bottom: u8,
    pub outfit_top_detail1: u8,
    pub outfit_top_detail2: u8,
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
    pub sigma_mode: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct UiControlBinding {
    pub action: String,
    pub action_code: u8,
    pub label: String,
    pub keys: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct UiSupportedKey {
    pub index: u8,
    pub name: String,
    pub label: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct UiSandboxBlockOption {
    pub code: u8,
    pub name: String,
    pub icon: Option<String>,
    /// Special "remove/eraser" tool entry.
    pub remove: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct UiSandboxEnemyOption {
    pub code: u8,
    pub kind: String,
    pub name: String,
    pub sprite: String,
    pub frame_x: i32,
    pub frame_y: i32,
    pub frame_w: i32,
    pub frame_h: i32,
    pub sheet_w: i32,
    pub sheet_h: i32,
}

#[derive(Debug, Clone, Serialize)]
pub struct UiSandboxItemOption {
    pub code: u8,
    pub name: String,
    pub icon: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct UiSandboxState {
    pub active: bool,
    pub tab: u8,        // 0=character,1=blocks,2=enemies,3=items
    pub block_mode: u8, // 0=place,1=remove
    pub selected_block: u8,
    pub selected_enemy: u8,
    pub selected_item: u8,
    pub blocks: Vec<UiSandboxBlockOption>,
    pub enemies: Vec<UiSandboxEnemyOption>,
    pub items: Vec<UiSandboxItemOption>,
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
    pub supported_keys: Vec<UiSupportedKey>,
    pub sandbox: Option<UiSandboxState>,
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
        MenuScreen::Controls => "controls",
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
        sandbox_unlocked: menu.sandbox_unlocked,
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
            outfit_top_detail1: visual_profile.outfit_top_detail1,
            outfit_top_detail2: visual_profile.outfit_top_detail2,
            shoes: visual_profile.shoes,
        },
        health: 10.0,
        max_health: 10.0,
        aura: 10.0,
        max_aura: 10.0,
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
            ego: 0,
        },
        slots: Vec::new(),
        catalog: Vec::new(),
        equipment_slots: Vec::new(),
        inventory: Vec::new(),
        sigma_mode: false,
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
            skill_icon: slot.skill.map(|s| s.def().assets.icon.to_owned()),
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
            icon: entry.skill.def().assets.icon.to_owned(),
            category: category_label(entry.skill.def().category).to_owned(),
            description: i18n.skill_description(entry.skill).to_owned(),
            available: entry.available,
            lock_reason: entry.lock.as_ref().map(|l| format!("{:?}", l)),
            equipped_slot: entry.equipped_slot.map(|s| s.index() as u8),
            aura_cost: entry.aura_cost,
            cooldown_max: entry.cooldown_max,
            effective_level: entry.effective_level,
            active_tier: entry.active_tier,
            damage_preview: entry.damage_preview.clone(),
            damage_calc: entry.damage_calc.clone(),
            range_calc: entry.range_calc.clone(),
            knockback_calc: entry.knockback_calc.clone(),
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
            outfit_top_detail1: p.visual_profile.outfit_top_detail1,
            outfit_top_detail2: p.visual_profile.outfit_top_detail2,
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
            ego: p.attributes.ego,
        },
        slots,
        catalog,
        equipment_slots,
        inventory,
        sigma_mode: p.sigma_mode,
    })
}

pub fn build_state(
    scene: &str,
    snapshot: Option<&WorldSnapshot>,
    menu: Option<&Menu>,
    i18n: I18n,
    game_toast: Option<&str>,
    bindings: Option<&KeyBindings>,
    sandbox: Option<UiSandboxState>,
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
        supported_keys: crate::input::keycode::SUPPORTED_KEYS
            .iter()
            .enumerate()
            .map(|(index, (name, label))| UiSupportedKey {
                index: index as u8,
                name: (*name).to_owned(),
                label: (*label).to_owned(),
            })
            .collect(),
        sandbox,
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
    publish_with_sandbox(scene, snapshot, menu, i18n, game_toast, bindings, None);
}

pub fn publish_with_sandbox(
    scene: &str,
    snapshot: Option<&WorldSnapshot>,
    menu: Option<&Menu>,
    i18n: I18n,
    game_toast: Option<&str>,
    bindings: Option<&KeyBindings>,
    sandbox: Option<UiSandboxState>,
) {
    let state = build_state(scene, snapshot, menu, i18n, game_toast, bindings, sandbox);
    if let Ok(json) = serde_json::to_string(&state) {
        crate::ui_bridge::set_ui_state_json(json);
    }
}

fn sandbox_block_icon(id: enzo_core::map::BlockId) -> Option<String> {
    use enzo_core::map::BlockId;
    let path = match id {
        BlockId::Ground => "assets/blocks/ground/top.png",
        BlockId::Block => "assets/blocks/block/default.png",
        BlockId::Platform => "assets/blocks/platform/default.png",
        BlockId::Crate => "assets/blocks/crate/intact.png",
        BlockId::BreakableStone => "assets/blocks/breakable_stone/intact.png",
        BlockId::FlagBottom => "assets/blocks/flag_bottom/default.png",
        BlockId::FlagTop => "assets/blocks/flag_top/default.png",
        // Invisible block has no visible texture; render as labeled slot in UI.
        BlockId::InvisibleBlock => return None,
        BlockId::Empty => return None,
    };
    Some(path.to_owned())
}

fn sandbox_block_options() -> Vec<UiSandboxBlockOption> {
    use enzo_core::map::BlockId;
    let ids = [
        BlockId::Ground,
        BlockId::Block,
        BlockId::Platform,
        BlockId::Crate,
        BlockId::BreakableStone,
        BlockId::InvisibleBlock,
        BlockId::FlagBottom,
        BlockId::FlagTop,
    ];
    let mut out = Vec::with_capacity(ids.len() + 1);
    // Special remove tool at top.
    out.push(UiSandboxBlockOption {
        code: 255,
        name: "Remove".to_owned(),
        icon: None,
        remove: true,
    });
    for id in ids {
        out.push(UiSandboxBlockOption {
            code: id as u8,
            name: enzo_core::map::Tile::new(id).def().name.to_owned(),
            icon: sandbox_block_icon(id),
            remove: false,
        });
    }
    out
}

pub fn default_sandbox_ui_state() -> UiSandboxState {
    let enemies = vec![UiSandboxEnemyOption {
        code: 0,
        kind: "slime".to_owned(),
        name: "Slime".to_owned(),
        sprite: "assets/entities/slime/sheet.png".to_owned(),
        frame_x: 0,
        frame_y: 0,
        frame_w: 32,
        frame_h: 32,
        sheet_w: 192,
        sheet_h: 32,
    }];
    let items = enzo_core::entities::ItemKind::ALL
        .iter()
        .map(|k| UiSandboxItemOption {
            code: k.code(),
            name: format!("{:?}", k),
            icon: k.icon_path().to_owned(),
        })
        .collect();
    UiSandboxState {
        active: true,
        tab: 0,
        block_mode: 0,
        selected_block: enzo_core::map::BlockId::Block as u8,
        selected_enemy: 0,
        selected_item: enzo_core::entities::ItemKind::Sword.code(),
        blocks: sandbox_block_options(),
        enemies,
        items,
    }
}

fn control_bindings(bindings: &KeyBindings, i18n: I18n) -> Vec<UiControlBinding> {
    let is_en = i18n.language().code() == "en";
    let mut out = Vec::new();
    for action in InputAction::ALL {
        // Attribute allocation is performed via the in-game UI, not keyboard buttons.
        // Aura farming (Six/Seven alternation) is a fixed mechanic and must not be rebound.
        // Restart is not configurable in the controls UI.
        if action.attribute().is_some()
            || matches!(
                action,
                InputAction::Six | InputAction::Seven | InputAction::Restart
            )
        {
            continue;
        }
        let keys: Vec<String> = bindings
            .keys(action)
            .iter()
            .map(|k| key_label(*k))
            .collect();
        out.push(UiControlBinding {
            action: format!("{:?}", action),
            action_code: action.code(),
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
        (InputAction::Slot1, true) => "Action slot 1",
        (InputAction::Slot1, false) => "Slot de ação 1",
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

/// Apply menu-only UI actions. Returns menu-level action if produced.
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
        | UiAction::MenuOpenControls
        | UiAction::MenuOpenSandbox
        | UiAction::SandboxUnlockDigit { .. }
        | UiAction::ControlsResetDefaults
        | UiAction::ControlsSetKey { .. }
        | UiAction::ControlsClearAction { .. }
        | UiAction::SetPlayableCharacter { .. }
        | UiAction::SetVisualProfileField { .. }
        | UiAction::MenuStart { .. }
        | UiAction::MenuResume
        | UiAction::MenuRequestDelete { .. }
        | UiAction::MenuCancelDelete
        | UiAction::MenuConfirmDelete
        | UiAction::SetLanguage { .. }
        | UiAction::SetMobileControls { .. }
        | UiAction::SetInterfaceScale { .. }
        | UiAction::SetInterfaceScaleAuto { .. } => menu.apply_ui_action(action, i18n),
        _ => None,
    }
}

pub fn attribute_from_code(code: u8) -> Option<Attribute> {
    Attribute::ALL.get(code as usize).copied()
}
