use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;
use bevy_rapier2d::prelude::*;
use bevy_kira_audio::prelude::*;

pub static SCENE_Z_INDEX: f32 = -1.;
pub static STAR_Z_INDEX: f32 = 3.;
pub static LEVEL_SCENE_LINE_WIDTH: f32 = 5.;
pub static STAR_RADIUS: f32 = 15.;

use crate::{GameState, ColliderType, PlayerStatus};

#[derive(Debug, Clone, Copy, Default, Resource, Reflect)]
#[reflect(Resource)]
pub struct LevelState {
    pub id: usize,
    pub stars: usize,
}

#[derive(Debug, Clone)]
pub struct Level {
    pub player: Vec2,
    pub polylines: Vec<Vec<Vec2>>,
    pub stars: Vec<Vec2>,
}

impl Level {
    pub fn load_all_levels() -> Vec<Self> {
        let levels = [
            include_str!("./levels/welcome.txt"),
            include_str!("./levels/howareu.txt"),
        ];

        levels.iter().map(|f| {
            let mut polylines = Vec::new();
            let mut stars = Vec::new();
            let mut player = Vec2::ZERO;
            for line in f.lines() {
                let mut vertices = Vec::new();
                let (mode, data) = line.split_at(2);
                match mode {
                    "l " => {
                        for vertex in data.split(' ') {
                            let mut coords = vertex.split(',');
                            let x = coords.next().unwrap().parse::<f32>().unwrap();
                            let y = coords.next().unwrap().parse::<f32>().unwrap();
                            vertices.push(Vec2::new(x, y));
                        }
                    }
                    "s " => {
                        for star in data.split(' ') {
                            let mut coords = star.split(',');
                            let x = coords.next().unwrap().parse::<f32>().unwrap();
                            let y = coords.next().unwrap().parse::<f32>().unwrap();
                            stars.push(Vec2::new(x, y));
                        }
                    }
                    "p " => {
                        let mut coords = data.split(',');
                        let x = coords.next().unwrap().parse::<f32>().unwrap();
                        let y = coords.next().unwrap().parse::<f32>().unwrap();
                        player = Vec2::new(x, y);
                    }
                    _ => { error!("Not supported parsing mode {} ", mode); }
                }
                polylines.push(vertices);
            }
            Level {
                player,
                polylines,
                stars,
            }
        }).collect()
    }
}

#[derive(Debug, Resource, Clone)]
pub struct GameLevels(pub Vec<Level>);

impl Default for GameLevels {
    fn default() -> Self {
        GameLevels(Vec::new())
    }
}


pub fn load_all_levels(
    mut all_levels: ResMut<GameLevels>,
    mut current_level_state: ResMut<LevelState>
) {
    all_levels.0 = Level::load_all_levels();
    let level_brief = all_levels.0.iter().map(|level| {
        format!("{} polylines, {} stars", level.polylines.len(), level.stars.len())
    }).collect::<Vec<_>>();
    info!("Loaded {} levels:\n{:?}", all_levels.0.len(), level_brief);
    *current_level_state = LevelState {
        id: 0,
        stars: all_levels.0[0].stars.len(),
    };
}

pub fn switch_level(
    mut next_state: ResMut<NextState<GameState>>,
    all_levels: Res<GameLevels>,
    mut level_state: ResMut<LevelState>,
    q_scene: Query<&Visibility, With<ColliderType>>,
) {
    if q_scene.iter().filter(|v| **v == Visibility::Visible).count() > 0 {
        return;
    }
    let state = *level_state;
    if state.id < all_levels.0.len() - 1 {
        info!("Switching to level {}", level_state.id + 1);
        *level_state = LevelState {
            id: state.id + 1,
            stars: all_levels.0[state.id + 1].stars.len(),
        };
    } else {
        info!("No more levels, replaying level 0");
        *level_state = LevelState {
            id: 0,
            stars: all_levels.0[0].stars.len(),
        };
    }
    next_state.set(GameState::Loading);
}

#[derive(Debug, Clone, Resource)]
pub struct SlowLoadTimer(pub Timer);


pub fn setup_current_level(
    all_levels: Res<GameLevels>,
    current_level_state: Res<LevelState>,
    mut commands: Commands,
) {
    info!("Set up level {} with stars {}", current_level_state.id, current_level_state.stars);
    let level_id = (*current_level_state).id;

    for vertices in &all_levels.0[level_id].polylines {
        if vertices.len() < 1 { continue; }
        let mut path = PathBuilder::new();
        path.move_to(vertices[0]);
        for vertex in vertices.iter().skip(1) { path.line_to(*vertex); }
        let polyline = path.build();
        commands.spawn((
            Collider::polyline(vertices.clone(), None),
            ShapeBundle {
                path: GeometryBuilder::build_as(
                    &polyline,
                ),
                ..Default::default()
            },
            Stroke::new(Color::BLACK, LEVEL_SCENE_LINE_WIDTH),
            ColliderType::Scene,
        )).insert((
            Transform::from_xyz(0., 0., SCENE_Z_INDEX),
        )).insert(Visibility::Hidden);
    }

    let stars = &all_levels.0[level_id].stars;
    if stars.len() < 1 { return; }
    for star in stars {
        commands.spawn((
            ShapeBundle {
                path: GeometryBuilder::build_as(&shapes::Circle {
                    radius: STAR_RADIUS,
                    center: Vec2::ZERO,
                }),
                ..default()
            },
            Collider::ball(STAR_RADIUS * 0.8),
            Fill::color(Color::WHITE),
            ColliderType::Star,
            ActiveEvents::COLLISION_EVENTS,
        ))
        .insert(Transform::from_xyz(star.x, star.y, SCENE_Z_INDEX),)
        .insert(Visibility::Hidden)
        .insert(Sensor);
    }
    info!("Level {} set up, playing", level_id);
}

pub fn slow_load_level(
    mut slow_load_timer: ResMut<SlowLoadTimer>,
    time: Res<Time>,
    mut q_scene: Query<&mut Visibility, With<ColliderType>>,
) {
    if slow_load_timer.0.tick(time.delta()).just_finished() {
        for mut visibility in q_scene.iter_mut().filter(|v| **v == Visibility::Hidden).take(1) {
            *visibility = Visibility::Visible;
        }
    }
}

pub fn slow_clean_level(
    mut slow_load_timer: ResMut<SlowLoadTimer>,
    time: Res<Time>,
    mut q_scene: Query<&mut Visibility, With<ColliderType>>,
) {
    if slow_load_timer.0.tick(time.delta()).just_finished() {
        for mut visibility in q_scene.iter_mut().filter(|v| **v == Visibility::Visible).take(1) {
            *visibility = Visibility::Hidden;
        }
    }
}

pub fn switch_playing(
    mut next_state: ResMut<NextState<GameState>>,
    q_scene: Query<&Visibility, With<ColliderType>>,
) {
    // if all scenes and stars are visible, then playing
    if q_scene.iter().filter(|v| **v == Visibility::Hidden).count() > 0 {
        return;
    }
    next_state.set(GameState::Playing);
}

pub fn collect_star(
    mut next_state: ResMut<NextState<GameState>>,
    mut level_state: ResMut<LevelState>,
    mut commands: Commands,
    q_stars: Query<(Entity, &ColliderType), With<Sensor>>,
    q_player: Query<Entity, With<PlayerStatus>>,
    rapier_context: Res<RapierContext>,
) {
    let player = q_player.single();
    
    /* Iterate through all the contact pairs involving a specific collider. */
    for (collider1, collider2, intersecting) in rapier_context.intersections_with(player) {
        if intersecting {
            let other_collider = if collider1 == player { collider2 } else { collider1 };
            info!("Player is intersecting with {:?}", other_collider);
            if let Ok((star, collider_type)) = q_stars.get(other_collider) {
                info!("collider_type: {:?}", collider_type);
                if *collider_type == ColliderType::Star {
                    if level_state.stars <= 1 {
                        level_state.stars -= 1;
                        info!("No more stars left, switching level");
                        commands.entity(star).despawn();
                        next_state.set(GameState::Cleaning);
                        return;
                    }
                    level_state.stars -= 1;
                    info!("Star collected! {} left", level_state.stars);
                    commands.entity(star).despawn();
                }
            }
        }
    }
}

pub fn clean_current_level(
    mut commands: Commands,
    q_colliders: Query<Entity, With<ColliderType>>,
    q_player: Query<Entity, With<PlayerStatus>>
) {
    for entity in q_colliders.iter() {
        commands.entity(entity).despawn_recursive();
    }
    for entity in q_player.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

pub fn play_drawing_sound(
    asset_server: Res<AssetServer>,
    audio: Res<Audio>,
) {
    audio.play(asset_server.load("drawing.ogg"))
        .with_volume(0.3);
}