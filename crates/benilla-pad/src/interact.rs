//! The pad's Interact: the right-click context action with no cursor. benilla runs an object's
//! own interact dispatcher for a crate on top (`benilla_app::Interact`) and lists the streamed
//! objects (`benilla_app::Objects`); what to interact with is decided here.
//!
//! With a target, it is the target. With none, a soft target: a corpse to loot or skin (1.12
//! clears the selection when the target dies), else the nearest NPC with a service or usable
//! object ahead of the camera.

use benilla_app::{Interact, ObjectFields, ObjectType, Objects};
use benilla_world::view::WorldCamera;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

/// How far a soft target may be, in yards: corpses, then NPCs and objects (whose use still needs
/// benilla's own reach).
const CORPSE_RANGE: f32 = 10.0;
const RANGE: f32 = 6.0;
/// `UNIT_FLAG_SKINNABLE`.
const UNIT_FLAG_SKINNABLE: u32 = 0x0400_0000;
/// The GameObject types a soft Interact uses: quest giver, chest (herbs and veins too), goober,
/// text, mailbox.
const GO_TYPES: [i32; 5] = [2, 3, 10, 9, 19];
/// `GAMEOBJECT_FLAGS` bit `0x10`: no interact.
const GO_FLAG_NO_INTERACT: u32 = 0x10;

/// What the pad's Interact reads and writes.
#[derive(SystemParam)]
pub struct Interacting<'w, 's> {
    objects: Objects<'w, 's>,
    transforms: Query<'w, 's, &'static Transform>,
    camera: Query<'w, 's, &'static GlobalTransform, With<WorldCamera>>,
    out: MessageWriter<'w, Interact>,
    me: Local<'s, Option<(Entity, u64)>>,
}

/// One candidate, reduced to what the ranking reads.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Candidate {
    pub guid: u64,
    pub kind: Kind,
    /// Flat offset from the player, yards.
    pub offset: Vec2,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// A dead unit with loot or a hide.
    Corpse,
    /// A live unit with an NPC service, or a usable GameObject.
    Service,
}

/// What an object is to a soft Interact, if anything.
pub fn kind_of(fields: &ObjectFields) -> Option<Kind> {
    match fields.object_type()? {
        ObjectType::Unit | ObjectType::Player => {
            if fields.unit_is_dead() {
                (fields.unit_lootable() || fields.unit_flags() & UNIT_FLAG_SKINNABLE != 0)
                    .then_some(Kind::Corpse)
            } else {
                (fields.unit_npc_flags() != 0).then_some(Kind::Service)
            }
        }
        ObjectType::GameObject => (GO_TYPES.contains(&fields.gameobject_type_id())
            && fields.gameobject_flags() & GO_FLAG_NO_INTERACT == 0)
            .then_some(Kind::Service),
        _ => None,
    }
}

/// The soft target among `candidates`: a corpse within [`CORPSE_RANGE`] (`last`, the last target,
/// first, else the nearest), else the service within [`RANGE`] that is nearest and most ahead of
/// the camera (`ahead`, flat and unit, or zero for no preference); nothing behind the camera.
pub fn soft_target(
    candidates: impl Iterator<Item = Candidate>,
    ahead: Vec2,
    last: Option<u64>,
) -> Option<u64> {
    let mut corpse: Option<(f32, u64)> = None;
    let mut service: Option<(f32, u64)> = None;
    for c in candidates {
        let d = c.offset.length();
        match c.kind {
            Kind::Corpse if d <= CORPSE_RANGE => {
                let rank = if last == Some(c.guid) { -1.0 } else { d };
                if corpse.is_none_or(|(r, _)| rank < r) {
                    corpse = Some((rank, c.guid));
                }
            }
            Kind::Service if d <= RANGE => {
                let facing = if ahead == Vec2::ZERO || d < 0.5 {
                    1.0
                } else {
                    c.offset.normalize().dot(ahead)
                };
                if facing <= 0.0 {
                    continue;
                }
                let rank = d * (2.0 - facing);
                if service.is_none_or(|(r, _)| rank < r) {
                    service = Some((rank, c.guid));
                }
            }
            _ => {}
        }
    }
    corpse.or(service).map(|(_, guid)| guid)
}

impl Interacting<'_, '_> {
    /// Our player's target, as the server last recorded it (`UNIT_FIELD_TARGET`).
    pub fn target(&mut self) -> Option<u64> {
        let me = self.objects.player()?;
        let guid = match *self.me {
            Some((entity, guid)) if entity == me => guid,
            _ => {
                let guid = self.objects.iter().find(|(_, e, _)| *e == me)?.0;
                *self.me = Some((me, guid));
                guid
            }
        };
        self.objects
            .object(guid)
            .and_then(|fields| fields.unit_target())
            .filter(|g| *g != 0)
    }

    /// Interact with the target; `false` when it is not streamed (yet).
    pub fn with_target(&mut self) -> bool {
        let Some(entity) = self.target().and_then(|g| self.objects.entity(g)) else {
            return false;
        };
        self.out.write(Interact(entity));
        true
    }

    /// Interact with the soft target, if there is one.
    pub fn with_soft_target(&mut self, last: Option<u64>) -> bool {
        let Some(me) = self.objects.player() else {
            return false;
        };
        let Ok(origin) = self.transforms.get(me).map(|t| t.translation) else {
            return false;
        };
        let ahead = self
            .camera
            .single()
            .ok()
            .map(|c| {
                let f = c.forward().as_vec3();
                Vec2::new(f.x, f.z).normalize_or_zero()
            })
            .unwrap_or(Vec2::ZERO);
        let candidates = self.objects.iter().filter_map(|(guid, entity, fields)| {
            if entity == me {
                return None;
            }
            let kind = kind_of(fields)?;
            let at = self.transforms.get(entity).ok()?.translation;
            Some(Candidate {
                guid,
                kind,
                offset: Vec2::new(at.x - origin.x, at.z - origin.z),
            })
        });
        let Some(entity) =
            soft_target(candidates, ahead, last).and_then(|g| self.objects.entity(g))
        else {
            return false;
        };
        self.out.write(Interact(entity));
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn c(guid: u64, kind: Kind, x: f32, z: f32) -> Candidate {
        Candidate {
            guid,
            kind,
            offset: Vec2::new(x, z),
        }
    }

    #[test]
    fn a_corpse_outranks_a_service_and_the_last_target_outranks_distance() {
        let ahead = Vec2::new(0.0, 1.0);
        let all = [
            c(1, Kind::Service, 0.0, 2.0),
            c(2, Kind::Corpse, 0.0, 8.0),
            c(3, Kind::Corpse, 0.0, 3.0),
        ];
        assert_eq!(soft_target(all.into_iter(), ahead, None), Some(3));
        assert_eq!(soft_target(all.into_iter(), ahead, Some(2)), Some(2));
        // Out of range, a corpse is nothing; the service ahead takes it.
        let far = [c(1, Kind::Service, 0.0, 2.0), c(2, Kind::Corpse, 0.0, 11.0)];
        assert_eq!(soft_target(far.into_iter(), ahead, Some(2)), Some(1));
    }

    #[test]
    fn a_service_must_be_near_and_not_behind_the_camera() {
        let ahead = Vec2::new(0.0, 1.0);
        let behind = [c(1, Kind::Service, 0.0, -2.0)];
        assert_eq!(soft_target(behind.into_iter(), ahead, None), None);
        let far = [c(1, Kind::Service, 0.0, 7.0)];
        assert_eq!(soft_target(far.into_iter(), ahead, None), None);
        // Straight ahead beats nearer but off to the side.
        let two = [c(1, Kind::Service, 3.0, 0.5), c(2, Kind::Service, 0.0, 4.0)];
        assert_eq!(soft_target(two.into_iter(), ahead, None), Some(2));
        // With no camera direction, the nearest.
        assert_eq!(soft_target(two.into_iter(), Vec2::ZERO, None), Some(1));
    }
}
