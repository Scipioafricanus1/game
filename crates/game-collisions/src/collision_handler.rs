use bevy::prelude::*;
use bevy_rapier2d::{
    physics::{RigidBodyHandleComponent, EventQueue}, 
    rapier::{
        dynamics::RigidBodySet,
        geometry::{ColliderSet, ContactEvent},
    }
};
use game_data::*;


pub fn handle_contacts(
    mut commands: Commands,
    events: Res<EventQueue>,
    colliders: ResMut<ColliderSet>,
    bodies: ResMut<RigidBodySet>,
    enemies: Query<&Enemy>,
    mut bullets: Query<&Bullet>,
) {
    let mut contacts = vec![];
    while let Ok(contact_event) = events.contact_events.pop() {
        match contact_event {
            ContactEvent::Started(h1, h2) => {
                let c1 = colliders.get(h1).unwrap();
                let c2 = colliders.get(h2).unwrap();
                let b1 = bodies.get(c1.parent()).unwrap();
                let b2 = bodies.get(c2.parent()).unwrap();
                let e1 = Entity::from_bits(b1.user_data as u64);
                let e2 = Entity::from_bits(b2.user_data as u64);
                
                if bullets.get_component::<Bullet>(e1).is_ok() 
                && enemies.get_component::<Enemy>(e2).is_ok() {
                    info!("e1 is a bullet");
                    contacts.push(Contacts::BulletEnemy(e1,e2));
                } else if bullets.get_component::<Bullet>(e2).is_ok() 
                && enemies.get_component::<Enemy>(e1).is_ok() {
                    info!("e2 is a bullet");
                    contacts.push(Contacts::BulletEnemy(e2,e1));
                }
            },
            _ => (),
        }        
    }
    
    for contact in contacts.into_iter() {
        match contact {
            Contacts::BulletEnemy(e1, e2) => {
                info!("despawning bullet");
                let bullet = bullets.get_component_mut::<Bullet>(e1);
                commands.entity(e1).despawn();
                let enemy = enemies.get_component::<Enemy>(e2);
                commands.entity(e2).despawn();
            }
        }
    }
}