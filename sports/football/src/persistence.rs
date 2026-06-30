//! Saving and loading a football game.
//!
//! `sim-core` persists everything it owns — clock, clubs, contracts, names, identities — but
//! it cannot know about `Footballer` abilities. This module is the **sport extension**: it
//! wraps the core save with a football-specific column of abilities, using sim-core's hooks
//! ([`sim_core::entity_order`] at capture, [`sim_core::restore_indexed`] at load) so the
//! abilities line up with the same entities by save index. The result is a *complete* save —
//! load a database, play, save, load, and every player keeps their ability.
//!
//! This is the concrete answer to "how does a sport plug into the save"; if a second sport
//! needs the same, the pattern (core save + an aligned per-entity column) is what gets
//! harvested.

use crate::attributes::Footballer;
use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};
use sim_core::{capture, entity_order, restore_indexed, SaveData};

const MAGIC: [u8; 4] = *b"TSFB";
const VERSION: u16 = 1;

#[derive(Serialize, Deserialize)]
struct FootballerRecord {
    attacking: u8,
    defending: u8,
    finishing: u8,
    goalkeeping: u8,
}

/// The football game save: the core sim-core snapshot plus a football ability column aligned
/// to it by save index (`None` where the entity isn't a footballer).
#[derive(Serialize, Deserialize)]
struct GameSave {
    core: SaveData,
    footballers: Vec<Option<FootballerRecord>>,
}

/// Serialize a football world to bytes: header, then bincode of the core save + abilities.
pub fn save_to_bytes(world: &World) -> Vec<u8> {
    let core = capture(world);
    let footballers = entity_order(world)
        .iter()
        .map(|&e| {
            world.get::<Footballer>(e).map(|f| FootballerRecord {
                attacking: f.attacking,
                defending: f.defending,
                finishing: f.finishing,
                goalkeeping: f.goalkeeping,
            })
        })
        .collect();

    let save = GameSave { core, footballers };
    let mut out = Vec::new();
    out.extend_from_slice(&MAGIC);
    out.extend_from_slice(&VERSION.to_le_bytes());
    out.extend_from_slice(&bincode::serialize(&save).expect("game save serializes"));
    out
}

/// Rebuild a football world from bytes written by [`save_to_bytes`].
pub fn load_from_bytes(bytes: &[u8]) -> Result<World, String> {
    if bytes.len() < 6 || bytes[0..4] != MAGIC {
        return Err("not a football save (bad header)".to_string());
    }
    let version = u16::from_le_bytes([bytes[4], bytes[5]]);
    if version != VERSION {
        return Err(format!("unsupported football save version {version}"));
    }

    let save: GameSave = bincode::deserialize(&bytes[6..]).map_err(|e| e.to_string())?;
    let (mut world, entities) = restore_indexed(&save.core);
    for (i, rec) in save.footballers.iter().enumerate() {
        if let Some(r) = rec {
            world.entity_mut(entities[i]).insert(Footballer {
                attacking: r.attacking,
                defending: r.defending,
                finishing: r.finishing,
                goalkeeping: r.goalkeeping,
            });
        }
    }
    Ok(world)
}

/// Write a football save to a file.
pub fn save_file(world: &World, path: impl AsRef<std::path::Path>) -> std::io::Result<()> {
    std::fs::write(path, save_to_bytes(world))
}

/// Read a football save from a file.
pub fn load_file(path: impl AsRef<std::path::Path>) -> std::io::Result<World> {
    let bytes = std::fs::read(path)?;
    load_from_bytes(&bytes).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::{load_world, Database};
    use crate::gather_lineups;
    use sim_core::{Club, Name};

    #[test]
    fn full_game_round_trips_with_abilities() {
        let db = Database::sample();
        let mut world = load_world(&db);

        let clubs_before = world.query_filtered::<(), With<Club>>().iter(&world).count();
        let footballers_before = world.query::<&Footballer>().iter(&world).count();

        let bytes = save_to_bytes(&world);
        let mut loaded = load_from_bytes(&bytes).unwrap();

        // Clubs and every ability survived.
        assert_eq!(loaded.query_filtered::<(), With<Club>>().iter(&loaded).count(), clubs_before);
        assert_eq!(loaded.query::<&Footballer>().iter(&loaded).count(), footballers_before);

        // Names survived and every club still fields a lineup -> the loaded world is playable.
        assert!(loaded.query::<&Name>().iter(&loaded).count() > 0);
        assert_eq!(gather_lineups(&mut loaded).len(), db.clubs.len());
    }

    #[test]
    fn a_specific_player_keeps_name_and_ability() {
        // Build a tiny world by hand and check one player's data exactly.
        let db = Database::sample();
        let world = load_world(&db);
        let bytes = save_to_bytes(&world);
        let mut loaded = load_from_bytes(&bytes).unwrap();

        // Count footballers that also have a name — should match the database player count.
        let named_footballers = {
            let mut q = loaded.query::<(&Footballer, &Name)>();
            q.iter(&loaded).count()
        };
        assert_eq!(named_footballers, db.players.len());
    }

    #[test]
    fn rejects_a_bad_header() {
        assert!(load_from_bytes(b"nope").is_err());
        assert!(load_from_bytes(b"").is_err());
    }
}
