use bevy::prelude::*;
use bevy_rapier2d::rapier::na::Vector2;



#[derive(Debug, Clone, Copy)]
pub enum Direction{
    North,
    NorthEast,
    NorthWest,
    East,
    West,
    South,
    SouthEast,
    Southwest,
}
///e1: Bullet, e2: Enemy
pub enum Contacts {
    BulletEnemy(Entity, Entity),
    //TODO: EnemyPlayer(Entity, Entity),
}
pub struct ShootEvent(pub Entity);

pub struct BulletTimer(pub Timer);

pub struct BulletSpeedTimer(pub Timer);

pub struct BulletLifetime(pub Timer);

pub struct EnemySpawnTimer(pub Timer);

pub struct EnemyCount(pub i32);

pub struct Enemy;

pub struct Player{
    pub max_velocity: f32,
    pub acceleration: f32,
    pub velocity: Vector2<f32>,
}
pub struct Bullet(pub f32);

impl Default for Player {
    fn default() -> Self {
        Player {
            max_velocity: 20.0,
            acceleration: 50.0,
            velocity: Vector2::new(0.0, 0.0),
        }
    }
}



#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
