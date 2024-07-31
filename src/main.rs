#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use crate::game::{
    level::{self, UserDefinedStartupLevel},
    GamePlugin,
};
use bevy::prelude::*;
use framework::{level_asset::LevelAsset, loading_queue::WatchAssetLoading};
use game::{
    audio::{are_audio_assets_ready, load_audio_assets, AudioAssets},
    level::LevelInitialized,
    minion::minion_builder::{are_minion_assets_ready, load_minion_assets, MinionAssets},
    objects::assets::{are_object_assets_ready, load_object_assets, GameObjectAssets},
    player::player_builder::{are_player_assets_ready, load_player_assets, PlayerAssets},
};

pub mod framework;
pub mod game;
pub mod runner;
pub mod tooling;

pub struct GameRunArgs {
    pub init: bool,
    pub level: Option<String>,
}

impl Default for GameRunArgs {
    fn default() -> Self {
        Self {
            init: true,
            level: None,
        }
    }
}

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
pub enum AppState {
    #[default]
    Startup,
    Loading,
    InitLevel,
    Ingame,
}

fn main() -> AppExit {
    let (mut app, run_args) = runner::create_app();

    /* Add the base plugins */
    #[cfg(feature = "debug_visuals")]
    app.add_plugins(tooling::fps_counter::FpsCounterPlugin);

    /* Add the plugins depending on our config */
    if run_args.init {
        app.add_plugins(GamePlugin)
            .init_state::<AppState>()
            .add_event::<LevelInitialized>()
            .add_systems(Startup, setup_loading_screen)
            .add_systems(OnEnter(AppState::Loading), load_audio_assets)
            .add_systems(OnEnter(AppState::Loading), load_minion_assets)
            .add_systems(OnEnter(AppState::Loading), load_object_assets)
            .add_systems(OnEnter(AppState::Loading), load_player_assets)
            .add_systems(Update, wait_for_load.run_if(in_state(AppState::Loading)))
            .add_systems(OnEnter(AppState::InitLevel), load_level)
            .add_systems(
                Update,
                (game::level::init_level, wait_for_level_load)
                    .run_if(in_state(AppState::InitLevel)),
            )
            .add_systems(OnExit(AppState::InitLevel), cleanup_loading_screen);

        if let Some(level) = run_args.level {
            app.insert_resource(UserDefinedStartupLevel(level));
            app.add_systems(Startup, level::load_user_defined_startup_level);
        }
    }
    runner::run_app(&mut app)
}

#[derive(Component)]
pub struct LoadingScreenTag;

#[derive(Component)]
pub struct LoadingScreenText;

fn setup_loading_screen(
    mut cmd: Commands,
    mut next: ResMut<NextState<AppState>>,
    // mut audio: ResMut<Audio>,
    // ass: Res<AssetServer>,
) {
    // not working either
    // audio.play_loop(
    //     ass.load("sfx/silence.ogg"),
    //     AudioChannel::AMB,
    //     LoopConfig {
    //         looping: true,
    //         ..Default::default()
    //     },
    // );
    cmd.spawn((
        LoadingScreenTag,
        NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..Default::default()
            },
            ..Default::default()
        },
    ))
    .with_children(|cmd| {
        cmd.spawn((
            LoadingScreenText,
            TextBundle::from_section("Loading...", TextStyle::default()),
        ));
    });
    cmd.spawn((LoadingScreenTag, Camera2dBundle::default()));
    next.set(AppState::Loading);
}

fn cleanup_loading_screen(mut cmd: Commands, loading: Query<Entity, With<LoadingScreenTag>>) {
    for loading in loading.iter() {
        cmd.entity(loading).despawn_recursive();
    }
}

fn wait_for_load(
    ass: Res<AssetServer>,
    player_assets: Res<PlayerAssets>,
    minion_assets: Res<MinionAssets>,
    object_assets: Res<GameObjectAssets>,
    audio_assets: Res<AudioAssets>,
    mut next: ResMut<NextState<AppState>>,
) {
    let ready = are_player_assets_ready(&ass, &player_assets)
        && are_minion_assets_ready(&ass, &minion_assets)
        && are_object_assets_ready(&ass, &object_assets)
        && are_audio_assets_ready(&ass, &audio_assets);

    if ready {
        next.set(AppState::InitLevel)
    }
}

fn load_level(
    ass: Res<AssetServer>,
    mut text: Query<&mut Text, With<LoadingScreenText>>,
    already_init: Option<Res<UserDefinedStartupLevel>>,
    mut load: EventWriter<WatchAssetLoading<LevelAsset>>,
) {
    text.single_mut().sections[0].value = "Initializing Level...".to_owned();

    if !already_init.is_some() {
        info!("Loading Preview Scene");
        load.send(WatchAssetLoading::new(ass.load("level/preview.level")));
    }
}

fn wait_for_level_load(load: EventReader<LevelInitialized>, mut next: ResMut<NextState<AppState>>) {
    if load.len() > 0 {
        next.set(AppState::Ingame);
    }
}
