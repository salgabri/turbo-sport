//! The club entity — the keystone that unifies team identity with the economy.
//!
//! Until now there were two disconnected notions of "club": a player's [`TeamId`] (used by
//! lineups and the league) and the economy's club entity (used by `Balance`, payroll, and
//! transfers via [`Contract`]`.club`). This module makes them one thing: a **club is an
//! entity** carrying both its `TeamId` and its finances + squad target, and a player's team
//! membership is *derived from their contract*. Sign a player to a different club and, after
//! [`sync_squad_membership`], their `TeamId` follows — so the transfer market actually moves
//! players between teams in a running league.
//!
//! Sport-agnostic: every team sport's clubs are this same model. The sport adds only its own
//! player ability components.

use crate::economy::{Balance, Money, WeeklyIncome};
use crate::entity::{Contract, SquadTarget, TeamId};
use bevy_ecs::prelude::*;
use std::collections::HashMap;

/// Marker: this entity is a club (a team's organisation — identity, finances, squad).
#[derive(Component, Clone, Copy, Debug)]
pub struct Club;

/// Everything a club entity carries. Spawn one with [`ClubBundle::new`].
#[derive(Bundle)]
pub struct ClubBundle {
    pub club: Club,
    pub team: TeamId,
    pub balance: Balance,
    pub income: WeeklyIncome,
    pub squad_target: SquadTarget,
}

impl ClubBundle {
    pub fn new(team_id: u32, balance: Money, weekly_income: Money, squad_target: u32) -> Self {
        Self {
            club: Club,
            team: TeamId(team_id),
            balance: Balance(balance),
            income: WeeklyIncome(weekly_income),
            squad_target: SquadTarget(squad_target),
        }
    }
}

/// Resolves a `TeamId` value to its club entity. Rebuilt from the world with [`index_clubs`]
/// (clubs are created rarely, so rebuilding on demand is cheap).
#[derive(Resource, Default, Clone, Debug)]
pub struct ClubRegistry(pub HashMap<u32, Entity>);

impl ClubRegistry {
    pub fn club_of(&self, team_id: u32) -> Option<Entity> {
        self.0.get(&team_id).copied()
    }
}

/// Scan all club entities and build the `TeamId -> Entity` registry.
pub fn index_clubs(world: &mut World) -> ClubRegistry {
    let mut map = HashMap::new();
    let mut q = world.query_filtered::<(Entity, &TeamId), With<Club>>();
    for (e, t) in q.iter(world) {
        map.insert(t.0, e);
    }
    ClubRegistry(map)
}

/// Make every player's `TeamId` match the club they're contracted to, and remove it from
/// players with no contract (a free agent plays for no team). Call after the transfer market
/// runs so lineup gathering — which groups by `TeamId` — follows the contracts. Club entities
/// keep their own `TeamId` untouched.
pub fn sync_squad_membership(world: &mut World) {
    // Club entity -> its team id.
    let mut club_team: HashMap<Entity, u32> = HashMap::new();
    {
        let mut q = world.query_filtered::<(Entity, &TeamId), With<Club>>();
        for (e, t) in q.iter(world) {
            club_team.insert(e, t.0);
        }
    }

    // Contracted players (non-clubs) -> the team id of their club.
    let mut assign: Vec<(Entity, u32)> = Vec::new();
    {
        let mut q = world.query_filtered::<(Entity, &Contract), Without<Club>>();
        for (e, c) in q.iter(world) {
            if let Some(&team) = club_team.get(&c.club) {
                assign.push((e, team));
            }
        }
    }

    // Non-club players that have a TeamId but no contract -> free agents -> drop the TeamId.
    let mut clear: Vec<Entity> = Vec::new();
    {
        let mut q =
            world.query_filtered::<Entity, (With<TeamId>, Without<Club>, Without<Contract>)>();
        for e in q.iter(world) {
            clear.push(e);
        }
    }

    for (e, team) in assign {
        world.entity_mut(e).insert(TeamId(team));
    }
    for e in clear {
        world.entity_mut(e).remove::<TeamId>();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::{FreeAgent, WageDemand};
    use crate::time::{Date, SimClock};
    use crate::transfers::run_transfer_window;

    #[test]
    fn index_maps_team_ids_to_their_entities() {
        let mut world = World::new();
        let c0 = world.spawn(ClubBundle::new(0, 1_000, 100, 20)).id();
        let c7 = world.spawn(ClubBundle::new(7, 1_000, 100, 20)).id();

        let reg = index_clubs(&mut world);
        assert_eq!(reg.club_of(0), Some(c0));
        assert_eq!(reg.club_of(7), Some(c7));
        assert_eq!(reg.club_of(99), None);
    }

    #[test]
    fn membership_follows_the_contract_and_clears_for_free_agents() {
        let mut world = World::new();
        let c0 = world.spawn(ClubBundle::new(0, 1_000, 100, 20)).id();
        let c1 = world.spawn(ClubBundle::new(1, 1_000, 100, 20)).id();
        let player = world.spawn(Contract { club: c0, until: Date::new(2030, 1, 1), wage: 50 }).id();

        sync_squad_membership(&mut world);
        assert_eq!(world.entity(player).get::<TeamId>().copied(), Some(TeamId(0)));

        // Transfer: re-point the contract at club 1.
        world.entity_mut(player).insert(Contract { club: c1, until: Date::new(2030, 1, 1), wage: 50 });
        sync_squad_membership(&mut world);
        assert_eq!(world.entity(player).get::<TeamId>().copied(), Some(TeamId(1)));

        // Release: no contract -> no team.
        world.entity_mut(player).remove::<Contract>();
        sync_squad_membership(&mut world);
        assert!(world.entity(player).get::<TeamId>().is_none());

        // Clubs keep their own identity throughout.
        assert_eq!(world.entity(c0).get::<TeamId>().copied(), Some(TeamId(0)));
    }

    #[test]
    fn the_full_keystone_signed_free_agents_join_the_club_squad() {
        // Transfer market + club model + sync compose: free agents get signed to a club and
        // then appear in that club's squad (by TeamId).
        let mut world = World::new();
        world.insert_resource(SimClock::starting_on(Date::new(2026, 6, 1)));
        world.spawn(ClubBundle::new(0, 100_000, 10_000, 3));
        world.spawn((FreeAgent, WageDemand(100)));
        world.spawn((FreeAgent, WageDemand(100)));

        run_transfer_window(&mut world);
        sync_squad_membership(&mut world);

        let in_squad = world
            .query_filtered::<&TeamId, Without<Club>>()
            .iter(&world)
            .filter(|t| t.0 == 0)
            .count();
        assert_eq!(in_squad, 2, "both signings should now be in club 0's squad");
    }
}
