use bevy::prelude::*;
use bevy_rapier2d::physics::{RapierPhysicsPlugin, RapierConfiguration,RigidBodyHandleComponent};
use bevy_rapier2d::rapier::dynamics::{RigidBodyBuilder, RigidBodySet};
use bevy_rapier2d::rapier::geometry::ColliderBuilder;
use bevy_rapier2d::rapier::na::Vector2;
use rand::Rng;
fn main() {
    App::build()
    .add_plugins(DefaultPlugins)
    .add_plugin(RapierPhysicsPlugin)
    .add_startup_system(setup.system())
    .add_system(movement_system.system())
    .add_system(spawn_bullet.system())
    .add_system(move_bullets.system())
    .add_system(spawn_enemies.system())
    .add_system(despawn_bullets.system())
    .add_resource(BulletSpeedTimer(Timer::from_seconds(0.1, true)))
    .add_resource(EnemySpawnTimer(Timer::from_seconds(3.0, true)))
    .add_resource(EnemyCount(0))
    .run();

    //defaults to a window of 1280x720. 
}



fn setup(
    commands: &mut Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut rapier_config: ResMut<RapierConfiguration>
    
) {
    //spawn camera
    commands.spawn(Camera2dBundle::default());

    rapier_config.gravity = Vector2::zeros();

    let sprite_size_x = 40.0;
    let sprite_size_y = 40.0;

    // While we want our sprite to look ~40 px square, we want to keep the physics units smaller
    // to prevent float rounding problems. To do this, we set the scale factor in RapierConfiguration
    // and divide our sprite_size by the scale.
    rapier_config.scale = 20.0;
    let collider_size_x = sprite_size_x / rapier_config.scale;
    let collider_size_y = sprite_size_y / rapier_config.scale;


    commands.spawn(SpriteBundle{
        transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
        material: materials.add(Color::WHITE.into()),
        sprite: Sprite::new(Vec2::new(sprite_size_x, sprite_size_y)),
        ..Default::default()
    })
    .with(Player::default())
    .with(Direction::East)
    .with(RigidBodyBuilder::new_dynamic())
    .with(ColliderBuilder::cuboid(collider_size_x / 2.0, collider_size_y / 2.0));
}



fn movement_system(
    mut player_query: Query<( & mut Player, &mut RigidBodyHandleComponent)>,
    time: Res<Time>,
    keyboard_input: Res<Input<KeyCode>>,
    mut rigid_bodies: ResMut<RigidBodySet>,
) {
    let mut x = 0.0;
    let mut y = 0.0;
    if keyboard_input.pressed(KeyCode::W) {
        y = 1.0;
    } else if keyboard_input.pressed(KeyCode::S) {
        y = -1.0;
    }
    if keyboard_input.pressed(KeyCode::A) {
        x = -1.0
    } else if keyboard_input.pressed(KeyCode::D) {
        x = 1.0
    } 
    for ( mut player, rigid_body_component) in player_query.iter_mut() {
        let mut x_velocity = 0.0;
        let mut y_velocity = 0.0;
        //speed * dt * direction + current_velocity = some value. 
        //if that value is larger than the max_velocity, set it to max velocity
        //if a player stops pushing, the player will accelerate in the opposite direction of current movement
        x_velocity += player.velocity.x + player.acceleration * time.delta_seconds() * x;
        y_velocity += player.velocity.y + player.acceleration * time.delta_seconds() * y;
        // if abs value of velocity is greater than
        if player.max_velocity < x_velocity.abs(){
            debug!("before applying max x: {} ", x_velocity );
            x_velocity = player.max_velocity*x; 
            debug!("after applying max x: {} ", x_velocity );
        } 
        if player.max_velocity < y_velocity.abs() {
            debug!("before applying max y: {} ", y_velocity );
            y_velocity = player.max_velocity*y;
            debug!("after applying max y: {} ", y_velocity );

        }
        debug!("before applying friction x: {} ", x_velocity );
        debug!("before applying friction y: {} ", y_velocity );

        x_velocity = apply_frictions(x_velocity);
        y_velocity = apply_frictions(y_velocity);
        
        player.velocity.x = x_velocity;
        debug!("x velocity: {}", x_velocity);
        player.velocity.y = y_velocity;
        debug!("y velocity: {}", y_velocity);

        if let Some(rb) = rigid_bodies.get_mut(rigid_body_component.handle()) {
            rb.set_linvel(player.velocity, true);
        }
    }
}
/// using player position as origin of shot, fires into direction of latest arrowkey position
/// spawns a projectile that despawns on hit or after time elapses
fn spawn_bullet(
    commands: &mut Commands,
    mut player_query: Query<(&Transform, &Player, &mut Timer, &mut Direction)>,
    mut player_entity_query: Query<(&Player, Entity, &Transform, &mut Direction), Without<Timer>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    keyboard_input: Res<Input<KeyCode>>,
    rapier_config: ResMut<RapierConfiguration>,
    time: Res<Time>,
) {
    if keyboard_input.pressed(KeyCode::Space) {
        for(_, entity, transform, last_direction) in player_entity_query.iter_mut() {
            create_bullet(commands, &rapier_config, &keyboard_input, transform, materials.add(Color::YELLOW.into()), last_direction);
            commands.insert_one(entity, Timer::from_seconds(0.15, true));
            debug!("Inserted one timer and created a bullet");
        }
        for (transform, _, mut timer, last_direction) in player_query.iter_mut() {
            debug!("ticking {}", time.delta_seconds());
            timer.tick(time.delta_seconds());
            if timer.finished() {
                debug!("Timer finished so I'm creating one bullet");
                create_bullet( commands, &rapier_config, &keyboard_input, transform, materials.add(Color::YELLOW.into()), last_direction);    
            }
        }
        
    }
}

fn create_enemy(
    commands: &mut Commands,
    rapier_config: &ResMut<RapierConfiguration>,
    material: Handle<ColorMaterial>,
    x_position: i32,
    y_position: i32,
) {
    let sprite_size_x = 10.0;
    let sprite_size_y = 10.0;
    
    let collider_size_x = sprite_size_x / rapier_config.scale;
    let collider_size_y = sprite_size_y / rapier_config.scale;

    commands
        .spawn(SpriteBundle {
            material,
            transform: Transform::from_translation(Vec3::new(x_position as f32, y_position as f32, 0.0)),
            sprite: Sprite::new(Vec2::new(sprite_size_x, sprite_size_y)),
            ..Default::default()
        })
        .with(Enemy)
        .with(RigidBodyBuilder::new_dynamic()
        .translation(x_position as f32 / rapier_config.scale, y_position as f32 / rapier_config.scale))
        .with(ColliderBuilder::cuboid(collider_size_x/2., collider_size_y/2.));
}

fn create_bullet (
    commands: &mut Commands, 
    rapier_config: &ResMut<RapierConfiguration>, 
    keyboard_input: &Res<Input<KeyCode>>,
    transform: &Transform,
    material: Handle<ColorMaterial>,
    default_direction: Mut<Direction>,
) {
    let sprite_size_x = 5.0;
    let sprite_size_y = 5.0;
    // While we want our sprite to look ~40 px square, we want to keep the physics units smaller
    // to prevent float rounding problems. To do this, we set the scale factor in RapierConfiguration
    // and divide our sprite_size by the scale.
    let collider_size_x = sprite_size_x / rapier_config.scale;
    let collider_size_y = sprite_size_y / rapier_config.scale;

    let direction = determine_direction(&keyboard_input, default_direction);
    debug!("current x translation for player entity: {} y: {}", transform.translation.x, transform.translation.y );
    
    let translation = match direction {
        Direction::North => {
            Vec2::new(transform.translation.x, transform.translation.y  + 21.)
        },Direction::NorthEast => {
            Vec2::new(transform.translation.x + 21., transform.translation.y  + 21.)
        },Direction::NorthWest => {
            Vec2::new(transform.translation.x - 21., transform.translation.y  + 21.)
        },Direction::South => {
            Vec2::new(transform.translation.x , transform.translation.y  - 21.)
        },Direction::SouthEast => {
            Vec2::new(transform.translation.x + 21., transform.translation.y  - 21.)
        }, Direction::Southwest => {
            Vec2::new(transform.translation.x  - 21., transform.translation.y  - 21.)
        }, Direction::East => {
            Vec2::new(transform.translation.x  + 21., transform.translation.y )
        }, Direction::West => {
            Vec2::new(transform.translation.x  - 21., transform.translation.y )
        }
    };

    commands
        .spawn(SpriteBundle{
            material,
            transform: Transform::from_translation(Vec3::new(transform.translation.x, transform.translation.y, 0.)),
            sprite: Sprite::new(Vec2::new(sprite_size_x, sprite_size_y)),
            ..Default::default()
        })
        .with(direction)
        .with(Bullet(30.))
        .with(RigidBodyBuilder::new_dynamic()
        .translation(translation.x / rapier_config.scale, translation.y / rapier_config.scale))
        .with(ColliderBuilder::cuboid(collider_size_x/2., collider_size_y/2.))
        .with(BulletLifetime(Timer::from_seconds(1.5, true)));
}

fn move_bullets(
    mut query_bullet: Query<(& Direction, &RigidBodyHandleComponent, &Bullet)>,
    mut rigid_bodies: ResMut<RigidBodySet>,
    mut timer: ResMut< BulletSpeedTimer>,
    time: Res<Time>,
) {
    timer.0.tick(time.delta_seconds());
    if timer.0.finished() {
        for (direction, rigid_body_handle, bullet) in query_bullet.iter_mut() {
            debug!("Direction: {:?}", direction);
            match direction {
                Direction::North => {
                    if let Some(rb) = rigid_bodies.get_mut(rigid_body_handle.handle()) {
                        rb.set_linvel(Vector2::new(0.0, bullet.0), true);
                    }
                },Direction::NorthEast => {
                    if let Some(rb) = rigid_bodies.get_mut(rigid_body_handle.handle()) {
                        rb.set_linvel(Vector2::new(bullet.0 * 0.7, bullet.0 * 0.7), true);
                    }
                },Direction::NorthWest => {
                    if let Some(rb) = rigid_bodies.get_mut(rigid_body_handle.handle()) {
                        rb.set_linvel(Vector2::new(-bullet.0 * 0.7, bullet.0 * 0.7), true);
                    }
                },Direction::South => {
                    if let Some(rb) = rigid_bodies.get_mut(rigid_body_handle.handle()) {
                        rb.set_linvel(Vector2::new(0.0, -bullet.0), true);
                    }
                },Direction::SouthEast => {
                    if let Some(rb) = rigid_bodies.get_mut(rigid_body_handle.handle()) {
                        rb.set_linvel(Vector2::new(bullet.0 * 0.7, -bullet.0 * 0.7), true);
                    }
                }, Direction::Southwest => {
                    if let Some(rb) = rigid_bodies.get_mut(rigid_body_handle.handle()) {
                        rb.set_linvel(Vector2::new(-bullet.0 * 0.7, -bullet.0 * 0.7), true);
                    }
                }, Direction::East => {
                    if let Some(rb) = rigid_bodies.get_mut(rigid_body_handle.handle()) {
                        rb.set_linvel(Vector2::new(bullet.0, 0.0), true);
                    }
                }, Direction::West => {
                    if let Some(rb) = rigid_bodies.get_mut(rigid_body_handle.handle()) {
                        rb.set_linvel(Vector2::new(-bullet.0, 0.0), true);
                    }
                }
            }
        }
    }
}

fn despawn_bullets(
    commands: &mut Commands,
    mut bullet_query: Query<(&mut BulletLifetime, Entity)>,
    time: Res<Time>,
) {
    for (mut bullet_timer, entity) in bullet_query.iter_mut() {
        bullet_timer.0.tick(time.delta_seconds());
        if bullet_timer.0.finished() {
            debug!("Despawning a bullet");
            commands.despawn(entity);
        }
    }
}

///generates a random number that is outside of the range of the player position plus some buffer distance
fn spawn_enemies(
    commands: &mut Commands,
    player_position_query: Query<&Transform, With<Player>>,
    mut enemy_count: ResMut<EnemyCount>,
    mut enemy_spawn_timer: ResMut<EnemySpawnTimer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    rapier_config: ResMut<RapierConfiguration>,
    time: Res<Time>,
) {
    //get player position, generate random number around that position, spawn the enemy there, 
    // use timer and enemy count to decide when to spawn
    enemy_spawn_timer.0.tick(time.delta_seconds());
    if enemy_spawn_timer.0.finished() && enemy_count.0 < 20 {
        info!("timer finished");
        for transform in player_position_query.iter() {
            let (x_position, y_position) = generate_xy_values(&transform);
            info!("Spawn at pos x: {}, pos y: {}", x_position, y_position);
            create_enemy(commands, &rapier_config, materials.add(Color::RED.into()), x_position, y_position);
            enemy_count.0 +=1;
        }   
    }
}
///Enemies will know where player is and move towards that direction
/// TODO: Maybe some types of enemies move in different ways.
fn move_enemies(

) {

}

fn generate_xy_values(transform: &Transform) -> (i32, i32) {
    let window_max_x= 640;
    let window_max_y= 360;
    let window_min_x = -window_max_x;
    let window_min_y = -window_max_y;
    let mut rng = rand::thread_rng();
    info!("translation.x: {}, translation.y: {}", transform.translation.x, transform.translation.y);
    let mut x = 0;
    let mut is_x_valid = false;
    if transform.translation.x as i32 + 50 < window_max_x {
        x = rng.gen_range(transform.translation.x as i32 + 50, window_max_x);
        is_x_valid = true;
    }
    let mut x2 = 0;
    let mut is_x2_valid = false;
    if transform.translation.x as i32 -50 > window_min_x {
        x2 = rng.gen_range(window_min_x, transform.translation.x as i32 + 50);
        is_x2_valid = true;
    }
    let mut y = 0;
    let mut is_y_valid = false;
    if transform.translation.y as i32 + 50 < window_max_y {
        y = rng.gen_range(transform.translation.y as i32 + 50, window_max_y);
        is_y_valid = true;
    }
    let mut y2 = 0;
    let mut is_y2_valid = false;
    if transform.translation.y as i32 - 50 > window_min_y {
        y2 = rng.gen_range(window_min_y, transform.translation.y as i32) - 50;
        is_y2_valid = true;
    }
    let x_pair = [x, x2];
    let y_pair = [y, y2];
    // pick between one of the two x values, as long as the value is within range of 0..1280 for x and 0..720 for y.
    let choose_x = rng.gen_range(0usize, 2usize);
    let choose_y = rng.gen_range(0usize,2usize);
    let mut x_position = 0;
    let mut y_position = 0;
    if is_x_valid && is_x2_valid {
        x_position = x_pair[choose_x];
    } else if is_x_valid {
        x_position = x;
    } else if is_x2_valid {
        x_position = x2;
    }
    if is_y_valid && is_y2_valid {
        y_position = y_pair[choose_y];
    } else if is_y_valid {
        y_position = y;
    } else if is_y2_valid {
        y_position = y2;
    }
    info!("x_position: {}, y_position: {}, translation.x: {}, translation.y: {}", x_position, y_position, transform.translation.x, transform.translation.y);
    (x_position, y_position)
}

///TODO: Don't think it's proper to pass and mutate the past_direction here. 
/// logic should be above this to generalize this method for later uses
fn determine_direction(keyboard_input: &Res<Input<KeyCode>>, mut past_direction: Mut<Direction>) -> Direction {
    let mut  latest_direction = *past_direction;
    if keyboard_input.pressed(KeyCode::Up) 
    && !(keyboard_input.pressed(KeyCode::Left)  || keyboard_input.pressed(KeyCode::Right)) {
        latest_direction = Direction::North;
    } else if keyboard_input.pressed(KeyCode::Down)
    && !(keyboard_input.pressed(KeyCode::Left) || keyboard_input.pressed(KeyCode::Right)) {
        latest_direction = Direction::South;
    } else if keyboard_input.pressed(KeyCode::Up) && keyboard_input.pressed(KeyCode::Left) {
        latest_direction =  Direction::NorthWest;
    } else if keyboard_input.pressed(KeyCode::Up) && keyboard_input.pressed(KeyCode::Right) {
        latest_direction = Direction::NorthEast;
    } else if keyboard_input.pressed(KeyCode::Down) && keyboard_input.pressed(KeyCode::Right) {
        latest_direction = Direction::SouthEast;
    } else if keyboard_input.pressed(KeyCode::Down) && keyboard_input.pressed(KeyCode::Left) {
        latest_direction = Direction::Southwest;
    } else if keyboard_input.pressed(KeyCode::Left) {
        latest_direction = Direction::West;
    } else if keyboard_input.pressed(KeyCode::Right) {
        latest_direction = Direction::East;
    }
        *past_direction = latest_direction;
        latest_direction
        
}
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
pub struct ShootEvent(Entity);

pub struct BulletTimer(Timer);

pub struct BulletSpeedTimer(Timer);

pub struct BulletLifetime(Timer);

pub struct EnemySpawnTimer(Timer);

pub struct EnemyCount(i32);

pub struct Enemy;

pub struct Player{
    max_velocity: f32,
    acceleration: f32,
    velocity: Vector2<f32>,
}
pub struct Bullet(f32);

impl Default for Player {
    fn default() -> Self {
        Player {
            max_velocity: 20.0,
            acceleration: 50.0,
            velocity: Vector2::new(0.0, 0.0),
        }
    }
}


pub fn apply_frictions( mut velocity: f32 ) -> f32 {
    let friction_force = 0.02;
    if velocity.abs() > 0.0 {
        if velocity > 0. {
            velocity -= 0.1;
            if velocity < 0. {
                velocity = 0.0;
            } else {
                velocity -= friction_force * velocity;
            }
        } else {
            debug!("neg velocity: {}", velocity);
            velocity += 0.3;
            if velocity > 0. {
                velocity = 0.0;
            } else {
                debug!("neg velocity added: {}", velocity);
                velocity += friction_force * velocity.abs();
                debug!("neg velocity added frictionforce: {}", velocity);

            }
        }
    }
    velocity
}