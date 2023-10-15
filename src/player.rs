#![allow(unused_imports)]
use bevy::{prelude::*, window::PrimaryWindow, utils::{HashMap, tracing::level_filters}};
use bevy_rapier2d::prelude::*;
use bevy_prototype_lyon::prelude::*;

use crate::{ColliderType, GameLevels, LevelState, level, GameState};

pub static PLAYER_RADIUS: f32 = 25.0;
pub static PLAYER_GRAVITY_SCALE: f32 = 9.8;
pub static PLAYER_DRAW_LINE_WIDTH: f32 = 10.;
pub static PLAYER_DRAW_DISTANCE_TO_BALL_THRESHOLD: f32 = 23.;
pub static PLAYER_DRAW_VERTICES_DISTANCE_THRESHOLD: f32 = 5.8;
pub static PLAYER_ERASE_DISTANCE_THRESHOLD: f32 = 20.;
pub static PLAYER_DRAW_Z_INDEX: f32 = 1.;

#[derive(Debug, Component, Clone, Copy, Default, PartialEq, Reflect)]
#[reflect(Component)]
pub enum PlayerStatus {
    #[default]
    Still,
    Moving(f32),
    Falling,
}

#[derive(Resource, PartialEq, Debug, Clone, Copy, Reflect)]
#[reflect(Resource)]
pub struct PlayerCurrentLineEntity(pub Option<Entity>);

impl Default for PlayerCurrentLineEntity {
    fn default() -> Self {
        PlayerCurrentLineEntity(None)
    }
}

#[derive(Resource, PartialEq, Debug, Clone, Reflect)]
#[reflect(Resource)]
pub struct Lines(pub HashMap<Entity, Vec<Vec2>>);

impl Default for Lines {
    fn default() -> Self {
        Lines(HashMap::default())
    }
}

pub fn mouse_draw(
    buttons: Res<Input<MouseButton>>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    q_player: Query<&Transform, With<PlayerStatus>>,
    mut q_path: Query<(&mut Path, &mut Collider), With<ColliderType>>,
    mut current_line_hid: ResMut<PlayerCurrentLineEntity>,
    mut lines: ResMut<Lines>,
    mut commands: Commands,
) {
    let center_x = q_windows.single().width() / 2.0;
    let center_y = q_windows.single().height() / 2.0;
    if let Some(position) = q_windows.single().cursor_position() {
        let (mouse_x, mouse_y) = (position.x - center_x, center_y - position.y);
        let vec_mouse = Vec2::from([mouse_x, mouse_y]);
        let player_translation = q_player.get_single().unwrap().translation;
        if vec_mouse.distance(Vec2::from([player_translation.x, player_translation.y])) < PLAYER_DRAW_DISTANCE_TO_BALL_THRESHOLD {
            return;
        }
        if buttons.just_pressed(MouseButton::Left) {
            let mut path_builder = PathBuilder::new();
            path_builder.move_to(vec_mouse);
            let new_path = path_builder.build();

            let new_path_entity = commands.spawn((
                Collider::compound([(vec_mouse, 0., Collider::ball(PLAYER_DRAW_LINE_WIDTH / 2.0))].to_vec()),
                ShapeBundle {
                    path: GeometryBuilder::build_as(&new_path),
                    ..default()
                },
                Stroke::new(Color::GRAY, PLAYER_DRAW_LINE_WIDTH),
                ColliderType::Ground,
            ))
            .insert(Transform::from_xyz(0., 0., PLAYER_DRAW_Z_INDEX))
            // spawn round head
            .with_children(|children| {
                children.spawn((
                    ShapeBundle {
                        path: GeometryBuilder::build_as(&shapes::Circle {
                            radius: PLAYER_DRAW_LINE_WIDTH / 2.0,
                            center: vec_mouse,
                        }),
                        ..default()
                    },
                    Fill::color(Color::GRAY),
                ));
            })
            .id();

            *lines.0.entry(new_path_entity).or_insert_with(Vec::new) = vec![vec_mouse];

            info!("Spawned line with id: {:?}", new_path_entity);
            (*current_line_hid).0 = Some(new_path_entity);
            return;  // the bevy entity update will not be called until the next frame, so we need to return here
        }
        if buttons.pressed(MouseButton::Right) {
            // erase lines
            let mut new_vertices_hm = lines.0.clone();
            for (e, polyline) in lines.0.clone().into_iter() {
                // spawn one half
                let mut despawn_original_entity = false;
                let mut start_index = 0;
                for i in 0..polyline.len() {
                    if polyline[i].distance(vec_mouse) < PLAYER_ERASE_DISTANCE_THRESHOLD { 
                        despawn_original_entity = true;
                        if i - start_index > 1 {
                            let new_line_vertices = polyline[start_index..i].to_vec();
                            info!("original line length {}", polyline.len());
                            info!("new line [{}..{}]", start_index, i);
                            let mut new_line_path = PathBuilder::new();
                            new_line_path.move_to(new_line_vertices[0]);
                            for v in new_line_vertices.iter().skip(0) {
                                new_line_path.line_to(*v);
                            }
                            let new_line_path = new_line_path.build();
                            let new_polyline_entity = commands.spawn((
                                Collider::compound(new_line_vertices.iter()
                                    .map(|v| (*v, 0., Collider::ball(PLAYER_DRAW_LINE_WIDTH / 2.0)))
                                    .collect::<Vec<_>>()),
                                ShapeBundle {
                                    path: GeometryBuilder::build_as(&new_line_path),
                                    ..default()
                                },
                                Stroke::new(Color::GRAY, PLAYER_DRAW_LINE_WIDTH),
                                ColliderType::Ground,
                            ))
                            .insert(Transform::from_xyz(0., 0., PLAYER_DRAW_Z_INDEX))
                            // spawn round head and tail
                            .with_children(|children| {
                                children.spawn((
                                    ShapeBundle {
                                        path: GeometryBuilder::build_as(&shapes::Circle {
                                            radius: PLAYER_DRAW_LINE_WIDTH / 2.0,
                                            center: *new_line_vertices.first().unwrap_or(&vec_mouse),
                                        }),
                                        ..default()
                                    },
                                    Fill::color(Color::GRAY),
                                ));
                            })
                            .with_children(|children| {
                                children.spawn((
                                    ShapeBundle {
                                        path: GeometryBuilder::build_as(&shapes::Circle {
                                            radius: PLAYER_DRAW_LINE_WIDTH / 2.0,
                                            center: *new_line_vertices.last().unwrap_or(&vec_mouse),
                                        }),
                                        ..default()
                                    },
                                    Fill::color(Color::GRAY),
                                ));
                            })
                            
                            .id();
                            new_vertices_hm.insert(new_polyline_entity, new_line_vertices);
                        }
                        start_index = i + 1;
                        info!("start index {}", start_index);
                        if start_index >= polyline.len() { break; }
                    }
                }
                // spawn the rest
                if start_index + 1 < polyline.len() && despawn_original_entity == true {
                    let new_line_vertices = polyline[start_index..].to_vec();
                    let mut new_line_path = PathBuilder::new();
                    new_line_path.move_to(new_line_vertices[0]);
                    for v in new_line_vertices.iter().skip(0) {
                        new_line_path.line_to(*v);
                    }
                    let new_line_path = new_line_path.build();
                    let new_polyline_entity = commands.spawn((
                        Collider::compound(new_line_vertices.iter()
                            .map(|v| (*v, 0., Collider::ball(PLAYER_DRAW_LINE_WIDTH / 2.0)))
                            .collect::<Vec<_>>()),
                        ShapeBundle {
                            path: GeometryBuilder::build_as(&new_line_path),
                            ..default()
                        },
                        Stroke::new(Color::GRAY, PLAYER_DRAW_LINE_WIDTH),
                        ColliderType::Ground,
                    ))
                    .insert((
                        Transform::from_xyz(0., 0., PLAYER_DRAW_Z_INDEX),
                    ))
                    // spawn round head and tail
                    .with_children(|children| {
                        children.spawn((
                            ShapeBundle {
                                path: GeometryBuilder::build_as(&shapes::Circle {
                                    radius: PLAYER_DRAW_LINE_WIDTH / 2.0,
                                    center: *new_line_vertices.first().unwrap_or(&vec_mouse),
                                }),
                                ..default()
                            },
                            Fill::color(Color::GRAY),
                        ));
                    })
                    .with_children(|children| {
                        children.spawn((
                            ShapeBundle {
                                path: GeometryBuilder::build_as(&shapes::Circle {
                                    radius: PLAYER_DRAW_LINE_WIDTH / 2.0,
                                    center: *new_line_vertices.last().unwrap_or(&vec_mouse),
                                }),
                                ..default()
                            },
                            Fill::color(Color::GRAY),
                        ));
                    }).id();
                    new_vertices_hm.insert(new_polyline_entity, new_line_vertices);
                }
                if despawn_original_entity == true {
                    new_vertices_hm.remove(&e);
                    commands.entity(e).despawn_recursive();
                }
            }
            lines.0 = new_vertices_hm;
            return;
        }
        if buttons.pressed(MouseButton::Left) {
            if let Some(handle) = (*current_line_hid).0 {
                if let Ok((mut old_path, mut collider)) = q_path.get_mut(handle) {
                    // build new shape
                    let new_line_shape = (*lines).0.get_mut(&handle).ok_or_else(|| {
                        info!("Failed to get path for entity: {:?}", handle);
                        return;
                    }).unwrap();
                    let last_endpoint = new_line_shape.last().unwrap();
                    if vec_mouse.distance(*last_endpoint) < PLAYER_DRAW_VERTICES_DISTANCE_THRESHOLD {
                        return;
                    }

                    new_line_shape.push(vec_mouse);
                    let mut new_path = PathBuilder::new();
                    new_path.move_to(new_line_shape[0]);
                    for v in new_line_shape.iter().skip(0) {
                        new_path.line_to(*v);
                    }
                    let new_path = new_path.build();
                    *old_path = ShapePath::new().add(&new_path).build();
                    *collider = Collider::compound(new_line_shape.iter().map(|v| (*v, 0., Collider::ball(PLAYER_DRAW_LINE_WIDTH / 2.0))).collect::<Vec<_>>());
                    // spawn round tail
                    commands.entity(handle).with_children(|children| {
                        children.spawn((
                            ShapeBundle {
                                path: GeometryBuilder::build_as(&shapes::Circle {
                                    radius: PLAYER_DRAW_LINE_WIDTH / 2.0,
                                    center: vec_mouse,
                                }),
                                ..default()
                            },
                            Fill::color(Color::GRAY),
                        ));
                    });
                } else { 
                    info!("Failed to get path for entity: {:?}", handle);
                    (*current_line_hid).0 = None;
                    return;
                };
            } else { return; }
        } else if buttons.just_released(MouseButton::Left) {
            info!("Released mouse");
            (*current_line_hid).0 = None;
        }
    } else {
        return;
    }

}

pub fn spawn_player(
    current_level: Res<LevelState>,
    all_levels: Res<GameLevels>,
    mut commands: Commands
) {
    let level_id = (*current_level).id;
    let player_start_position = (*all_levels).0[level_id].player;
    commands
        .spawn((
            ShapeBundle {
                path: GeometryBuilder::build_as(&shapes::Circle {
                    radius: PLAYER_RADIUS,
                    center: Vec2::ZERO,
                }),
                ..default()
            },
            Fill::color(Color::WHITE),
            RigidBody::Dynamic,
            PlayerStatus::default(),
        ))
        .insert((
            Collider::ball(PLAYER_RADIUS),
            ColliderMassProperties::Density(1.0),
            ActiveEvents::COLLISION_EVENTS,
            Restitution::coefficient(0.8),
            Friction::default(),
            TransformBundle::from(Transform::from_xyz(player_start_position.x, player_start_position.y, 0.0)),
            Velocity::default(),
            GravityScale(0.0),
            Sleeping::disabled(),
            Ccd::enabled(),
        ));
}


pub fn set_gravity(
    keyboard: Res<Input<KeyCode>>,
    mut gravity: Query<(&mut GravityScale, &mut Velocity), With<PlayerStatus>>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        info!("pressed space");
        let (mut g, mut v) = gravity.single_mut();
        (*g).0 = PLAYER_GRAVITY_SCALE;
        (*v).linvel.y = -0.1;
    }
}