#![allow(unused_parens)]
use bevy::{prelude::*, render::texture::ImageSampler, window::{PresentMode::AutoVsync, WindowResolution}, utils::Duration};
use bevy_rapier2d::prelude::*;
use bevy_prototype_lyon::prelude::*;
use bevy_kira_audio::prelude::*;

use cfg_if::cfg_if;

pub mod player;
pub mod level;

use player::*;
use level::*;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

cfg_if! {
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
            AudioPlugin,
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
        .add_systems(Startup, (setup_graphics, load_all_levels, setup_music, setup_slow_load))
        .add_systems(OnEnter(GameState::Loading), (setup_current_level,).run_if(in_state(GameState::Loading)))
        .add_systems(Update, (slow_load_level, switch_playing).run_if(in_state(GameState::Loading)))
        .add_systems(OnEnter(GameState::Playing), (spawn_player,))
        .add_systems(Update, (set_gravity, mouse_draw, collect_star).run_if(in_state(GameState::Playing)))
        .add_systems(OnExit(GameState::Playing), (cleanup_level, switch_level))
        .run();
}


fn setup_graphics(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn setup_music(asset_server: Res<AssetServer>, audio: Res<Audio>) {
    audio.play(asset_server.load("Tokyo Ghoul：re OST - Mvt.11 “Memories”.ogg"))
        .looped()
        .fade_in(AudioTween::new(Duration::from_secs(1), AudioEasing::OutPowi((2))))
        .with_volume(0.62);
}

fn setup_slow_load(mut commands: Commands) {
    commands.insert_resource(SlowLoadTimer(Timer::from_seconds(0.3, TimerMode::Repeating)));
}

