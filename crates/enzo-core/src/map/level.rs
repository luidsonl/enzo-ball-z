//! Level grid, collision queries, dynamic block state, and map metadata.

use crate::events::{BlockPos, GameEvent};
use crate::math::{Rect, Vec2};

use super::block::BlockId;
use super::format::{parse_map_text, EnemySpawnDef, Legend, LegendTarget, MapMetadata};
use super::parse::LevelError;
use super::tile::Tile;

pub const TILE_SIZE: f32 = 32.0;

#[derive(Clone, Debug, PartialEq)]
pub struct Level {
    width: usize,
    height: usize,
    tiles: Vec<Tile>,
    damage: Vec<u16>,
    pub metadata: MapMetadata,
    pub spawn_tiles: Vec<(usize, usize)>,
    pub enemy_spawns: Vec<EnemySpawnDef>,
    pub flag: Option<Rect>,
}

impl Level {
    /// Parse an ASCII map. Supports the new metadata/legend/grid format and the legacy plain grid.
    pub fn parse(source: &str) -> Result<Self, LevelError> {
        let parsed = parse_map_text(source);
        let grid = parsed.grid.trim_end_matches(['\n', '\r']);
        if grid.trim().is_empty() {
            return Err(LevelError::Empty);
        }

        let lines: Vec<&str> = grid.lines().collect();
        let width = lines[0].chars().count();
        if width == 0 {
            return Err(LevelError::Empty);
        }

        let height = lines.len();
        let mut tiles = vec![Tile::EMPTY; width * height];
        let mut spawn_tiles = Vec::new();
        let mut enemy_spawns = Vec::new();
        let mut flag = None;
        let legend = if parsed.used_new_format {
            parsed.legend
        } else {
            Legend::defaults()
        };

        for (row, line) in lines.iter().enumerate() {
            let found = line.chars().count();
            if found != width {
                return Err(LevelError::InconsistentWidth {
                    row,
                    expected: width,
                    found,
                });
            }

            for (col, symbol) in line.chars().enumerate() {
                let Some(target) = legend.get(symbol).cloned() else {
                    return Err(LevelError::UnknownTile { row, col, symbol });
                };
                let tile = match target {
                    LegendTarget::Block(id) => Tile::new(id),
                    LegendTarget::PlayerSpawn => {
                        spawn_tiles.push((col, row));
                        Tile::EMPTY
                    }
                    LegendTarget::Enemy(kind) => {
                        enemy_spawns.push(EnemySpawnDef {
                            tile: (col, row),
                            kind,
                        });
                        Tile::EMPTY
                    }
                    LegendTarget::Flag => {
                        // 2-tile-tall flag: top + bottom blocks (2 parts).
                        // Bottom is placed at the marker row; top goes one row above when safe.
                        let has_top = row > 0
                            && !spawn_tiles.contains(&(col, row - 1))
                            && !enemy_spawns.iter().any(|e| e.tile == (col, row - 1))
                            && tiles[(row - 1) * width + col] == Tile::EMPTY;
                        if has_top {
                            tiles[(row - 1) * width + col] = Tile::FLAG_TOP;
                        }
                        // Trigger rect covers the flag's visual tiles for victory detection.
                        if flag.is_none() {
                            let top_row = if has_top { row - 1 } else { row };
                            flag = Some(Rect::new(
                                col as f32 * TILE_SIZE + 8.0,
                                top_row as f32 * TILE_SIZE,
                                16.0,
                                64.0,
                            ));
                        }
                        Tile::FLAG_BOTTOM
                    }
                };
                tiles[row * width + col] = tile;
            }
        }

        if spawn_tiles.is_empty() {
            return Err(LevelError::NoSpawn);
        }

        Ok(Self {
            width,
            height,
            damage: vec![0; tiles.len()],
            tiles,
            metadata: parsed.metadata,
            spawn_tiles,
            enemy_spawns,
            flag,
        })
    }

    /// Builds a level from an explicit block-id grid (supports map editor / block-id incorporation).
    /// `cells` is row-major. Missing cells default to empty; extra cells are ignored.
    pub fn from_block_ids(
        width: usize,
        height: usize,
        cells: Vec<BlockId>,
        spawn_tiles: Vec<(usize, usize)>,
        enemy_tiles: Vec<(usize, usize)>,
        flag: Option<Rect>,
    ) -> Result<Self, LevelError> {
        Self::from_block_ids_with_meta(
            width,
            height,
            cells,
            spawn_tiles,
            enemy_tiles
                .into_iter()
                .map(|tile| EnemySpawnDef {
                    tile,
                    kind: "slime".to_owned(),
                })
                .collect(),
            flag,
            MapMetadata::default(),
        )
    }

    pub fn from_block_ids_with_meta(
        width: usize,
        height: usize,
        cells: Vec<BlockId>,
        spawn_tiles: Vec<(usize, usize)>,
        enemy_spawns: Vec<EnemySpawnDef>,
        flag: Option<Rect>,
        metadata: MapMetadata,
    ) -> Result<Self, LevelError> {
        if width == 0 || height == 0 {
            return Err(LevelError::Empty);
        }
        let mut tiles = vec![Tile::EMPTY; width * height];
        for (i, id) in cells.into_iter().take(tiles.len()).enumerate() {
            tiles[i] = Tile::new(id);
        }
        let in_bounds = |&(x, y): &(usize, usize)| x < width && y < height;
        let spawn_tiles: Vec<_> = spawn_tiles.into_iter().filter(in_bounds).collect();
        if spawn_tiles.is_empty() {
            return Err(LevelError::NoSpawn);
        }
        let enemy_spawns: Vec<_> = enemy_spawns
            .into_iter()
            .filter(|e| in_bounds(&e.tile))
            .collect();
        let flag = flag.filter(|rect| {
            rect.w > 0.0
                && rect.h > 0.0
                && rect.left() < width as f32 * TILE_SIZE
                && rect.top() < height as f32 * TILE_SIZE
                && rect.right() > 0.0
                && rect.bottom() > 0.0
        });
        Ok(Self {
            width,
            height,
            damage: vec![0; tiles.len()],
            tiles,
            metadata,
            spawn_tiles,
            enemy_spawns,
            flag,
        })
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn width_px(&self) -> f32 {
        self.width as f32 * TILE_SIZE
    }

    pub fn height_px(&self) -> f32 {
        self.height as f32 * TILE_SIZE
    }

    fn index(&self, tx: usize, ty: usize) -> Option<usize> {
        if tx >= self.width || ty >= self.height {
            None
        } else {
            Some(ty * self.width + tx)
        }
    }

    pub fn base_tile(&self, tx: usize, ty: usize) -> Tile {
        self.index(tx, ty)
            .map(|i| self.tiles[i])
            .unwrap_or(Tile::EMPTY)
    }

    pub fn block_damage(&self, tx: usize, ty: usize) -> u16 {
        self.index(tx, ty).map(|i| self.damage[i]).unwrap_or(0)
    }

    fn is_destroyed(&self, tx: usize, ty: usize) -> bool {
        let Some(index) = self.index(tx, ty) else {
            return false;
        };
        let tile = self.tiles[index];
        let Some(max_hits) = tile.def().max_hits() else {
            return false;
        };
        self.damage[index] >= max_hits
    }

    pub fn tile(&self, tx: usize, ty: usize) -> Tile {
        if self.is_destroyed(tx, ty) {
            Tile::EMPTY
        } else {
            self.base_tile(tx, ty)
        }
    }

    /// Out of bounds counts as empty: the player can fall and die.
    pub fn tile_at(&self, tx: i32, ty: i32) -> Tile {
        if tx < 0 || ty < 0 {
            return Tile::EMPTY;
        }
        self.tile(tx as usize, ty as usize)
    }

    pub fn tile_at_pos(&self, position: Vec2) -> Tile {
        self.tile_at(
            (position.x / TILE_SIZE).floor() as i32,
            (position.y / TILE_SIZE).floor() as i32,
        )
    }

    pub fn solid_at_pos(&self, position: Vec2) -> bool {
        self.tile_at_pos(position).is_solid()
    }

    pub fn standable_at_pos(&self, position: Vec2) -> bool {
        self.tile_at_pos(position).is_standable()
    }

    pub fn rect_overlaps_solid(&self, rect: Rect) -> bool {
        for_each_tile_in_rect(rect, |tx, ty| self.tile_at(tx, ty).is_solid())
    }

    pub fn rect_overlaps_platform(&self, rect: Rect) -> bool {
        for_each_tile_in_rect(rect, |tx, ty| self.tile_at(tx, ty).is_platform())
    }

    /// Damages destructible blocks overlapping `rect`. Returns true if any solid
    /// collision remains relevant for blocking the source.
    pub fn damage_solid_blocks_in_rect(
        &mut self,
        rect: Rect,
        amount: u16,
        events: &mut Vec<GameEvent>,
    ) -> bool {
        if amount == 0 {
            return self.rect_overlaps_solid(rect);
        }
        let mut blocked = false;
        for_each_tile_in_rect_mut(rect, |tx, ty| {
            let tile = self.tile_at(tx, ty);
            if !tile.is_solid() {
                return;
            }
            let (x, y) = (tx as usize, ty as usize);
            let Some(index) = self.index(x, y) else {
                blocked = true;
                return;
            };
            let base = self.tiles[index];
            if let Some(max_hits) = base.def().max_hits() {
                let before = self.damage[index];
                if before >= max_hits {
                    return;
                }
                let after = (before + amount).min(max_hits);
                self.damage[index] = after;
                let pos = BlockPos { x, y };
                if after >= max_hits {
                    events.push(GameEvent::BlockDestroyed { pos });
                } else {
                    events.push(GameEvent::BlockDamaged {
                        pos,
                        remaining_hits: max_hits - after,
                    });
                }
                blocked = true;
            } else {
                blocked = true;
            }
        });
        blocked
    }

    pub fn tiles(&self) -> impl Iterator<Item = (usize, usize, Tile)> + '_ {
        self.tiles.iter().copied().enumerate().map(|(index, tile)| {
            let (x, y) = (index % self.width, index / self.width);
            (
                x,
                y,
                if self.is_destroyed(x, y) {
                    Tile::EMPTY
                } else {
                    tile
                },
            )
        })
    }

    pub fn spawn_points(&self, size: Vec2) -> Vec<Vec2> {
        self.spawn_tiles
            .iter()
            .map(|&(tx, ty)| tile_anchor(tx, ty, size))
            .collect()
    }

    /// Backward-compatible: enemy spawn tile positions (no type info).
    pub fn enemy_tiles(&self) -> Vec<(usize, usize)> {
        self.enemy_spawns.iter().map(|e| e.tile).collect()
    }

    pub fn enemy_points(&self, size: Vec2) -> Vec<Vec2> {
        self.enemy_spawns
            .iter()
            .map(|e| tile_anchor(e.tile.0, e.tile.1, size))
            .collect()
    }

    pub fn flag_rect(&self) -> Option<Rect> {
        self.flag
    }
}

fn tile_anchor(tx: usize, ty: usize, size: Vec2) -> Vec2 {
    Vec2::new(
        tx as f32 * TILE_SIZE + (TILE_SIZE - size.x) * 0.5,
        (ty as f32 + 1.0) * TILE_SIZE - size.y,
    )
}

fn for_each_tile_in_rect(rect: Rect, mut predicate: impl FnMut(i32, i32) -> bool) -> bool {
    if rect.w <= 0.0 || rect.h <= 0.0 {
        return false;
    }
    let tx0 = (rect.left() / TILE_SIZE).floor() as i32;
    let tx1 = ((rect.right() - 0.001) / TILE_SIZE).floor() as i32;
    let ty0 = (rect.top() / TILE_SIZE).floor() as i32;
    let ty1 = ((rect.bottom() - 0.001) / TILE_SIZE).floor() as i32;
    for ty in ty0..=ty1 {
        for tx in tx0..=tx1 {
            if predicate(tx, ty) {
                return true;
            }
        }
    }
    false
}

fn for_each_tile_in_rect_mut(rect: Rect, mut visit: impl FnMut(i32, i32)) {
    if rect.w <= 0.0 || rect.h <= 0.0 {
        return;
    }
    let tx0 = (rect.left() / TILE_SIZE).floor() as i32;
    let tx1 = ((rect.right() - 0.001) / TILE_SIZE).floor() as i32;
    let ty0 = (rect.top() / TILE_SIZE).floor() as i32;
    let ty1 = ((rect.bottom() - 0.001) / TILE_SIZE).floor() as i32;
    for ty in ty0..=ty1 {
        for tx in tx0..=tx1 {
            visit(tx, ty);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::GameEvent;
    use crate::map::block::{BlockId, SpecialKind};
    use crate::map::blocks::block_def_by_symbol;
    use crate::map::parse::{symbol_action, SymbolAction};

    const SAMPLE: &str = "\
......
..P.E.
..F...
######
";

    #[test]
    fn parses_tiles_and_markers() {
        let level = Level::parse(SAMPLE).unwrap();
        assert_eq!(level.width(), 6);
        assert_eq!(level.height(), 4);
        assert_eq!(level.tile(0, 3), Tile::GROUND);
        assert_eq!(level.tile(2, 1), Tile::EMPTY); // player spawn remains empty
        assert_eq!(level.tile(2, 2), Tile::FLAG_BOTTOM);
        assert_eq!(level.spawn_tiles, vec![(2, 1)]);
        assert_eq!(level.enemy_tiles(), vec![(4, 1)]);
        assert!(level.flag.is_some());
        assert_eq!(level.width_px(), 6.0 * TILE_SIZE);
    }

    #[test]
    fn block_registry_maps_core_symbols() {
        assert_eq!(block_def_by_symbol('#').unwrap().id, BlockId::Ground);
        assert_eq!(block_def_by_symbol('B').unwrap().id, BlockId::Block);
        assert_eq!(block_def_by_symbol('=').unwrap().id, BlockId::Platform);
        assert_eq!(block_def_by_symbol('.').unwrap().id, BlockId::Empty);
        assert_eq!(block_def_by_symbol('D').unwrap().id, BlockId::Crate);
        assert_eq!(
            block_def_by_symbol('S').unwrap().id,
            BlockId::BreakableStone
        );
        assert_eq!(symbol_action('P'), Some(SymbolAction::PlayerSpawn));
        assert_eq!(symbol_action('E'), Some(SymbolAction::EnemySpawn));
        assert_eq!(symbol_action('F'), Some(SymbolAction::Flag));
        assert_eq!(symbol_action('?'), None);
    }

    #[test]
    fn breakable_stone_is_destructible_solid() {
        assert!(Tile::BREAKABLE_STONE.is_solid());
        assert_eq!(Tile::BREAKABLE_STONE.def().max_hits(), Some(3));
        assert!(matches!(
            Tile::BREAKABLE_STONE.def().special,
            SpecialKind::Destructible
        ));
        let level = Level::parse(
            "P..
S..
###
",
        )
        .unwrap();
        assert_eq!(level.tile(0, 1), Tile::BREAKABLE_STONE);
    }

    #[test]
    fn legend_can_reference_breakable_stone_and_enemy() {
        let src = "name: t

[legend]
. = block:empty
# = block:ground
S = block:breakable_stone
P = marker:player
G = enemy:slime

[map]
P.S
#G#
";
        let level = Level::parse(src).unwrap();
        assert_eq!(level.tile(2, 0), Tile::BREAKABLE_STONE);
        assert_eq!(level.enemy_spawns[0].kind, "slime");
    }

    #[test]
    fn flag_marker_places_two_tile_goal_blocks() {
        let src = "P...\n....\n...F\n####\n";
        let level = Level::parse(src).unwrap();
        assert_eq!(level.tile(3, 2), Tile::FLAG_BOTTOM);
        assert_eq!(level.tile(3, 1), Tile::FLAG_TOP);
        let flag = level.flag_rect().expect("flag rect");
        assert!((flag.y - TILE_SIZE).abs() < f32::EPSILON);
        assert!((flag.h - TILE_SIZE * 2.0).abs() < f32::EPSILON);
    }

    #[test]
    fn block_properties_preserve_core_behavior() {
        assert!(Tile::GROUND.is_solid());
        assert!(Tile::BLOCK.is_solid());
        assert!(!Tile::PLATFORM.is_solid());
        assert!(Tile::PLATFORM.is_platform());
        assert!(Tile::PLATFORM.is_standable());
        assert!(!Tile::EMPTY.is_standable());
        assert!(!Tile::GROUND.is_fluid());
        assert!(!Tile::GROUND.is_climbable());
    }

    #[test]
    fn crate_is_destructible_solid() {
        assert!(Tile::CRATE.is_solid());
        assert_eq!(Tile::CRATE.def().max_hits(), Some(3));
        assert!(matches!(
            Tile::CRATE.def().special,
            SpecialKind::Destructible
        ));
    }

    #[test]
    fn crate_takes_damage_and_breaks() {
        let mut level = Level::parse("P..\nD..\n###\n").unwrap();
        let rect = Rect::new(0.0, TILE_SIZE, TILE_SIZE, TILE_SIZE);
        let mut events = Vec::new();

        assert!(level.tile(0, 1).is_solid());
        assert!(level.damage_solid_blocks_in_rect(rect, 1, &mut events));
        assert_eq!(level.block_damage(0, 1), 1);
        assert!(level.tile(0, 1).is_solid());
        assert!(matches!(
            events.as_slice(),
            [GameEvent::BlockDamaged { .. }]
        ));

        events.clear();
        level.damage_solid_blocks_in_rect(rect, 1, &mut events);
        level.damage_solid_blocks_in_rect(rect, 1, &mut events);
        assert_eq!(level.block_damage(0, 1), 3);
        assert!(!level.tile(0, 1).is_solid());
        assert!(events
            .iter()
            .any(|e| matches!(e, GameEvent::BlockDestroyed { .. })));
    }

    #[test]
    fn spawn_points_anchor_bottom_center() {
        let level = Level::parse(SAMPLE).unwrap();
        let size = Vec2::new(24.0, 30.0);
        let spawn = level.spawn_points(size)[0];
        assert_eq!(spawn.x, 2.0 * TILE_SIZE + 4.0);
        assert_eq!(spawn.y, 2.0 * TILE_SIZE - 30.0);
    }

    #[test]
    fn rejects_inconsistent_width() {
        let error = Level::parse("P..\n###\n##").unwrap_err();
        assert!(matches!(
            error,
            LevelError::InconsistentWidth { row: 2, .. }
        ));
    }

    #[test]
    fn rejects_unknown_tile() {
        let error = Level::parse("P?\n##").unwrap_err();
        assert!(matches!(
            error,
            LevelError::UnknownTile {
                row: 0,
                col: 1,
                symbol: '?'
            }
        ));
    }

    #[test]
    fn rejects_map_without_spawn() {
        assert_eq!(Level::parse("..\n##").unwrap_err(), LevelError::NoSpawn);
    }

    #[test]
    fn out_of_bounds_is_empty() {
        let level = Level::parse(SAMPLE).unwrap();
        assert_eq!(level.tile_at(-1, 0), Tile::EMPTY);
        assert_eq!(level.tile_at(0, 99), Tile::EMPTY);
    }

    #[test]
    fn detects_solid_overlap() {
        let level = Level::parse(SAMPLE).unwrap();
        let touching_ground = Rect::new(0.0, 3.0 * TILE_SIZE, 32.0, 32.0);
        let above_ground = Rect::new(0.0, 2.0 * TILE_SIZE, 32.0, 32.0);
        assert!(level.rect_overlaps_solid(touching_ground));
        assert!(!level.rect_overlaps_solid(above_ground));
    }

    #[test]
    fn parses_level_one() {
        let level = Level::parse(crate::levels::LEVEL_1).unwrap();
        assert_eq!(level.width(), 100);
        assert_eq!(level.height(), 22);
        assert_eq!(level.spawn_tiles.len(), 1);
        assert_eq!(level.enemy_tiles().len(), 6);
        assert!(level.flag.is_some());
    }

    #[test]
    fn block_id_lookup_supports_editor_incorporation() {
        assert_eq!(
            crate::map::block_id_by_name("ground"),
            Some(crate::map::BlockId::Ground)
        );
        assert_eq!(
            crate::map::block_id_by_name("crate"),
            Some(crate::map::BlockId::Crate)
        );
        assert_eq!(crate::map::block_id_by_name("unknown"), None);
    }

    #[test]
    fn level_can_be_built_from_block_ids() {
        let level = Level::from_block_ids(
            3,
            2,
            vec![
                crate::map::BlockId::Empty,
                crate::map::BlockId::Empty,
                crate::map::BlockId::Empty,
                crate::map::BlockId::Ground,
                crate::map::BlockId::Crate,
                crate::map::BlockId::Ground,
            ],
            vec![(0, 0)],
            vec![],
            None,
        )
        .unwrap();
        assert_eq!(level.tile(1, 1), Tile::CRATE);
        assert_eq!(level.width(), 3);
    }

    #[test]
    fn from_block_ids_ignores_out_of_bounds_markers_and_short_cells() {
        let level = Level::from_block_ids(
            2,
            2,
            vec![crate::map::BlockId::Ground], // short -> rest empty
            vec![(0, 0), (9, 9)],
            vec![(1, 1), (8, 8)],
            Some(Rect::new(0.0, 0.0, 16.0, 32.0)),
        )
        .unwrap();
        assert_eq!(level.tile(0, 0), Tile::GROUND);
        assert_eq!(level.tile(1, 1), Tile::EMPTY);
        assert_eq!(level.spawn_tiles, vec![(0, 0)]);
        assert_eq!(level.enemy_tiles(), vec![(1, 1)]);
        assert!(level.flag.is_some());
    }

    #[test]
    fn from_block_ids_requires_in_bounds_spawn() {
        let error = Level::from_block_ids(
            2,
            2,
            vec![crate::map::BlockId::Empty; 4],
            vec![(5, 5)],
            vec![],
            None,
        )
        .unwrap_err();
        assert_eq!(error, LevelError::NoSpawn);
    }

    #[test]
    fn parses_new_format_with_metadata_and_legend() {
        let src = r#"
name: Test Stage
world: grasslands
background: hills
stage: 2

[legend]
. = block:empty
# = block:ground
P = marker:player
S = enemy:slime
F = marker:flag

[map]
.....
.P.S.
..F..
#####
"#;
        let level = Level::parse(src).unwrap();
        assert_eq!(level.metadata.name, "Test Stage");
        assert_eq!(level.metadata.world, "grasslands");
        assert_eq!(level.metadata.background, "hills");
        assert_eq!(level.metadata.stage, 2);
        assert_eq!(level.spawn_tiles, vec![(1, 1)]);
        assert_eq!(level.enemy_spawns.len(), 1);
        assert_eq!(level.enemy_spawns[0].kind, "slime");
        assert!(level.flag.is_some());
        assert_eq!(level.tile(0, 3), Tile::GROUND);
    }
}
