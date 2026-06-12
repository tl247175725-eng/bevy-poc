//! A* grid pathfinding — Godot `pathfinding.gd` rules.

use std::collections::{BinaryHeap, HashMap, HashSet};

use bevy::prelude::*;

use crate::terrain::{is_blocked_terrain, terrain_at};
use crate::world_rules::{GRID_HEIGHT, GRID_WIDTH};
use crate::world_state::WorldState;

#[derive(Resource, Default)]
pub struct PathGrid {
    pub revision: u64,
}

#[derive(Clone, Copy, Eq, PartialEq, Hash)]
struct Node {
    x: u8,
    y: u8,
}

#[derive(Clone, Copy, PartialEq)]
struct Scored {
    f: u32,
    g: u32,
    node: Node,
}

impl Eq for Scored {}

impl Ord for Scored {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.f.cmp(&self.f)
    }
}

impl PartialOrd for Scored {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

pub fn find_path(
    world: &WorldState,
    sx: u8,
    sy: u8,
    tx: u8,
    ty: u8,
    exclude: Option<u64>,
) -> Vec<(u8, u8)> {
    if sx == tx && sy == ty {
        return Vec::new();
    }
    if !in_grid(sx, sy) || !in_grid(tx, ty) {
        return Vec::new();
    }
    if is_blocked_for(world, tx, ty, exclude) {
        return Vec::new();
    }

    let start = Node { x: sx, y: sy };
    let goal = Node { x: tx, y: ty };
    let mut open = BinaryHeap::new();
    let mut came_from: HashMap<Node, Node> = HashMap::new();
    let mut g_score: HashMap<Node, u32> = HashMap::new();
    g_score.insert(start, 0);
    open.push(Scored {
        f: heuristic(start, goal),
        g: 0,
        node: start,
    });

    let mut closed = HashSet::new();
    while let Some(current) = open.pop() {
        if current.node == goal {
            return reconstruct(&came_from, current.node);
        }
        if closed.contains(&current.node) {
            continue;
        }
        closed.insert(current.node);

        for (nx, ny) in neighbors(current.node.x, current.node.y) {
            let next = Node { x: nx, y: ny };
            if closed.contains(&next) {
                continue;
            }
            if is_blocked_for(world, nx, ny, exclude) && next != goal && next != start {
                continue;
            }
            let tentative = current.g + 1;
            if tentative < *g_score.get(&next).unwrap_or(&u32::MAX) {
                came_from.insert(next, current.node);
                g_score.insert(next, tentative);
                open.push(Scored {
                    f: tentative + heuristic(next, goal),
                    g: tentative,
                    node: next,
                });
            }
        }
    }
    Vec::new()
}

pub fn is_blocked_for(world: &WorldState, x: u8, y: u8, exclude: Option<u64>) -> bool {
    if !in_grid(x, y) {
        return true;
    }
    if is_blocked_terrain(terrain_at(world, x, y)) {
        return true;
    }
    for id in world.entities_at(x, y) {
        if Some(id.0) == exclude {
            continue;
        }
        if let Some(e) = world.entities.get(&id) {
            if blocks_cell(e) {
                return true;
            }
        }
    }
    false
}

fn blocks_cell(entity: &crate::world_state::Entity) -> bool {
    use crate::axioms::composition::entity_occupies_active_cell;
    use crate::world_rules::card_has_tag;
    if !entity_occupies_active_cell(entity) {
        return false;
    }
    if entity.profile.incorporeal {
        return false;
    }
    if entity.profile.height <= crate::axioms::Height::Low {
        return false;
    }
    true
}

fn in_grid(x: u8, y: u8) -> bool {
    x < GRID_WIDTH && y < GRID_HEIGHT
}

fn neighbors(x: u8, y: u8) -> [(u8, u8); 4] {
    let x = x as i16;
    let y = y as i16;
    let max_x = GRID_WIDTH as i16 - 1;
    let max_y = GRID_HEIGHT as i16 - 1;
    [
        ((x - 1).clamp(0, max_x) as u8, y as u8),
        ((x + 1).clamp(0, max_x) as u8, y as u8),
        (x as u8, (y - 1).clamp(0, max_y) as u8),
        (x as u8, (y + 1).clamp(0, max_y) as u8),
    ]
}

fn heuristic(a: Node, b: Node) -> u32 {
    (a.x.abs_diff(b.x) + a.y.abs_diff(b.y)) as u32
}

fn reconstruct(came_from: &HashMap<Node, Node>, mut current: Node) -> Vec<(u8, u8)> {
    let mut path = vec![(current.x, current.y)];
    while let Some(prev) = came_from.get(&current) {
        current = *prev;
        path.push((current.x, current.y));
    }
    path.pop();
    path.reverse();
    path
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world_state::empty_world;

    #[test]
    fn path_same_cell_empty() {
        let w = empty_world();
        assert!(find_path(&w, 5, 5, 5, 5, None).is_empty());
    }

    #[test]
    fn path_adjacent() {
        let w = empty_world();
        let p = find_path(&w, 5, 5, 6, 5, None);
        assert_eq!(p, vec![(6, 5)]);
    }
}
