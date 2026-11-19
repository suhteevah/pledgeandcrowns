// SPDX-License-Identifier: MIT
//! Ambient townsfolk: villagers who stand about the dressed plazas and
//! give no mission (REF-173..179, approved by Matt 2026-11-19).
//!
//! They are not in the mission roster (`npc::NPC_ROSTER`): no NPC id, no
//! roster slot, no dialogue, nothing in the save. Placement is data, like
//! the props: [`TOWNSFOLK`] lists each figure's district, kind, offset from
//! the plaza centre and the prop they stand at. A figure is a 32x32
//! centre-anchored sprite like a roster villager (y-sorted with the props
//! and characters, faded props over it, a 1 px idle bob), but it carries
//! no [`Npc`](crate::plugins::npc::Npc) component, so the F key never
//! finds it and no quest marker is hung on it. Like the roster NPCs it is
//! not solid: the player walks past, so it can never block a lane, a
//! street mouth, a talk spot or a restored position.
//!
//! `tests/npc_placement.rs` checks every figure stands on its plaza's
//! paving within a tile of its prop, clear of every prop's sprite and
//! every talk spot, at least `world::CAST_MIN_SPACING` from every roster
//! NPC; `prop.rs` tests keep them out of the street mouths.

use crate::assets::{
    SPRITE_FOLK_BOY, SPRITE_FOLK_ELDER, SPRITE_FOLK_FARMER, SPRITE_FOLK_GIRL, SPRITE_FOLK_GOODWIFE,
    SPRITE_FOLK_GUARD, SPRITE_FOLK_MERCHANT,
};
use crate::plugins::animation::{IdleBob, SpriteLift};
use crate::plugins::prop::PropKind;
use crate::plugins::state::GameState;
use crate::plugins::world::DISTRICTS;
use bevy::prelude::*;

/// Native size of every townsfolk sprite (REF-173..179 are 32x32).
pub const FOLK_PX: f32 = 32.0;

/// Idle-bob phases continue past the roster's, so a figure never bobs in
/// step with the NPC listed at the same index.
const BOB_PHASE_OFFSET: usize = 100;

/// The seven ambient figures (REF-173..179).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Folk {
    Farmer,
    Goodwife,
    Elder,
    Boy,
    Merchant,
    Guard,
    Girl,
}

impl Folk {
    /// Every kind, in ref order (REF-173..179).
    pub const ALL: [Folk; 7] = [
        Folk::Farmer,
        Folk::Goodwife,
        Folk::Elder,
        Folk::Boy,
        Folk::Merchant,
        Folk::Guard,
        Folk::Girl,
    ];

    pub fn sprite_path(self) -> &'static str {
        match self {
            Folk::Farmer => SPRITE_FOLK_FARMER,
            Folk::Goodwife => SPRITE_FOLK_GOODWIFE,
            Folk::Elder => SPRITE_FOLK_ELDER,
            Folk::Boy => SPRITE_FOLK_BOY,
            Folk::Merchant => SPRITE_FOLK_MERCHANT,
            Folk::Guard => SPRITE_FOLK_GUARD,
            Folk::Girl => SPRITE_FOLK_GIRL,
        }
    }
}

/// One ambient figure: where it stands and what it stands at.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Townsperson {
    /// Index into `world::DISTRICTS`.
    pub district: usize,
    pub folk: Folk,
    /// Sprite centre, as an offset from the plaza centre.
    pub offset: Vec2,
    /// The prop this figure is gathered at (a placed prop of this kind in
    /// the same district stands within a tile of the sprite).
    pub at: PropKind,
}

const fn folk(district: usize, folk: Folk, x: f32, y: f32, at: PropKind) -> Townsperson {
    Townsperson {
        district,
        folk,
        offset: Vec2::new(x, y),
        at,
    }
}

/// Every ambient figure, by district (indices into `world::DISTRICTS`).
pub const TOWNSFOLK: &[Townsperson] = &[
    // Hearthstone Square: the farmer in his chicken yard, the elder and a
    // girl by the benches of the seating corner.
    folk(0, Folk::Farmer, 156.0, 204.0, PropKind::ChickenCoop),
    folk(0, Folk::Elder, -184.0, -160.0, PropKind::Bench),
    folk(0, Folk::Girl, -248.0, -160.0, PropKind::Bench),
    // Market Row: the merchant at the second pitch's handcart, the
    // goodwife and her boy by the bench in the south middle.
    folk(1, Folk::Merchant, -160.0, 52.0, PropKind::Handcart),
    folk(1, Folk::Goodwife, 32.0, -152.0, PropKind::Bench),
    folk(1, Folk::Boy, 88.0, -152.0, PropKind::Planter),
    // The Commons: the goodwife and the girl at the commons well, a guard
    // by the street mouth's stores.
    folk(2, Folk::Goodwife, -48.0, -112.0, PropKind::Well),
    folk(2, Folk::Girl, 8.0, -112.0, PropKind::BucketShovel),
    folk(2, Folk::Guard, -200.0, -88.0, PropKind::Barrels),
    // Guildhall Quarter: a merchant at the west bench, the elder at a
    // garden bench.
    folk(3, Folk::Merchant, -152.0, 8.0, PropKind::Bench),
    folk(3, Folk::Elder, 152.0, -140.0, PropKind::Bench),
    // The Mage's Tower: the elder at the south-west bench, a girl at the
    // second reading spot's bench.
    folk(4, Folk::Elder, -168.0, -152.0, PropKind::Bench),
    folk(4, Folk::Girl, 152.0, -128.0, PropKind::Bench),
    // Tavern Row: a farmer and a merchant at the round tables either side
    // of the south street.
    folk(5, Folk::Farmer, -136.0, -160.0, PropKind::RoundTable),
    folk(5, Folk::Merchant, 136.0, -160.0, PropKind::RoundTable),
    // The Forge: a farmer at the delivery handcart, his boy by the
    // weapon rack.
    folk(6, Folk::Farmer, 136.0, -136.0, PropKind::Handcart),
    folk(6, Folk::Boy, 96.0, -104.0, PropKind::WeaponRack),
    // The Vault: two guards flanking the vault door, a merchant at the
    // south-east treasury.
    folk(8, Folk::Guard, -56.0, 148.0, PropKind::VaultDoor),
    folk(8, Folk::Guard, 56.0, 148.0, PropKind::VaultDoor),
    folk(8, Folk::Merchant, 72.0, -152.0, PropKind::CoinPile),
    // The Forbidden Library: the elder at the reading bench, a boy at
    // the east globe.
    folk(9, Folk::Elder, -208.0, -104.0, PropKind::Bench),
    folk(9, Folk::Boy, 208.0, 88.0, PropKind::Globe),
    // The Throne: two guards flanking the runner at the top of the aisle,
    // on the approach to the dais.
    folk(10, Folk::Guard, -32.0, -24.0, PropKind::Column),
    folk(10, Folk::Guard, 32.0, -24.0, PropKind::Column),
];

/// World-space sprite centre of a figure, or `None` for an unknown
/// district.
pub fn townsperson_pos(t: &Townsperson) -> Option<Vec2> {
    DISTRICTS.get(t.district).map(|d| d.center + t.offset)
}

/// Marker for an ambient figure. Not an [`Npc`](crate::plugins::npc::Npc):
/// nothing reads it for talk, markers or saves.
#[derive(Component, Clone, Copy, Debug)]
pub struct Townsfolk {
    pub folk: Folk,
}

pub struct TownsfolkPlugin;

impl Plugin for TownsfolkPlugin {
    fn build(&self, app: &mut App) {
        tracing::debug!("TownsfolkPlugin::build");
        app.add_systems(OnEnter(GameState::InGame), spawn_townsfolk);
    }
}

fn spawn_townsfolk(mut commands: Commands, asset_server: Res<AssetServer>) {
    let mut spawned = 0usize;
    for (i, t) in TOWNSFOLK.iter().enumerate() {
        let Some(pos) = townsperson_pos(t) else {
            tracing::warn!(?t, "townsperson references unknown district; skipped");
            continue;
        };
        tracing::debug!(
            index = i,
            folk = ?t.folk,
            district = t.district,
            x = pos.x,
            y = pos.y,
            at = ?t.at,
            "spawning townsperson"
        );
        commands.spawn((
            Sprite {
                image: asset_server.load(t.folk.sprite_path()),
                custom_size: Some(Vec2::splat(FOLK_PX)),
                ..default()
            },
            Transform::from_xyz(pos.x, pos.y, 5.0),
            Townsfolk { folk: t.folk },
            IdleBob::for_roster_index(BOB_PHASE_OFFSET + i),
            SpriteLift::default(),
        ));
        spawned += 1;
    }
    tracing::info!("spawned {spawned} ambient townsfolk");
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every figure names a real district, every kind is used, and each
    /// kind stands in more than one district.
    #[test]
    fn every_kind_is_placed_in_several_districts() {
        for t in TOWNSFOLK {
            assert!(townsperson_pos(t).is_some(), "{t:?}");
        }
        for f in Folk::ALL {
            let mut ds: Vec<usize> = TOWNSFOLK
                .iter()
                .filter(|t| t.folk == f)
                .map(|t| t.district)
                .collect();
            ds.sort_unstable();
            ds.dedup();
            assert!(ds.len() >= 2, "{f:?} only in {ds:?}");
        }
    }

    /// Two to five figures in each dressed district, none in the Wharf
    /// (not dressed yet).
    #[test]
    fn two_to_five_per_dressed_district() {
        for (d, district) in DISTRICTS.iter().enumerate() {
            let n = TOWNSFOLK.iter().filter(|t| t.district == d).count();
            if district.name == "The Wharf" {
                assert_eq!(n, 0, "the Wharf is not dressed");
            } else {
                assert!((2..=5).contains(&n), "{} has {n}", district.name);
            }
        }
    }
}
