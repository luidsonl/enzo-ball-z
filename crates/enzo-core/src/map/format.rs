//! Map text format: metadata, legend, and grid.
//!
//! New format:
//! ```text
//! name: Level 1
//! world: grasslands
//! background: hills
//! stage: 1
//!
//! [legend]
//! . = block:empty
//! # = block:ground
//! E = enemy:slime
//! P = marker:player
//! F = marker:flag
//!
//! [map]
//! <ASCII grid>
//! ```
//!
//! Legacy format (plain grid, no section headers/metadata block) is still accepted.

use std::collections::HashMap;

use super::block::BlockId;
use super::blocks;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MapMetadata {
    pub name: String,
    pub world: String,
    pub background: String,
    pub stage: u32,
}

impl Default for MapMetadata {
    fn default() -> Self {
        Self {
            name: String::new(),
            world: String::new(),
            background: String::new(),
            stage: 1,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EnemySpawnDef {
    pub tile: (usize, usize),
    pub kind: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LegendTarget {
    Block(BlockId),
    PlayerSpawn,
    Enemy(String),
    Flag,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Legend {
    map: HashMap<char, LegendTarget>,
}

impl Legend {
    pub fn insert(&mut self, symbol: char, target: LegendTarget) {
        self.map.insert(symbol, target);
    }

    pub fn get(&self, symbol: char) -> Option<&LegendTarget> {
        self.map.get(&symbol)
    }

    /// Default legend matching the historical symbol set.
    pub fn defaults() -> Self {
        let mut l = Self::default();
        for def in blocks::BLOCK_DEFS {
            if let Some(sym) = def.symbol {
                l.insert(sym, LegendTarget::Block(def.id));
            }
        }
        // Ensure common empties exist.
        l.insert('.', LegendTarget::Block(BlockId::Empty));
        l.insert('P', LegendTarget::PlayerSpawn);
        l.insert('E', LegendTarget::Enemy("slime".to_owned()));
        l.insert('F', LegendTarget::Flag);
        l
    }

    pub fn parse_value(symbol: char, value: &str) -> Option<LegendTarget> {
        let value = value.trim();
        let (kind, id) = value
            .split_once(':')
            .map(|(k, v)| (k.trim(), v.trim()))
            .unwrap_or(("", value));
        match kind {
            "block" => blocks::block_def_by_id(id).map(|d| LegendTarget::Block(d.id)),
            "enemy" => Some(LegendTarget::Enemy(if id.is_empty() {
                "slime".to_owned()
            } else {
                id.to_owned()
            })),
            "marker" | "spawn" => match id {
                "player" | "player_spawn" | "P" => Some(LegendTarget::PlayerSpawn),
                "flag" | "goal" | "F" => Some(LegendTarget::Flag),
                _ => None,
            },
            "" => {
                // Shorthand: block id or legacy marker
                if let Some(d) = blocks::block_def_by_id(value) {
                    Some(LegendTarget::Block(d.id))
                } else {
                    match value {
                        "player" | "P" => Some(LegendTarget::PlayerSpawn),
                        "flag" | "F" => Some(LegendTarget::Flag),
                        _ => Some(LegendTarget::Enemy(value.to_owned())),
                    }
                }
            }
            _ => {
                let _ = symbol;
                None
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParsedMapText {
    pub metadata: MapMetadata,
    pub legend: Legend,
    pub grid: String,
    pub used_new_format: bool,
}

pub fn parse_map_text(source: &str) -> ParsedMapText {
    let source = source.replace("\r\n", "\n");
    let trimmed = source.trim_matches('\n');
    if trimmed.is_empty() {
        return ParsedMapText {
            metadata: MapMetadata::default(),
            legend: Legend::defaults(),
            grid: String::new(),
            used_new_format: false,
        };
    }

    let has_sections = trimmed.lines().any(|l| {
        let t = l.trim();
        t.eq_ignore_ascii_case("[map]")
            || t.eq_ignore_ascii_case("[legend]")
            || t.eq_ignore_ascii_case("[metadata]")
    });
    let has_header = looks_like_metadata_header(trimmed);

    if !has_sections && !has_header {
        return ParsedMapText {
            metadata: MapMetadata::default(),
            legend: Legend::defaults(),
            grid: trimmed.to_owned(),
            used_new_format: false,
        };
    }

    let mut metadata = MapMetadata::default();
    let mut legend = Legend::defaults();
    let mut section = "metadata";
    let mut grid_lines: Vec<&str> = Vec::new();
    let mut saw_structured = false;

    for line in trimmed.lines() {
        let t = line.trim();
        if t.is_empty() {
            if section == "map" {
                // preserve intentional? map rows shouldn't be empty in legacy maps, ignore
            }
            continue;
        }
        if t.starts_with('#') && section != "map" {
            continue;
        }
        let lower = t.to_ascii_lowercase();
        if lower == "[metadata]" {
            section = "metadata";
            saw_structured = true;
            continue;
        }
        if lower == "[legend]" {
            section = "legend";
            saw_structured = true;
            continue;
        }
        if lower == "[map]" || lower == "[grid]" {
            section = "map";
            saw_structured = true;
            continue;
        }
        if lower == "---" {
            // convenience separator: advance metadata->legend->map
            section = match section {
                "metadata" => "legend",
                "legend" => "map",
                _ => "map",
            };
            saw_structured = true;
            continue;
        }

        match section {
            "metadata" => {
                if let Some((k, v)) = t.split_once(':') {
                    let k = k.trim().to_ascii_lowercase();
                    let v = v.trim().to_owned();
                    saw_structured = true;
                    match k.as_str() {
                        "name" | "title" => metadata.name = v,
                        "world" => metadata.world = v,
                        "background" | "bg" => metadata.background = v,
                        "stage" | "level" => metadata.stage = v.parse().unwrap_or(1),
                        _ => {}
                    }
                } else if !saw_structured && looks_like_grid_line(t) {
                    grid_lines.push(line);
                    section = "map";
                }
            }
            "legend" => {
                if let Some((sym_s, val)) = t.split_once('=') {
                    let sym_s = sym_s.trim();
                    let symbol = sym_s.chars().next().unwrap_or('.');
                    if let Some(target) = Legend::parse_value(symbol, val) {
                        legend.insert(symbol, target);
                    }
                } else if looks_like_grid_line(t) {
                    // legend ended implicitly
                    grid_lines.push(line);
                    section = "map";
                }
            }
            "map" => {
                // keep original line content (but not trailing \r)
                grid_lines.push(line.trim_end_matches('\r'));
            }
            _ => {}
        }
    }

    if !saw_structured {
        return ParsedMapText {
            metadata: MapMetadata::default(),
            legend: Legend::defaults(),
            grid: trimmed.to_owned(),
            used_new_format: false,
        };
    }

    ParsedMapText {
        metadata,
        legend,
        grid: grid_lines.join("\n"),
        used_new_format: true,
    }
}

fn looks_like_metadata_header(source: &str) -> bool {
    let mut lines = source
        .lines()
        .filter(|l| !l.trim().is_empty() && !l.trim().starts_with('#'));
    let Some(first) = lines.next() else {
        return false;
    };
    let t = first.trim();
    if let Some((k, _)) = t.split_once(':') {
        matches!(
            k.trim().to_ascii_lowercase().as_str(),
            "name" | "title" | "world" | "background" | "bg" | "stage" | "level"
        )
    } else {
        false
    }
}

fn looks_like_grid_line(line: &str) -> bool {
    !line.is_empty()
        && !line.contains(':')
        && !line.starts_with('[')
        && line.chars().all(|c| c.is_ascii_graphic() || c == ' ')
}
