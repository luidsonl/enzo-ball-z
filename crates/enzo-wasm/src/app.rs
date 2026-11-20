//! Loop principal: menu, sessão de jogo, tick fixo, eventos e desenho.
//!
//! O `app` orquestra duas cenas: o menu inicial e uma sessão de jogo. A
//! simulação continua isolada em [`World`]; o cliente acumula tempo de frame e
//! avança o mundo em passos exatos de `1/60 s`. O render interpola entre o
//! snapshot anterior e o atual.

use enzo_core::entities::EntityId;
use enzo_core::events::GameEvent;
use enzo_core::input::{Attribute, PlayerCommand, TickInput};
use enzo_core::levels::LEVEL_1;
use enzo_core::map::LevelError;
use enzo_core::progression::SaveData;
use enzo_core::snapshot::WorldSnapshot;
use enzo_core::world::World;
use enzo_core::DT;
use macroquad::prelude::*;

use crate::camera::GameCamera;
use crate::convert::to_vec2;
use crate::effects::Effects;
use crate::hud::{Hud, HudAction};
use crate::i18n::{I18n, Text};
use crate::input::InputState;
use crate::menu::{Menu, MenuAction};
use crate::pointer::Pointer;
use crate::render::{self, Background};
use crate::save;
use crate::sprites::Sprites;
use crate::tiles::TileRenderer;
use crate::ui;
use crate::ui_bridge;
use crate::ui_controller;

/// Limite de tempo de frame aceito de uma vez. Evita "espiral da morte" quando
/// a aba fica em segundo plano e o navegador entrega um delta enorme.
const MAX_FRAME_TIME: f32 = 0.2;
/// Ticks simulados no máximo por frame. Se o frame atrasou, o resto do tempo é
/// descartado em vez de tentar recuperar tudo de uma vez e travar ainda mais.
const MAX_TICKS_PER_FRAME: u32 = 5;
const HINT_TIME: f32 = 14.0;
const SAVE_TOAST_TIME: f32 = 1.8;

const AURA: Color = Color::new(0.369, 0.949, 1.0, 1.0);
const GOLD: Color = Color::new(0.965, 0.769, 0.271, 1.0);
const RED: Color = Color::new(1.0, 0.42, 0.37, 1.0);
const PURPLE: Color = Color::new(0.722, 0.545, 1.0, 1.0);
const STEEL: Color = Color::new(0.82, 0.86, 0.94, 1.0);

enum Scene {
    Menu(Menu),
    Game(Box<GameSession>),
    MenuOverGame { menu: Menu, game: Box<GameSession> },
}

enum Transition {
    None,
    Start { slot: usize, save: Option<SaveData> },
    StartSandbox,
    OpenMenu,
    Resume,
}

pub struct App {
    scene: Scene,
    sprites: Sprites,
    i18n: I18n,
    pointer: Pointer,
    ui_publish_accum: f32,
    time: f32,
    bindings: crate::input::KeyBindings,
}

impl App {
    pub async fn new() -> Self {
        let sprites = Sprites::new();
        let bindings = save::load_keybindings();
        let mut menu = Menu::new();
        menu.bindings = bindings.clone();
        let app = Self {
            scene: Scene::Menu(menu),
            sprites,
            i18n: I18n::new(save::load_language()),
            pointer: Pointer::default(),
            ui_publish_accum: 0.0,
            time: 0.0,
            bindings,
        };

        if save::load_interface_scale_auto() {
            ui::set_interface_scale(1.25);
        } else {
            ui::set_interface_scale(save::load_interface_scale());
        }
        ui_bridge::set_mobile_controls_mode(save::load_mobile_controls_mode().bridge_code());

        app
    }

    pub async fn run(&mut self) {
        loop {
            let frame_dt = get_frame_time().clamp(0.0, MAX_FRAME_TIME);
            self.time += frame_dt;
            self.pointer.poll();

            let ui_actions = ui_bridge::drain_ui_actions();
            let had_ui_actions = !ui_actions.is_empty();
            if had_ui_actions {
                self.handle_ui_actions(ui_actions);
            }

            // Safety: prevent black flashes when assets load or draw paths miss.
            clear_background(Color::from_rgba(16, 16, 28, 255));
            let transition = match &mut self.scene {
                Scene::Menu(menu) => {
                    // Keep canvas stable (no black flashing) while DOM menu is shown.
                    draw_menu_canvas_background(self.time);
                    // Secret unlock via keyboard in menu.
                    if is_key_pressed(KeyCode::Key6) || is_key_pressed(KeyCode::Kp6) {
                        menu.handle_sandbox_unlock_digit(6);
                    }
                    if is_key_pressed(KeyCode::Key7) || is_key_pressed(KeyCode::Kp7) {
                        menu.handle_sandbox_unlock_digit(7);
                    }
                    if menu.current() == crate::menu::MenuScreen::Customize {
                        draw_character_preview(
                            &self.sprites,
                            menu.draft_character,
                            menu.draft_visual_profile,
                            self.time,
                        );
                    }
                    match menu.frame(self.pointer, &mut self.i18n, frame_dt) {
                        Some(MenuAction::Start { slot, save }) => Transition::Start { slot, save },
                        Some(MenuAction::StartSandbox) => Transition::StartSandbox,
                        Some(MenuAction::Resume) => Transition::None,
                        None => Transition::None,
                    }
                }
                Scene::Game(session) => {
                    session.i18n = self.i18n;
                    if session.frame(frame_dt, self.pointer, &self.sprites) {
                        Transition::OpenMenu
                    } else {
                        Transition::None
                    }
                }
                Scene::MenuOverGame { menu, game } => {
                    game.i18n = self.i18n;
                    let offscreen = Pointer::at(Vec2::new(-10_000.0, -10_000.0));
                    let _ = game.frame(frame_dt, offscreen, &self.sprites);
                    // Pause overlay on the main game canvas only; preview is drawn on top.
                    draw_pause_overlay();
                    // Preview character in idle while customizing (rendered by WASM).
                    if menu.current() == crate::menu::MenuScreen::Customize {
                        let player = game
                            .current
                            .player()
                            .and_then(|e| e.player.as_ref())
                            .or_else(|| {
                                game.current.entities.iter().find_map(|e| e.player.as_ref())
                            });
                        if let Some(player) = player {
                            draw_character_preview(
                                &self.sprites,
                                player.character,
                                player.visual_profile,
                                self.time,
                            );
                        } else {
                            draw_character_preview(
                                &self.sprites,
                                menu.draft_character,
                                menu.draft_visual_profile,
                                self.time,
                            );
                        }
                    }
                    match menu.frame(self.pointer, &mut self.i18n, frame_dt) {
                        Some(MenuAction::Start { slot, save }) => Transition::Start { slot, save },
                        Some(MenuAction::StartSandbox) => Transition::StartSandbox,
                        Some(MenuAction::Resume) => Transition::Resume,
                        None => Transition::None,
                    }
                }
            };

            self.apply_transition(transition);

            // Publish UI state at a reduced rate to avoid JSON/DOM overhead every frame.
            // Menus are stable; HUD cooldowns/bars update smoothly enough at ~15 Hz.
            self.ui_publish_accum += frame_dt;
            if had_ui_actions || self.ui_publish_accum >= (1.0 / 15.0) {
                self.ui_publish_accum = 0.0;
                match &self.scene {
                    Scene::Menu(menu) => ui_controller::publish(
                        "menu",
                        None,
                        Some(menu),
                        self.i18n,
                        None,
                        Some(&menu.bindings),
                    ),
                    Scene::Game(session) => {
                        let sandbox = if session.is_sandbox {
                            Some(session.sandbox_ui_state())
                        } else {
                            None
                        };
                        ui_controller::publish_with_sandbox(
                            "game",
                            Some(&session.current),
                            None,
                            self.i18n,
                            session.game_toast_text(),
                            Some(&session.input.bindings),
                            sandbox,
                        );
                    }
                    Scene::MenuOverGame { menu, game } => {
                        let sandbox = if game.is_sandbox {
                            Some(game.sandbox_ui_state())
                        } else {
                            None
                        };
                        ui_controller::publish_with_sandbox(
                            "menu_over_game",
                            Some(&game.current),
                            Some(menu),
                            self.i18n,
                            game.game_toast_text(),
                            Some(&game.input.bindings),
                            sandbox,
                        );
                    }
                }
            }

            // Poll lazy asset/rig downloads without blocking rendering (prevents flashes).
            self.sprites.pump();
            next_frame().await;
        }
    }

    fn apply_transition(&mut self, transition: Transition) {
        match transition {
            Transition::None => {}
            Transition::Start { slot, save } => {
                match GameSession::new(
                    slot,
                    save,
                    self.sprites.assets.clone(),
                    self.i18n,
                    self.bindings.clone(),
                ) {
                    Ok(session) => {
                        // New games inherit the character editor draft; loaded saves keep their data.
                        self.scene = Scene::Game(Box::new(session));
                    }
                    Err(err) => {
                        if let Scene::Menu(menu) = &mut self.scene {
                            menu.notify(format!("{}: {err}", self.i18n.t(Text::LevelLoadFailed)));
                        }
                    }
                }
            }
            Transition::StartSandbox => {
                match GameSession::new_sandbox(
                    self.sprites.assets.clone(),
                    self.i18n,
                    self.bindings.clone(),
                ) {
                    Ok(session) => {
                        self.scene = Scene::Game(Box::new(session));
                    }
                    Err(err) => match &mut self.scene {
                        Scene::Menu(menu) => {
                            menu.notify(format!("{}: {err}", self.i18n.t(Text::LevelLoadFailed)));
                        }
                        Scene::MenuOverGame { menu, .. } => {
                            menu.notify(format!("{}: {err}", self.i18n.t(Text::LevelLoadFailed)));
                        }
                        Scene::Game(_) => {}
                    },
                }
            }
            Transition::OpenMenu => {
                if let Scene::Game(mut game) =
                    std::mem::replace(&mut self.scene, Scene::Menu(Menu::new()))
                {
                    self.bindings = game.input.bindings.clone();
                    let mut menu = Menu::new();
                    menu.bindings = self.bindings.clone();
                    menu.has_background_game = true;
                    game.paused = true;
                    self.scene = Scene::MenuOverGame { menu, game };
                } else {
                    let mut menu = Menu::new();
                    menu.bindings = self.bindings.clone();
                    self.scene = Scene::Menu(menu);
                }
            }
            Transition::Resume => {
                if let Scene::MenuOverGame { mut game, menu } =
                    std::mem::replace(&mut self.scene, Scene::Menu(Menu::new()))
                {
                    self.bindings = menu.bindings.clone();
                    game.input.bindings = self.bindings.clone();
                    game.paused = false;
                    game.i18n = self.i18n;
                    self.scene = Scene::Game(game);
                }
            }
        }
    }

    fn apply_controls_action_to(
        bindings: &mut crate::input::KeyBindings,
        action: ui_bridge::UiAction,
    ) -> bool {
        use ui_bridge::UiAction;
        match action {
            UiAction::ControlsResetDefaults => {
                *bindings = crate::input::KeyBindings::default();
                let _ = save::store_keybindings(bindings);
                true
            }
            UiAction::ControlsClearAction { action } => {
                if let Some(input_action) = enzo_core::input::InputAction::from_code(action) {
                    if matches!(
                        input_action,
                        enzo_core::input::InputAction::Six | enzo_core::input::InputAction::Seven
                    ) {
                        return true;
                    }
                    bindings.set_keys(input_action, Vec::new());
                    let _ = save::store_keybindings(bindings);
                }
                true
            }
            UiAction::ControlsSetKey {
                action,
                key_index,
                replace,
            } => {
                if let (Some(input_action), Some((name, _))) = (
                    enzo_core::input::InputAction::from_code(action),
                    crate::input::keycode::supported_key_by_index(key_index),
                ) {
                    if matches!(
                        input_action,
                        enzo_core::input::InputAction::Six | enzo_core::input::InputAction::Seven
                    ) {
                        return true;
                    }
                    if let Some(key) = crate::input::keycode::key_from_name(name) {
                        // A key cannot perform more than one action; an action can have multiple keys.
                        bindings.bind_key_exclusive(input_action, key, replace);
                        // Keep fixed aura farming actions immutable.
                        let defaults = crate::input::KeyBindings::default();
                        bindings.set_keys(
                            enzo_core::input::InputAction::Six,
                            defaults.keys(enzo_core::input::InputAction::Six).to_vec(),
                        );
                        bindings.set_keys(
                            enzo_core::input::InputAction::Seven,
                            defaults.keys(enzo_core::input::InputAction::Seven).to_vec(),
                        );
                        let _ = save::store_keybindings(bindings);
                    }
                }
                true
            }
            _ => false,
        }
    }

    fn handle_ui_actions(&mut self, actions: Vec<ui_bridge::UiAction>) {
        use ui_bridge::UiAction;
        let mut transition = Transition::None;
        for action in actions {
            // Controls config is menu-only (Options > Controls), but may appear while paused over game.
            match &mut self.scene {
                Scene::Menu(menu) => {
                    if Self::apply_controls_action_to(&mut menu.bindings, action) {
                        self.bindings = menu.bindings.clone();
                        continue;
                    }
                }
                Scene::MenuOverGame { menu, game } => {
                    if Self::apply_controls_action_to(&mut menu.bindings, action) {
                        self.bindings = menu.bindings.clone();
                        game.input.bindings = self.bindings.clone();
                        continue;
                    }
                }
                Scene::Game(_) => {}
            }
            match &mut self.scene {
                Scene::Menu(menu) => {
                    if let Some(menu_action) = menu.apply_ui_action(action, &mut self.i18n) {
                        match menu_action {
                            MenuAction::Start { slot, save } => {
                                transition = Transition::Start { slot, save };
                            }
                            MenuAction::StartSandbox => {
                                transition = Transition::StartSandbox;
                            }
                            MenuAction::Resume => {}
                        }
                    }
                }
                Scene::Game(session) => match action {
                    UiAction::HudAllocate { attribute } => {
                        if let Some(attr) = Attribute::ALL.get(attribute as usize).copied() {
                            session.pending_allocate = Some(attr);
                        }
                    }
                    UiAction::HudConfirmLevelUp => session.pending_confirm = true,
                    UiAction::HudRestart => session.pending_restart = true,
                    UiAction::HudOpenMenu => {
                        session.persist(Text::SaveReasonProgress);
                        transition = Transition::OpenMenu;
                    }
                    UiAction::HudOpenSkills => session.hud.open(crate::hud::WindowId::Skills),
                    action if session.handle_sandbox_action(action) => {}
                    UiAction::LoadoutAssign { slot, skill } => {
                        if let (Some(slot), Some(skill)) = (
                            enzo_core::input::ActionSlot::from_index(slot as usize),
                            enzo_core::skills::SkillId::from_code(skill),
                        ) {
                            let _ = session.world.apply_loadout(
                                0,
                                enzo_core::skills::LoadoutRequest::Assign { slot, skill },
                            );
                        }
                    }
                    UiAction::LoadoutClear { slot } => {
                        if let Some(slot) = enzo_core::input::ActionSlot::from_index(slot as usize)
                        {
                            let _ = session.world.apply_loadout(
                                0,
                                enzo_core::skills::LoadoutRequest::Clear { slot },
                            );
                        }
                    }
                    UiAction::EquipInventoryItem { item } => {
                        if let Some(kind) = enzo_core::entities::ItemKind::from_code(item) {
                            let _ = session.world.equip_inventory_item(0, kind);
                        }
                    }
                    UiAction::UnequipSlot { slot } => {
                        if let Some(slot) = enzo_core::entities::EquipSlot::from_code(slot) {
                            let _ = session.world.unequip_slot(0, slot);
                        }
                    }
                    UiAction::SetPlayableCharacter { character } => {
                        if let Some(c) =
                            enzo_core::entities::PlayableCharacter::from_code(character)
                        {
                            let _ = session.world.set_playable_character(0, c);
                            let profile = enzo_core::entities::VisualProfile::default_for(c);
                            let _ = session.world.set_visual_profile(0, profile);
                            if !session.is_sandbox {
                                let _ = save::store_draft_character(c);
                                let _ = save::store_draft_visual_profile(profile);
                            }
                        }
                    }
                    UiAction::SetVisualProfileField { field, value } => {
                        if let Some(entity) = session.world.store().player(0) {
                            if let Some(player) = entity.player() {
                                let profile = player.visual_profile.with_field(field, value);
                                let _ = session.world.set_visual_profile(0, profile);
                                if !session.is_sandbox {
                                    let _ = save::store_draft_visual_profile(profile);
                                }
                            }
                        }
                    }
                    _ => {}
                },
                Scene::MenuOverGame { menu, game } => match action {
                    UiAction::HudAllocate { attribute } => {
                        if let Some(attr) = Attribute::ALL.get(attribute as usize).copied() {
                            game.pending_allocate = Some(attr);
                        }
                    }
                    UiAction::HudConfirmLevelUp => game.pending_confirm = true,
                    UiAction::HudRestart => {
                        game.pending_restart = true;
                        transition = Transition::Resume;
                    }
                    action if game.handle_sandbox_action(action) => {}
                    UiAction::HudOpenSkills => game.hud.open(crate::hud::WindowId::Skills),
                    UiAction::LoadoutAssign { slot, skill } => {
                        if let (Some(slot), Some(skill)) = (
                            enzo_core::input::ActionSlot::from_index(slot as usize),
                            enzo_core::skills::SkillId::from_code(skill),
                        ) {
                            let _ = game.world.apply_loadout(
                                0,
                                enzo_core::skills::LoadoutRequest::Assign { slot, skill },
                            );
                        }
                    }
                    UiAction::LoadoutClear { slot } => {
                        if let Some(slot) = enzo_core::input::ActionSlot::from_index(slot as usize)
                        {
                            let _ = game.world.apply_loadout(
                                0,
                                enzo_core::skills::LoadoutRequest::Clear { slot },
                            );
                        }
                    }
                    UiAction::EquipInventoryItem { item } => {
                        if let Some(kind) = enzo_core::entities::ItemKind::from_code(item) {
                            let _ = game.world.equip_inventory_item(0, kind);
                        }
                    }
                    UiAction::UnequipSlot { slot } => {
                        if let Some(slot) = enzo_core::entities::EquipSlot::from_code(slot) {
                            let _ = game.world.unequip_slot(0, slot);
                        }
                    }
                    UiAction::SetPlayableCharacter { character } => {
                        if let Some(c) =
                            enzo_core::entities::PlayableCharacter::from_code(character)
                        {
                            let _ = game.world.set_playable_character(0, c);
                            let profile = enzo_core::entities::VisualProfile::default_for(c);
                            let _ = game.world.set_visual_profile(0, profile);
                            game.world.write_snapshot(&mut game.current);
                            if !game.is_sandbox {
                                menu.draft_character = c;
                                menu.draft_visual_profile = profile;
                                let _ = save::store_draft_character(c);
                                let _ = save::store_draft_visual_profile(profile);
                            }
                        }
                    }
                    UiAction::SetVisualProfileField { field, value } => {
                        if let Some(entity) = game.world.store().player(0) {
                            if let Some(player) = entity.player() {
                                let profile = player.visual_profile.with_field(field, value);
                                let _ = game.world.set_visual_profile(0, profile);
                                game.world.write_snapshot(&mut game.current);
                                if !game.is_sandbox {
                                    menu.draft_visual_profile = profile;
                                    let _ = save::store_draft_visual_profile(profile);
                                }
                            }
                        }
                    }
                    _ => {
                        if let Some(menu_action) = menu.apply_ui_action(action, &mut self.i18n) {
                            match menu_action {
                                MenuAction::Start { slot, save } => {
                                    transition = Transition::Start { slot, save };
                                }
                                MenuAction::StartSandbox => transition = Transition::StartSandbox,
                                MenuAction::Resume => transition = Transition::Resume,
                            }
                        }
                    }
                },
            }
            if !matches!(transition, Transition::None) {
                break;
            }
        }

        if let Scene::MenuOverGame { game, .. } = &mut self.scene {
            game.i18n = self.i18n;
        }

        self.apply_transition(transition);
    }
}

#[derive(Clone, Copy, Debug)]
struct SandboxToolState {
    tab: u8,        // 0 character, 1 blocks, 2 enemies, 3 items
    block_mode: u8, // 0 place, 1 remove
    selected_block: u8,
    selected_enemy: u8,
    selected_item: u8,
}

impl Default for SandboxToolState {
    fn default() -> Self {
        Self {
            tab: 0,
            block_mode: 0,
            selected_block: enzo_core::map::BlockId::Block as u8,
            selected_enemy: 0,
            selected_item: enzo_core::entities::ItemKind::Sword.code(),
        }
    }
}

struct GameSession {
    slot: usize,
    world: World,
    tiles: TileRenderer,
    background: Background,
    camera: GameCamera,
    effects: Effects,
    input: InputState,
    hud: Hud,
    i18n: I18n,
    previous: WorldSnapshot,
    current: WorldSnapshot,
    last_command: PlayerCommand,
    accumulator: f32,
    hint_timer: f32,
    save_toast: Option<(String, f32)>,
    time: f32,
    /// Ações de UI acumuladas para o próximo tick.
    pending_allocate: Option<Attribute>,
    pending_confirm: bool,
    pending_restart: bool,
    paused: bool,
    is_sandbox: bool,
    sandbox_tool: SandboxToolState,
    assets: crate::assets::AssetStore,
}

impl GameSession {
    fn new(
        slot: usize,
        save: Option<SaveData>,
        assets: crate::assets::AssetStore,
        i18n: I18n,
        bindings: crate::input::KeyBindings,
    ) -> Result<Self, LevelError> {
        let mut world = World::new(LEVEL_1)?;
        if let Some(data) = save {
            world.load_save(&data);
        } else {
            // New game: inherit character editor draft (menu customization).
            let dc = save::load_draft_character();
            let dp = save::load_draft_visual_profile(dc);
            let _ = world.set_playable_character(0, dc);
            let _ = world.set_visual_profile(0, dp);
        }

        Self::from_world(slot, world, assets, i18n, bindings, false)
    }

    fn new_sandbox(
        assets: crate::assets::AssetStore,
        i18n: I18n,
        bindings: crate::input::KeyBindings,
    ) -> Result<Self, LevelError> {
        let mut world = World::new_sandbox(enzo_core::levels::SANDBOX_LEVEL)?;
        // Start with draft character customization in sandbox too.
        let dc = save::load_draft_character();
        let dp = save::load_draft_visual_profile(dc);
        let _ = world.set_playable_character(0, dc);
        let _ = world.set_visual_profile(0, dp);
        Self::from_world(0, world, assets, i18n, bindings, true)
    }

    fn from_world(
        slot: usize,
        world: World,
        assets: crate::assets::AssetStore,
        i18n: I18n,
        bindings: crate::input::KeyBindings,
        is_sandbox: bool,
    ) -> Result<Self, LevelError> {
        let tiles = TileRenderer::new(world.level(), assets.clone());
        let background = Background::from_metadata(&world.level().metadata.background);
        let current = world.snapshot();
        let mut camera = GameCamera::new();
        camera.resize(vec2(screen_width(), screen_height()));
        camera.update(0.0, &current, &current, 0.0);

        Ok(Self {
            slot,
            world,
            tiles,
            background,
            camera,
            effects: Effects::new(),
            input: InputState::with_bindings(bindings),
            hud: Hud::new(),
            i18n,
            previous: current.clone(),
            current,
            last_command: PlayerCommand::none(),
            accumulator: 0.0,
            hint_timer: HINT_TIME,
            save_toast: None,
            time: 0.0,
            pending_allocate: None,
            pending_confirm: false,
            pending_restart: false,
            paused: false,
            is_sandbox,
            sandbox_tool: SandboxToolState::default(),
            assets,
        })
    }

    fn sandbox_ui_state(&self) -> ui_controller::UiSandboxState {
        let mut state = ui_controller::default_sandbox_ui_state();
        state.tab = self.sandbox_tool.tab;
        state.block_mode = self.sandbox_tool.block_mode;
        state.selected_block = self.sandbox_tool.selected_block;
        state.selected_enemy = self.sandbox_tool.selected_enemy;
        state.selected_item = self.sandbox_tool.selected_item;
        state
    }

    fn screen_to_world(&self, screen_x: f32, screen_y: f32) -> Vec2 {
        // Convert screen pixels to world coords using the same camera as rendering.
        let cam = self.camera.camera();
        cam.screen_to_world(vec2(screen_x, screen_y))
    }

    fn handle_sandbox_action(&mut self, action: ui_bridge::UiAction) -> bool {
        if !self.is_sandbox {
            return false;
        }
        use ui_bridge::UiAction;
        match action {
            UiAction::SandboxToggleSigma => {
                let current = self
                    .world
                    .store()
                    .player(0)
                    .and_then(|e| e.player().map(|p| p.sigma_mode))
                    .unwrap_or(false);
                let _ = self.world.sandbox_set_sigma_mode(0, !current);
                true
            }
            UiAction::SandboxGrantLevels { levels } => {
                let _ = self
                    .world
                    .sandbox_debug_grant_levels(0, levels.max(1) as u32);
                true
            }
            UiAction::SandboxSetLevel { level } => {
                let _ = self.world.sandbox_debug_set_level(0, level.max(1) as u32);
                true
            }
            UiAction::SandboxResetAttrs => {
                let _ = self.world.sandbox_debug_reset_attributes(0);
                true
            }
            UiAction::SandboxDie => {
                let _ = self.world.sandbox_debug_die(0);
                true
            }
            UiAction::SandboxReset => {
                let _ = self.world.sandbox_reset();
                // Reset tool selection to defaults but keep current tab.
                self.sandbox_tool.selected_block = enzo_core::map::BlockId::Block as u8;
                self.sandbox_tool.block_mode = 0;
                self.tiles = TileRenderer::new(self.world.level(), self.assets.clone());
                self.background =
                    Background::from_metadata(&self.world.level().metadata.background);
                self.previous = self.world.snapshot();
                self.current = self.previous.clone();
                self.effects.clear();
                self.camera.reset();
                true
            }
            UiAction::SandboxSetTab { tab } => {
                self.sandbox_tool.tab = tab.min(3);
                true
            }
            UiAction::SandboxSetBlockTool { block_mode, block } => {
                self.sandbox_tool.block_mode = if block_mode == 1 { 1 } else { 0 };
                if let Some(b) = block_id_from_code(block) {
                    if b != enzo_core::map::BlockId::Empty {
                        self.sandbox_tool.selected_block = b as u8;
                    }
                }
                self.sandbox_tool.tab = 1;
                true
            }
            UiAction::SandboxSetEnemy { kind } => {
                self.sandbox_tool.selected_enemy = kind;
                self.sandbox_tool.tab = 2;
                true
            }
            UiAction::SandboxSetItem { item } => {
                if enzo_core::entities::ItemKind::from_code(item).is_some() {
                    self.sandbox_tool.selected_item = item;
                }
                self.sandbox_tool.tab = 3;
                true
            }
            UiAction::SandboxApplyAt { x, y } => {
                self.apply_sandbox_tool_at_screen(x as f32, y as f32);
                true
            }
            _ => false,
        }
    }

    fn apply_sandbox_tool_at_screen(&mut self, screen_x: f32, screen_y: f32) {
        let world_pos = self.screen_to_world(screen_x, screen_y);
        match self.sandbox_tool.tab {
            1 => {
                let tx = (world_pos.x / enzo_core::map::TILE_SIZE).floor() as i32;
                let ty = (world_pos.y / enzo_core::map::TILE_SIZE).floor() as i32;
                if self.sandbox_tool.block_mode == 1 {
                    let _ = self.world.sandbox_remove_block(tx, ty);
                } else if let Some(block) = block_id_from_code(self.sandbox_tool.selected_block) {
                    let _ = self.world.sandbox_set_block(tx, ty, block);
                }
            }
            2 => {
                // Spawn selected enemy at world position.
                let kind = "slime";
                let _ = self.world.sandbox_spawn_enemy(
                    kind,
                    enzo_core::math::Vec2::new(world_pos.x, world_pos.y),
                );
            }
            3 => {
                if let Some(item) =
                    enzo_core::entities::ItemKind::from_code(self.sandbox_tool.selected_item)
                {
                    let _ = self.world.sandbox_spawn_item(
                        item,
                        enzo_core::math::Vec2::new(world_pos.x, world_pos.y),
                    );
                }
            }
            _ => {}
        }
    }

    /// Avança e desenha um frame. Retorna `true` quando o jogador pediu para
    /// voltar ao menu.
    fn frame(&mut self, frame_dt: f32, pointer: Pointer, sprites: &Sprites) -> bool {
        self.time += frame_dt;
        self.camera.resize(vec2(screen_width(), screen_height()));
        if !self.paused {
            self.input.poll();
            if self.input.has_activity() || pointer.pressed {
                self.hint_timer = 0.0;
            }
        }
        let mut open_menu = if self.paused {
            false
        } else {
            self.input.take_menu_request()
        };

        if !self.paused {
            self.advance(frame_dt);
        }

        let alpha = (self.accumulator / DT).clamp(0.0, 1.0);
        self.effects.update(frame_dt);
        self.emit_continuous_particles();
        self.camera
            .update(frame_dt, &self.previous, &self.current, alpha);

        self.hint_timer = (self.hint_timer - frame_dt).max(0.0);
        self.tick_toast(frame_dt);

        render::draw_background(&self.background, &self.camera, self.time);
        render::draw_world(&mut render::RenderContext {
            sprites,
            tiles: &mut self.tiles,
            level: self.world.level(),
            previous: &self.previous,
            current: &self.current,
            alpha,
            camera: &self.camera,
            effects: &self.effects,
            time: self.time,
        });

        let toast = self
            .save_toast
            .as_ref()
            .map(|(text, remaining)| (text.as_str(), *remaining));
        let action = self.hud.draw(
            &self.current,
            &self.effects,
            self.i18n,
            pointer,
            self.time,
            self.hint_timer,
            toast,
        );

        if let Some(action) = action {
            match action {
                HudAction::Allocate(attribute) => self.pending_allocate = Some(attribute),
                HudAction::ConfirmLevelUp => self.pending_confirm = true,
                HudAction::Restart => self.pending_restart = true,
                HudAction::OpenMenu => open_menu = true,
            }
        }

        if open_menu {
            self.persist(Text::SaveReasonProgress);
        }
        open_menu
    }

    fn emit_continuous_particles(&mut self) {
        use enzo_core::entities::ActionPose;
        use enzo_core::snapshot::SnapshotKind;

        for entity in &self.current.entities {
            if entity.kind != SnapshotKind::Player {
                continue;
            }
            let center = to_vec2(entity.pos) + to_vec2(entity.size) * 0.5;
            let anim = entity.animation;
            if matches!(anim.action, ActionPose::Shout) {
                self.effects.emit_shout(center, entity.facing);
            }
        }
    }

    /// Consome o tempo acumulado em ticks fixos. O primeiro tick do frame pode
    /// usar edges; os extras só repetem estados contínuos.
    fn advance(&mut self, frame_dt: f32) {
        self.accumulator += frame_dt;

        let ticks = tick_budget(self.accumulator);
        let mut first_tick = true;
        for _ in 0..ticks {
            let command = if first_tick {
                let mut command = self.input.take_command();
                if let Some(attribute) = self.pending_allocate.take() {
                    command.allocate = Some(attribute);
                }
                if self.pending_confirm {
                    command.confirm = true;
                    self.pending_confirm = false;
                }
                if self.pending_restart {
                    command.restart = true;
                    self.pending_restart = false;
                }
                command
            } else {
                self.last_command.without_edges()
            };
            self.last_command = command;
            first_tick = false;

            // Alterna os buffers: `previous` recebe o snapshot antigo e
            // `current` é reescrito no mesmo vetor, sem alocar por tick.
            std::mem::swap(&mut self.previous, &mut self.current);
            let output = self.world.step(TickInput::single(command));
            self.world.write_snapshot(&mut self.current);
            self.handle_events(&output.events);

            self.accumulator -= DT;
        }

        // Frame lento: descarta o backlog restante para não emendar uma fila
        // de ticks e transformar um atraso em travamento.
        if self.accumulator >= DT {
            self.accumulator = 0.0;
        }
    }

    fn handle_events(&mut self, events: &[GameEvent]) {
        use enzo_core::events::SkillRejectReason;
        use enzo_core::skills::SkillId;
        for event in events {
            match *event {
                GameEvent::SkillHit {
                    target,
                    damage: _damage,
                    skill,
                    combo: _combo,
                    ..
                } => {
                    if let Some(pos) = entity_center(&self.current, target) {
                        let color = match skill {
                            SkillId::Melee => AURA,
                            SkillId::Mogging | SkillId::NutellaShot => PURPLE,
                            _ => STEEL,
                        };
                        self.effects.sparks(pos, color, 6);
                    }
                    self.camera.add_shake(if matches!(skill, SkillId::Mogging) {
                        4.0
                    } else {
                        2.0
                    });
                }
                GameEvent::SkillProjectileSpawned { owner, skill, .. } => {
                    if let Some(pos) = entity_center(&self.current, owner) {
                        let label = self.i18n.skill(skill);
                        self.effects.sparks(pos, PURPLE, 10);
                        self.effects.text(pos, label, PURPLE, 12.0);
                    }
                    self.camera.add_shake(1.5);
                }
                GameEvent::SkillProjectileBlocked { owner, .. } => {
                    if let Some(pos) = entity_center(&self.current, owner) {
                        self.effects.sparks(pos, STEEL, 5);
                    }
                    self.camera.add_shake(1.5);
                }
                GameEvent::SkillRejected { player, reason, .. } => {
                    if let Some(pos) = entity_center(&self.current, player) {
                        match reason {
                            SkillRejectReason::NoAura => {
                                self.effects.no_aura(pos, self.i18n.t(Text::EffectNoAura));
                            }
                            SkillRejectReason::Cooldown => {}
                            SkillRejectReason::Unavailable(_) => {
                                self.effects
                                    .text(pos, self.i18n.t(Text::SkillLocked), STEEL, 10.0);
                            }
                            SkillRejectReason::EmptySlot => {}
                        }
                    }
                    self.camera.add_shake(1.0);
                }
                GameEvent::SkillActivated { .. }
                | GameEvent::LoadoutChanged { .. }
                | GameEvent::BlockDamaged { .. }
                | GameEvent::BlockDestroyed { .. } => {}
                GameEvent::PlayerDamaged {
                    player,
                    damage: _damage,
                    ..
                } => {
                    if let Some(pos) = entity_center(&self.current, player) {
                        self.effects.hurt(pos);
                    }
                    self.camera.add_shake(5.0);
                }
                GameEvent::EnemyKilled { enemy, .. } => {
                    if let Some(pos) = entity_center(&self.current, enemy) {
                        self.effects.sparks(pos, GOLD, 14);
                    }
                    self.camera.add_shake(1.5);
                }
                GameEvent::LevelUp { player, level } => {
                    if let Some(pos) = entity_center(&self.current, player) {
                        self.effects.text(
                            pos,
                            format!("{} {level}!", self.i18n.t(Text::EffectLevel)),
                            GOLD,
                            16.0,
                        );
                        self.effects.sparks(pos, GOLD, 18);
                    }
                    self.effects.aura_flash = 0.7;
                    self.camera.add_shake(2.0);
                }
                GameEvent::PlayerDied { player } => {
                    if let Some(pos) = entity_center(&self.current, player) {
                        self.effects
                            .text(pos, self.i18n.t(Text::EffectDeath), RED, 18.0);
                        self.effects.sparks(pos, RED, 16);
                    }
                    self.effects.hurt_flash = 0.7;
                    self.camera.add_shake(9.0);
                    self.persist(Text::SaveReasonDeath);
                }
                GameEvent::Victory { player } => {
                    if let Some(pos) = entity_center(&self.current, player) {
                        self.effects.sparks(pos, GOLD, 24);
                    }
                    self.effects.victory_flash = 1.2;
                    self.camera.add_shake(4.0);
                }
                GameEvent::AttributeAllocated { player, attribute } => {
                    if let Some(pos) = entity_center(&self.current, player) {
                        self.effects.text(
                            pos,
                            format!("+{}", self.i18n.attribute(attribute)),
                            AURA,
                            11.0,
                        );
                    }
                    self.persist(Text::SaveReasonAttribute);
                }
                GameEvent::PhaseRestarted { .. } => {
                    self.camera.reset();
                    self.effects.clear();
                }
                GameEvent::SaveRequested => self.persist(Text::SaveReasonProgress),
            }
        }
    }

    fn persist(&mut self, reason: Text) {
        if self.is_sandbox {
            // Sandbox/debug sessions are isolated and must not write campaign saves.
            return;
        }
        let Some(data) = self.world.save_data() else {
            return;
        };
        let reason = self.i18n.t(reason).to_owned();
        match save::store_slot(self.slot, &data) {
            Ok(()) => self.save_toast = Some((reason, SAVE_TOAST_TIME)),
            Err(err) => {
                warn!("save: {err}");
                self.save_toast = Some((
                    format!("{}: {err}", self.i18n.t(Text::SaveFailed)),
                    SAVE_TOAST_TIME,
                ));
            }
        }
    }

    fn tick_toast(&mut self, frame_dt: f32) {
        let expired = match self.save_toast.as_mut() {
            Some((_, remaining)) => {
                *remaining -= frame_dt;
                *remaining <= 0.0
            }
            None => false,
        };
        if expired {
            self.save_toast = None;
        }
    }

    fn game_toast_text(&self) -> Option<&str> {
        self.save_toast.as_ref().map(|(text, _)| text.as_str())
    }
}

fn draw_pause_overlay() {
    set_default_camera();
    draw_rectangle(
        0.0,
        0.0,
        screen_width(),
        screen_height(),
        Color::from_rgba(8, 10, 22, 140),
    );
}

fn draw_menu_canvas_background(time: f32) {
    // Stable background to avoid black flashing while DOM menu/assets are active.
    set_default_camera();
    let w = screen_width();
    let h = screen_height();
    let top = Color::from_rgba(18, 24, 52, 255);
    let bottom = Color::from_rgba(74, 122, 156, 255);
    let bands = 24;
    for i in 0..bands {
        let t = i as f32 / (bands - 1) as f32;
        let c = Color::new(
            top.r + (bottom.r - top.r) * t,
            top.g + (bottom.g - top.g) * t,
            top.b + (bottom.b - top.b) * t,
            1.0,
        );
        let bh = h / bands as f32;
        draw_rectangle(0.0, i as f32 * bh, w, bh + 1.0, c);
    }
    let pulse = (time * 0.8).sin() * 0.5 + 0.5;
    draw_circle(
        w * 0.82,
        h * 0.18,
        h * 0.05 + pulse * 2.0,
        Color::from_rgba(255, 240, 190, 70),
    );
}

fn draw_character_preview(
    sprites: &Sprites,
    character: enzo_core::entities::PlayableCharacter,
    profile: enzo_core::entities::VisualProfile,
    time: f32,
) {
    set_default_camera();
    let w = screen_width();
    let h = screen_height();
    // Preview panel area (left side) rendered by WASM behind DOM menu.
    let panel = Rect::new(w * 0.05, h * 0.22, w * 0.22, h * 0.42);
    draw_rectangle(
        panel.x,
        panel.y,
        panel.w,
        panel.h,
        Color::from_rgba(10, 14, 30, 170),
    );
    draw_rectangle_lines(
        panel.x,
        panel.y,
        panel.w,
        panel.h,
        2.0,
        Color::from_rgba(94, 242, 255, 120),
    );
    // Ground line + idle character preview (all rig states share aligned frames).
    let ground_y = panel.y + panel.h * 0.82;
    draw_line(
        panel.x + 12.0,
        ground_y,
        panel.right() - 12.0,
        ground_y,
        2.0,
        Color::from_rgba(120, 170, 120, 140),
    );
    let foot = vec2(panel.x + panel.w * 0.5, ground_y);
    let tint = WHITE;
    match character {
        enzo_core::entities::PlayableCharacter::Valentina => {
            let _ =
                sprites.draw_valentina_rig("idle", time, foot, false, tint, profile, false, 0.0, 0);
        }
        _ => {
            let _ = sprites.draw_enzo_rig("idle", time, foot, false, tint, profile, false, 0.0, 0);
        }
    }
}

fn block_id_from_code(code: u8) -> Option<enzo_core::map::BlockId> {
    use enzo_core::map::BlockId;
    Some(match code {
        x if x == BlockId::Empty as u8 => BlockId::Empty,
        x if x == BlockId::Ground as u8 => BlockId::Ground,
        x if x == BlockId::Block as u8 => BlockId::Block,
        x if x == BlockId::Platform as u8 => BlockId::Platform,
        x if x == BlockId::Crate as u8 => BlockId::Crate,
        x if x == BlockId::FlagBottom as u8 => BlockId::FlagBottom,
        x if x == BlockId::FlagTop as u8 => BlockId::FlagTop,
        x if x == BlockId::BreakableStone as u8 => BlockId::BreakableStone,
        x if x == BlockId::InvisibleBlock as u8 => BlockId::InvisibleBlock,
        _ => return None,
    })
}

/// Quantos ticks de simulação cabem no tempo acumulado, respeitando o teto por
/// frame. Fora do laço para poder testar sem navegador.
fn tick_budget(accumulator: f32) -> u32 {
    if accumulator < DT {
        return 0;
    }
    (accumulator / DT).floor().min(MAX_TICKS_PER_FRAME as f32) as u32
}

/// Centro visual de uma entidade no snapshot atual.
fn entity_center(snapshot: &WorldSnapshot, id: EntityId) -> Option<Vec2> {
    snapshot
        .entities
        .iter()
        .find(|entity| entity.id == id)
        .map(|entity| to_vec2(entity.pos + entity.size * 0.5))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tick_budget_waits_for_a_full_step() {
        assert_eq!(tick_budget(0.0), 0);
        assert_eq!(tick_budget(DT * 0.99), 0);
        assert_eq!(tick_budget(DT), 1);
    }

    #[test]
    fn tick_budget_caps_catch_up_ticks() {
        assert_eq!(tick_budget(DT * 3.0), 3);
        assert_eq!(tick_budget(DT * 12.0), MAX_TICKS_PER_FRAME);
        assert_eq!(tick_budget(MAX_FRAME_TIME), MAX_TICKS_PER_FRAME);
    }
}
