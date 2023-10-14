#![allow(unused_parens)]
use bevy::{prelude::*, render::texture::ImageSampler, window::{PresentMode::AutoVsync, WindowResolution}};
use bevy_rapier2d::prelude::*;
use bevy_prototype_lyon::prelude::*;

use cfg_if::cfg_if;

pub mod player;
pub mod level;

use player::*;
use level::*;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

cfg_if! {
    // When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
    // allocator.
    if #[cfg(feature = "wee_alloc")] {
        extern crate wee_alloc;
        #[global_allocator]
        static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
    }
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn start() {
    run_app();
}

#[derive(Debug, Clone, Copy, Default, Component, PartialEq, Eq, Hash)]
pub enum ColliderType {
    #[default]
    Ground,
    Scene,
    Star,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, States, Default)]
pub enum GameState {
    #[default]
    Loading,
    Playing,
}

#[derive(Debug, Clone, Copy, Default, Resource, Reflect)]
#[reflect(Resource)]
pub struct LevelState {
    pub id: usize,
    pub stars: usize,
}

#[derive(Debug, Clone, Copy, Default, Resource, Reflect)]
#[reflect(Resource)]
pub struct StarsLeft(pub usize);

pub const PIXELS_PER_METER: f32 = 100.0;

pub fn run_app() {
    App::new()
        .add_state::<GameState>()
        .add_plugins((
            DefaultPlugins
                .set(ImagePlugin {
                    default_sampler: ImageSampler::nearest_descriptor(),
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        resolution: WindowResolution::new(1280., 720.),
                        present_mode: AutoVsync,
                        // for wasm
                        fit_canvas_to_parent: true,
                        prevent_default_event_handling: false,
                        ..default()
                    }),
                    ..default()
                }),
        ))
        .add_plugins((
            RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(PIXELS_PER_METER),
        ))
        .add_plugins(ShapePlugin)
        .register_type::<PlayerCurrentLineEntity>()
        .register_type::<Lines>()
        .register_type::<LevelState>()
        .insert_resource(Msaa::Sample8)
        .insert_resource(PlayerCurrentLineEntity::default())
        .insert_resource(Lines::default())
        .insert_resource(LevelState::default())
        .insert_resource(GameLevels::default())
        .add_systems(Startup, (setup_graphics,))
        .add_systems(OnEnter(GameState::Loading), (load_all_levels,))
        .add_systems(OnEnter(GameState::Playing), (setup_current_level, setup_physics,))
        .add_systems(Update, (set_gravity, mouse_draw, collect_star, switch_level).run_if(in_state(GameState::Playing)))
        .run();
}


fn setup_graphics(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn setup_physics(mut commands: Commands) {
    commands
        .spawn((
            ShapeBundle {
                path: GeometryBuilder::build_as(&shapes::Circle {
                    radius: 25.0,
                    center: Vec2::ZERO,
                }),
                ..default()
            },
            Fill::color(Color::WHITE),
            RigidBody::Dynamic,
            PlayerStatus::default(),
        ))
        .insert((
            Collider::ball(25.0),
            ColliderMassProperties::Density(1.0),
            ActiveEvents::COLLISION_EVENTS,
            Restitution::coefficient(0.8),
            Friction::default(),
            TransformBundle::from(Transform::from_xyz(0.0, 200.0, 0.0)),
            Velocity::default(),
            GravityScale(0.0),
            Sleeping::disabled(),
            Ccd::enabled(),
        ));
}

