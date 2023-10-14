use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;
use bevy_rapier2d::prelude::*;

pub static SCENE_Z_INDEX: f32 = -1.;
pub static STAR_Z_INDEX: f32 = 3.;
pub static LEVEL_SCENE_LINE_WIDTH: f32 = 5.;

use crate::{LevelState, GameState, ColliderType, PlayerStatus};

#[derive(Debug, Clone)]
pub struct Level {
    pub vertices_vec: Vec<Vec<Vec2>>,
    pub stars: Vec<Vec2>,
}

impl Level {
    pub fn load_all_levels() -> Vec<Self> {
        let levels = [
            include_str!("./levels/welcome.txt"),
            include_str!("./levels/howareu.txt"),
        ];

        levels.iter().map(|f| {
            let mut vertices_vec = Vec::new();
            let mut stars = Vec::new();
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
                    _ => { error!("Not supported parsing mode {} ", mode); }
                }
                vertices_vec.push(vertices);
            }
            Level {
                vertices_vec,
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

pub fn load_all_levels(mut all_levels: ResMut<GameLevels>, mut game_state: ResMut<NextState<GameState>>) {
    all_levels.0 = Level::load_all_levels();
    game_state.set(GameState::Playing);
}

pub fn switch_level(
    game_state: Res<State<GameState>>,
    all_levels: ResMut<GameLevels>,
    mut level_state: ResMut<LevelState>,
) {
    if *game_state != GameState::Playing {
        return;
    }
    let state = *level_state;
    if state.stars <= 0 && state.id < all_levels.0.len() - 1 {
        *level_state = LevelState {
            id: state.id + 1,
            stars: all_levels.0[state.id + 1].stars.len(),
        };
    }
}

pub fn setup_current_level(
    all_levels: Res<GameLevels>,
    level_state: Res<LevelState>,
    mut commands: Commands,
) {
    let level_id = (*level_state).id;

    for vertices in &all_levels.0[level_id].vertices_vec {
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
        ));
    }

    let stars = &all_levels.0[level_id].stars;
    if stars.len() < 1 { return; }
    for star in stars {
        commands.spawn((
            ShapeBundle {
                path: GeometryBuilder::build_as(&shapes::Circle {
                    radius: 20.0,
                    center: Vec2::ZERO,
                }),
                ..default()
            },
            Collider::ball(10.),
            Fill::color(Color::WHITE),
            ColliderType::Star,
            ActiveEvents::COLLISION_EVENTS,
        ))
        .insert(Transform::from_xyz(star.x, star.y, SCENE_Z_INDEX),)
        .insert(Sensor);
    }
}

pub fn collect_star(
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
                    level_state.stars -= 1;
                    info!("Star collected! {} left", level_state.stars);
                    commands.entity(star).despawn();
                }
            }
        }
    }
}