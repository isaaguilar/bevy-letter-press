use std::collections::VecDeque;

use crate::app::{
    AppState, DARK_COLOR, DisplayLanguage, LIGHT_COLOR, RESOLUTION_HEIGHT, RESOLUTION_WIDTH,
    RUNNING_SPEED,
};
use crate::assets::custom::{ImageAssets, SoundAssets};
use crate::assets::lexi::game_over::GameOverLex;
use crate::assets::lexi::levels::{LevelInfo, LevelLex};
use crate::camera;
use crate::util::handles::BODY_FONT;
use bevy::ecs::system::Commands;
use bevy::input::ButtonInput;
use bevy::input::ButtonState;
use bevy::input::common_conditions::input_just_pressed;
use bevy::input::keyboard::KeyboardInput;
use bevy::math::bounding::{Aabb2d, IntersectsVolume};
use bevy::platform::collections::HashMap;

use bevy::render::render_resource::Texture;
use bevy::sprite::Sprite;
use bevy::ui::{AlignItems, Display, FlexDirection, Node, PositionType, Val};
use bevy::{audio, prelude::*};
use bevy_aspect_ratio_mask::Hud;
use bevy_http_client::prelude::*;
use bevy_simple_text_input::{
    TextInput, TextInputPlugin, TextInputTextColor, TextInputTextFont, TextInputValue,
};
use rand::Rng;
use rand::seq::IndexedRandom;
use serde::Deserialize;

const LEADERBOARD_URL: &'static str = env!("LEADERBOARD_URL");
const MAX_VISIBLE_WEEDS: u32 = 10;
const MIN_PLACEMENT_DISTANCE: f32 = 35.0;

pub(super) fn plugin(app: &mut App) {
    app.init_state::<GameState>()
        .add_event::<SceneChange>()
        .add_event::<RemoveWeed>()
        .add_plugins((TextInputPlugin, HttpClientPlugin))
        .insert_resource(LoadedLevel::default())
        .insert_resource(GameTimer::default())
        .insert_resource(GameStatus::default())
        .insert_resource(PendingSceneChange::default())
        .insert_resource(CurrentLevelId(0))
        .insert_resource(SfxMusicVolume::default())
        .insert_resource(ActiveKey::default())
        .insert_resource(Affirmations::default())
        .insert_resource(TimeSpent::default())
        .insert_resource(DisplayAffirmation::default())
        .insert_resource(KeyMap::default())
        .insert_resource(KeyPosition::default())
        .insert_resource(WeedTracker::default())
        .add_systems(Startup, global_volume_set)
        .add_systems(OnEnter(AppState::Game), (sfx_setup, setup))
        .add_systems(Startup, camera::game_camera)
        .add_systems(
            Update,
            keypress_events.run_if(
                on_event::<KeyboardInput>
                    .and(in_state(GameState::Running))
                    .and(in_state(AppState::Game)),
            ),
        )
        .add_systems(Update, remove_weeds.run_if(on_event::<RemoveWeed>))
        .add_systems(
            Update,
            (
                // keypress_events,
                level_timer_counter,
                animate_key,
                update_healthbar_display,
                update_letters_remaining_display,
                level_timer_countdown,
                update_timeboard,
                update_affirmation_display,
            )
                .run_if(in_state(AppState::Game).and(in_state(GameState::Running))),
        )
        .add_systems(Update, game_over.run_if(on_event::<SceneChange>))
        .add_systems(Update, scene_transition)
        .add_systems(FixedUpdate, (fade_out_and_despawn, fade_in_music))
        .add_systems(OnEnter(AppState::GameOver), waiting_music)
        .add_systems(OnEnter(AppState::Menu), (waiting_music, volume_toggle_hud))
        .add_systems(OnEnter(AppState::HighScores), waiting_music)
        .add_systems(OnEnter(AppState::Credits), setup_credits)
        .add_systems(OnEnter(AppState::LoadNextLevel), setup_load_next_level)
        .add_systems(
            Update,
            load_next_level
                .run_if(on_event::<KeyboardInput>.and(in_state(AppState::LoadNextLevel))),
        )
        .add_systems(OnEnter(AppState::GameOver), setup_game_over)
        .add_systems(
            Update,
            load_menu.run_if(on_event::<KeyboardInput>.and(in_state(AppState::GameOver))),
        )
        .add_systems(
            Update,
            press_space_to_start.run_if(
                in_state(GameState::NotRunning)
                    .and(in_state(AppState::Game))
                    .and(input_just_pressed(KeyCode::Space)),
            ),
        )
        .add_systems(Update, music_toggle);
}

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum GameState {
    Running,
    #[default]
    NotRunning,
    LevelComplete,
}

#[derive(Component)]
pub struct GameMusic;

#[derive(Component)]
pub struct SpaceToStart;

pub fn press_space_to_start(
    mut commands: Commands,
    mut game_state: ResMut<NextState<GameState>>,
    query: Query<Entity, With<SpaceToStart>>,
) {
    for entity in query {
        commands.entity(entity).despawn()
    }
    game_state.set(GameState::Running);
}

pub fn global_volume_set(mut volume: ResMut<GlobalVolume>) {
    info!("Set Vol");
    volume.volume = bevy::audio::Volume::Linear(0.50); // Sets global volume to 50%
}

#[derive(Resource)]
pub struct SfxMusicVolume {
    pub music: bool,
    pub sfx: bool,
}

impl Default for SfxMusicVolume {
    fn default() -> Self {
        Self {
            music: false,
            sfx: false,
        }
    }
}

pub fn toggle_music_on_click(
    _: Trigger<Pointer<Click>>,
    mut sfx_music_volume: ResMut<SfxMusicVolume>,
    mut icon: Query<&mut ImageNode, With<VolumeToggleMusicMarker>>,
) {
    sfx_music_volume.music = !sfx_music_volume.music;

    if let Ok(mut sprite) = icon.single_mut() {
        if let Some(atlas) = sprite.texture_atlas.as_mut() {
            if sfx_music_volume.music {
                atlas.index = 0;
            } else {
                atlas.index = 1;
            }
        }
    }
}

pub fn toggle_sfx_on_click(
    _: Trigger<Pointer<Click>>,
    mut sfx_music_volume: ResMut<SfxMusicVolume>,
    mut icon: Query<&mut ImageNode, With<VolumeToggleSfxMarker>>,
) {
    sfx_music_volume.sfx = !sfx_music_volume.sfx;

    if let Ok(mut sprite) = icon.single_mut() {
        if let Some(atlas) = sprite.texture_atlas.as_mut() {
            if sfx_music_volume.sfx {
                atlas.index = 0;
            } else {
                atlas.index = 1;
            }
        }
    }
}

#[derive(Component)]
pub struct VolumeToggleMarker;

#[derive(Component)]
pub struct VolumeToggleMusicMarker;

#[derive(Component)]
pub struct VolumeToggleSfxMarker;

pub fn volume_toggle_hud(
    mut commands: Commands,
    hud: Res<Hud>,
    assets: Res<ImageAssets>,
    query: Query<(), With<VolumeToggleMarker>>,
) {
    if query.iter().next().is_some() {
        return;
    }
    commands.entity(hud.0).with_children(|parent| {
        parent
            .spawn((
                VolumeToggleMarker,
                VolumeToggleMusicMarker,
                Node {
                    position_type: PositionType::Absolute,
                    display: Display::Flex,
                    flex_direction: FlexDirection::Row,
                    left: Val::Px(15.0),
                    top: Val::Px(10.0),
                    width: Val::Px(18.),
                    height: Val::Px(18.),
                    align_items: AlignItems::Center,
                    ..default()
                },
                ImageNode {
                    image: assets.volume.clone(),
                    texture_atlas: Some(TextureAtlas {
                        layout: assets.volume_layout.clone(),
                        index: 0,
                    }),
                    ..default()
                },
            ))
            .observe(toggle_music_on_click);

        parent
            .spawn((
                VolumeToggleMarker,
                VolumeToggleSfxMarker,
                Node {
                    position_type: PositionType::Absolute,
                    display: Display::Flex,
                    flex_direction: FlexDirection::Row,
                    left: Val::Px(15.0),
                    top: Val::Px(30.0),
                    width: Val::Px(18.),
                    height: Val::Px(18.),
                    align_items: AlignItems::Center,
                    ..default()
                },
                ImageNode {
                    image: assets.sfx.clone(),
                    texture_atlas: Some(TextureAtlas {
                        layout: assets.sfx_layout.clone(),
                        index: 0,
                    }),
                    ..default()
                },
            ))
            .observe(toggle_sfx_on_click);
    });
}

#[derive(Component)]
pub struct MusicVolume(pub f32);

pub fn music_toggle(
    sfx_music_volume: Res<SfxMusicVolume>,
    music: Query<(&mut AudioSink, &MusicVolume)>,
) {
    for (mut audio, music_volume) in music {
        if !sfx_music_volume.music {
            audio.set_volume(audio::Volume::Linear(0.0));
        } else {
            audio.set_volume(audio::Volume::Linear(music_volume.0));
        }
    }
}

pub fn sfx_setup(
    mut commands: Commands,
    sound_assets: Res<SoundAssets>,
    music: Query<&mut AudioSink, With<GameMusic>>,
    waiting_music_query: Query<Entity, With<WaitingMusic>>,
) {
    if music.single().is_err() {
        commands.spawn((
            GameMusic,
            MusicVolume(1.2),
            FadeInMusic::new(1.2),
            PlaybackSettings::LOOP.with_volume(bevy::audio::Volume::Linear(0.0)),
            AudioPlayer(sound_assets.music.clone()),
        ));
    }

    if let Ok(entity) = waiting_music_query.single() {
        commands.entity(entity).despawn();
    }
}

pub fn setup(
    mut loaded_level: ResMut<LoadedLevel>,
    current_level_id: Res<CurrentLevelId>,
    levels: Res<Assets<LevelLex>>,
    mut commands: Commands,
    image_assets: Res<ImageAssets>,
    hud: Res<Hud>,
    mut game_state: ResMut<NextState<GameState>>,
    mut weeds_left: ResMut<WeedTracker>,
    mut game_timer: ResMut<GameTimer>,
    mut active_key: ResMut<ActiveKey>,
    mut time_spent: ResMut<TimeSpent>,
) {
    time_spent.0.clear();
    weeds_left.reset();
    game_state.set(GameState::NotRunning);
    game_timer.0.reset();
    active_key.reset();

    let level = levels
        .into_inner()
        .iter()
        .map(|(_, data)| data)
        .next()
        .unwrap();

    let Some(level_info) = level.levels.iter().nth(current_level_id.0) else {
        commands.entity(hud.0).with_children(|parent| {
            parent
                .spawn((
                    StateScoped(AppState::Game),
                    Node {
                        position_type: PositionType::Absolute,
                        display: Display::Flex,
                        flex_direction: FlexDirection::Column,
                        width: Val::Percent(100.0),
                        top: Val::Px(210.0),
                        align_items: AlignItems::Center,
                        ..default()
                    },
                ))
                .with_children(|p| {
                    // Press this key
                    p.spawn((
                        TextColor(LIGHT_COLOR),
                        TextFont::from_font(BODY_FONT)
                            .with_font_size(RESOLUTION_HEIGHT * 6. / 8. / 25.),
                        Text("No level data found".into()),
                    ));
                });
        });
        return;
    };

    loaded_level.0 = Some(Level::new(
        level_info.letters.clone(),
        level_info.time,
        level_info.count,
        level_info.fail_count,
        level_info.id.parse().unwrap(),
    ));

    commands.entity(hud.0).with_children(|parent| {
        parent
            .spawn((
                HealthbarDisplay,
                StateScoped(AppState::Game),
                Node {
                    position_type: PositionType::Absolute,
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    top: Val::Px(15.0),
                    left: Val::Px(45.0),
                    ..default()
                },
            ))
            .with_children(|p| {
                p.spawn((
                    LettersRemainingDisplay,
                    TextColor(LIGHT_COLOR),
                    TextFont::from_font(BODY_FONT)
                        .with_font_size(RESOLUTION_HEIGHT * 6. / 8. / 25.),
                    Text(format!("Level {}", current_level_id.0 + 1)),
                ));
            });

        parent.spawn((
            HealthbarDisplay,
            StateScoped(AppState::Game),
            Node {
                position_type: PositionType::Absolute,
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                top: Val::Px(45.0),
                left: Val::Px(525.0),
                ..default()
            },
            ImageNode {
                image: image_assets.healthbar.clone(),
                texture_atlas: Some(TextureAtlas {
                    layout: image_assets.healthbar_layout.clone(),
                    index: 0,
                }),
                ..default()
            },
        ));

        parent
            .spawn((
                StateScoped(AppState::Game),
                Node {
                    position_type: PositionType::Absolute,
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    // top: Val::Px(15.0),
                    width: Val::Percent(100.0),
                    align_items: AlignItems::End,
                    padding: UiRect::all(Val::Px(15.)),
                    ..default()
                },
            ))
            .with_children(|p| {
                p.spawn((
                    LettersRemainingDisplay,
                    TextColor(LIGHT_COLOR),
                    TextFont::from_font(BODY_FONT)
                        .with_font_size(RESOLUTION_HEIGHT * 6. / 8. / 25.),
                    Text("".into()),
                ));
            });

        parent.spawn((
            StateScoped(AppState::Game),
            Node {
                position_type: PositionType::Absolute,
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                left: Val::Px(250.0),
                top: Val::Px(15.0),
                width: Val::Px(18.0),
                height: Val::Px(18.0),
                ..default()
            },
            ImageNode {
                image: image_assets.clock.clone(),
                ..default()
            },
        ));

        parent
            .spawn((
                StateScoped(AppState::Game),
                Node {
                    position_type: PositionType::Absolute,
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    width: Val::Percent(100.0),
                    top: Val::Px(15.0),
                    align_items: AlignItems::Center,
                    ..default()
                },
            ))
            .with_children(|p| {
                p.spawn((
                    Timeboard,
                    TextColor(LIGHT_COLOR),
                    TextFont::from_font(BODY_FONT)
                        .with_font_size(RESOLUTION_HEIGHT * 6. / 8. / 25.),
                    Text("".into()),
                ));
            });

        parent
            .spawn((
                StateScoped(AppState::Game),
                Node {
                    position_type: PositionType::Absolute,
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    width: Val::Percent(100.0),
                    top: Val::Px(100.0),
                    align_items: AlignItems::Center,
                    ..default()
                },
            ))
            .with_children(|p| {
                p.spawn((
                    SpaceToStart,
                    TextColor(LIGHT_COLOR),
                    TextFont::from_font(BODY_FONT)
                        .with_font_size(RESOLUTION_HEIGHT * 6. / 8. / 15.),
                    Text("Press Spacebar to Start".into()),
                ));
            });
    });

    let mut rng = rand::rng();
    // Spawn a bunch of weeds that will later start diminishing
    // as the user whacks them by pressing the correct keys

    let mut weed_x_placements = Vec::new();

    while weed_x_placements.len() < MAX_VISIBLE_WEEDS as usize {
        let candidate = if rng.random_bool(0.5) {
            rng.random_range(-RESOLUTION_WIDTH / 2.0..=-50.0)
        } else {
            rng.random_range(50.0..=RESOLUTION_WIDTH / 2.0)
        };

        if weed_x_placements
            .iter()
            .all(|&x| ((x - candidate) as f32).abs() >= MIN_PLACEMENT_DISTANCE)
        {
            weed_x_placements.push(candidate);
        }
    }

    // let weed_x_placements = (0..MAX_VISIBLE_WEEDS)
    //     .map(|_| {

    //         if rng.random_bool(0.5) {
    //             rng.random_range(-RESOLUTION_WIDTH / 2.0..=-50.0)
    //         } else {
    //             rng.random_range(50.0..=RESOLUTION_WIDTH / 2.0)
    //         }

    //     })
    //     .collect::<Vec<f32>>();

    let weeds = vec![
        image_assets.weed1.clone(),
        image_assets.weed2.clone(),
        image_assets.weed3_1.clone(),
        image_assets.weed3_2.clone(),
        image_assets.weed3_3.clone(),
        image_assets.weed4.clone(),
    ];
    for x in weed_x_placements {
        let image = weeds.choose(&mut rng).unwrap();

        let flip_x = rng.random_bool(0.5);

        commands.spawn((
            StateScoped(AppState::Game),
            Weed,
            Transform::from_translation(Vec3::new(x, -20.0, 0.0)),
            Sprite {
                flip_x: flip_x,
                image: image.clone(),
                ..default()
            },
        ));
        weeds_left.visible += 1;
    }

    weeds_left.non_visible = if level_info.count >= weeds_left.visible {
        level_info.count - weeds_left.visible
    } else {
        0
    };

    commands.spawn((
        StateScoped(AppState::Game),
        Transform::from_translation(Vec3::new(0.0, 0.0, -10.0)),
        Sprite {
            image: image_assets.farm.clone(),
            ..default()
        },
    ));
    info!(count = level_info.count, visible = weeds_left.visible);
}

#[derive(Component)]
pub struct LettersRemainingDisplay;

#[derive(Component)]
pub struct HealthbarDisplay;

#[derive(Resource, Default)]
pub struct TimeSpent(pub HashMap<usize, f32>);

#[derive(Resource, Default)]
pub struct WeedTracker {
    pub visible: u32,
    pub non_visible: u32,
}

impl WeedTracker {
    pub fn reset(&mut self) {
        *self = Self::default();
    }
    pub fn total(&self) -> u32 {
        self.non_visible + self.visible
    }
}

#[derive(Component)]
pub struct Weed;

#[derive(Component)]
pub struct ActiveKeyDisplay;

#[derive(Resource)]
pub struct GameTimer(pub Timer);

impl Default for GameTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(200., TimerMode::Once))
    }
}

#[derive(Component)]
pub struct Timeboard;

#[derive(Component)]
pub struct Transition {
    pub timer: Timer,
    pub last_frame: usize,
    pub pause_frame: usize,
}

impl Transition {
    pub fn new(last_frame: usize, pause_frame: usize) -> Self {
        Self {
            timer: Timer::from_seconds(0.1, TimerMode::Repeating),
            last_frame: last_frame,
            pause_frame: pause_frame,
        }
    }
}

#[derive(Resource, Default)]
pub struct CurrentLevelId(pub usize);

#[derive(Resource, Default)]
pub struct LoadedLevel(Option<Level>);

#[derive(Debug)]
pub struct Level {
    _id: usize,
    pub letters: Vec<String>,
    pub timer: Timer,
    pub countdown: u32,
    pub max_fail: u32,
    pub fail_count: u32,
}

impl Default for Level {
    fn default() -> Self {
        Self {
            _id: 0,
            letters: vec![],
            timer: Timer::from_seconds(5.0, TimerMode::Once),
            countdown: 0,
            fail_count: 0,
            max_fail: 0,
        }
    }
}

impl Level {
    pub fn new(letters: Vec<String>, seconds: f32, count: u32, fail_count: u32, id: usize) -> Self {
        Self {
            _id: id,
            letters: letters,
            timer: Timer::from_seconds(seconds, TimerMode::Once),
            countdown: count,
            fail_count: 0,
            max_fail: fail_count,
        }
    }
}

#[derive(Resource)]
pub struct ActiveKey(Option<(KeyCode)>);

impl Default for ActiveKey {
    fn default() -> Self {
        Self(None)
    }
}

impl ActiveKey {
    pub fn reset(&mut self) {
        self.0 = None;
    }

    pub fn set_random(&mut self, key_map: &KeyMap, level_info: &Level) -> Result<char, ()> {
        let mut rng = rand::rng();

        let chars = &level_info.letters;

        let Some(s) = chars.choose(&mut rng) else {
            warn!(?chars, "failed setting char");
            self.reset();
            return Err(());
        };

        let c = s.chars().next().unwrap();

        // let c = rng.random_range(b'a'..=b'z') as char;
        self.0 = key_map.0.get(&c).cloned();
        Ok(c)
    }
}

#[derive(Resource)]
pub struct KeyMap(HashMap<char, KeyCode>);

impl Default for KeyMap {
    fn default() -> Self {
        Self(
            [
                ('a', KeyCode::KeyA),
                ('b', KeyCode::KeyB),
                ('c', KeyCode::KeyC),
                ('d', KeyCode::KeyD),
                ('e', KeyCode::KeyE),
                ('f', KeyCode::KeyF),
                ('g', KeyCode::KeyG),
                ('h', KeyCode::KeyH),
                ('i', KeyCode::KeyI),
                ('j', KeyCode::KeyJ),
                ('k', KeyCode::KeyK),
                ('l', KeyCode::KeyL),
                ('m', KeyCode::KeyM),
                ('n', KeyCode::KeyN),
                ('o', KeyCode::KeyO),
                ('p', KeyCode::KeyP),
                ('q', KeyCode::KeyQ),
                ('r', KeyCode::KeyR),
                ('s', KeyCode::KeyS),
                ('t', KeyCode::KeyT),
                ('u', KeyCode::KeyU),
                ('v', KeyCode::KeyV),
                ('w', KeyCode::KeyW),
                ('x', KeyCode::KeyX),
                ('y', KeyCode::KeyY),
                ('z', KeyCode::KeyZ),
            ]
            .into_iter()
            .collect(),
        )
    }
}

#[derive(Resource)]
pub struct KeyPosition(HashMap<char, Vec3>);

impl Default for KeyPosition {
    fn default() -> Self {
        let key_size = 50.0;

        let x_start_middle = -RESOLUTION_WIDTH / 2. + 40.;
        let x_end_middle = RESOLUTION_WIDTH / 2. - 60.;
        let y_middle = 30.0;

        let x_start_top = -RESOLUTION_WIDTH / 2. + 25.;
        let x_end_top = RESOLUTION_WIDTH / 2. - 25.;
        let y_top = y_middle + 65.0;

        let x_start_bottom = -RESOLUTION_WIDTH / 2. + 35.;
        let x_end_bottom = RESOLUTION_WIDTH / 2. - 100.;
        let y_bottom = y_middle - 65.0;

        let map = [
            ('a', Vec2::new(x_start_middle + key_size * 0., y_middle)),
            ('s', Vec2::new(x_start_middle + key_size * 1., y_middle)),
            ('d', Vec2::new(x_start_middle + key_size * 2., y_middle)),
            ('f', Vec2::new(x_start_middle + key_size * 3., y_middle)),
            ('g', Vec2::new(x_start_middle + key_size * 4., y_middle)),
            ('h', Vec2::new(x_end_middle - key_size * 3., y_middle)),
            ('j', Vec2::new(x_end_middle - key_size * 2., y_middle)),
            ('k', Vec2::new(x_end_middle - key_size * 1., y_middle)),
            ('l', Vec2::new(x_end_middle - key_size * 0., y_middle)),
            ('q', Vec2::new(x_start_top + key_size * 0., y_top)),
            ('w', Vec2::new(x_start_top + key_size * 1., y_top)),
            ('e', Vec2::new(x_start_top + key_size * 2., y_top)),
            ('r', Vec2::new(x_start_top + key_size * 3., y_top)),
            ('t', Vec2::new(x_start_top + key_size * 4., y_top)),
            ('y', Vec2::new(x_end_top - key_size * 4., y_top)),
            ('u', Vec2::new(x_end_top - key_size * 3., y_top)),
            ('i', Vec2::new(x_end_top - key_size * 2., y_top)),
            ('o', Vec2::new(x_end_top - key_size * 1., y_top)),
            ('p', Vec2::new(x_end_top - key_size * 0., y_top)),
            ('z', Vec2::new(x_start_bottom + key_size * 0., y_bottom)),
            ('x', Vec2::new(x_start_bottom + key_size * 1., y_bottom)),
            ('c', Vec2::new(x_start_bottom + key_size * 2., y_bottom)),
            ('v', Vec2::new(x_start_bottom + key_size * 3., y_bottom)),
            ('b', Vec2::new(x_start_bottom + key_size * 4., y_bottom)),
            ('n', Vec2::new(x_end_bottom - key_size * 1., y_bottom)),
            ('m', Vec2::new(x_end_bottom - key_size * 0., y_bottom)),
        ]
        .into_iter()
        .map(|(key, value)| (key, value.extend(0.0)))
        .collect();

        Self(map)
    }
}

#[derive(Component)]
pub struct AffirmationMarker {
    pub timer: Timer,
    pub affirmation: String,
    pub pos_y: f32,
}

impl AffirmationMarker {
    pub fn new(s: impl Into<String>) -> Self {
        Self {
            timer: Timer::from_seconds(1.0, TimerMode::Once),
            affirmation: s.into(),
            pos_y: 210.0,
        }
    }
}

#[derive(Resource, Default)]
pub struct DisplayAffirmation(pub Option<String>);

impl DisplayAffirmation {
    pub fn reset(&mut self) {
        self.0 = None;
    }
}

#[derive(Resource)]
pub struct Affirmations(VecDeque<String>);

impl Default for Affirmations {
    fn default() -> Self {
        Self(
            vec![
                "That's unbeleafable!",
                "Totally radish!",
                "Soil-shaking!",
                "Full bloom fabulous!",
                "Sproutstanding!",
                "Shear brilliance!",
                "Fernomenal",
                "Plantastic",
            ]
            .iter()
            .map(|s| s.to_string())
            .collect::<VecDeque<_>>(),
        )
    }
}

pub fn level_timer_counter(
    time: Res<Time>,
    mut time_spent: ResMut<TimeSpent>,
    current_level_id: Res<CurrentLevelId>,
) {
    if let Some(timer) = time_spent.0.get_mut(&current_level_id.0) {
        *timer += time.delta_secs();
    } else {
        time_spent.0.insert(current_level_id.0, time.delta_secs());
    }
}

// Countdown the letter timer and trigger game over
pub fn level_timer_countdown(
    mut commands: Commands,
    time: Res<Time>,
    mut current_level: ResMut<LoadedLevel>,
    mut game_status: ResMut<GameStatus>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    let Some(level) = current_level.0.as_mut() else {
        return;
    };
    level.timer.tick(time.delta());

    if level.timer.finished() {
        *game_status = GameStatus::Lose;
        game_state.set(GameState::NotRunning);
        commands.send_event(SceneChange(AppState::GameOver));
    }
}

pub fn keypress_events(
    mut commands: Commands,
    mut events: EventReader<KeyboardInput>,
    mut active_key: ResMut<ActiveKey>,
    key_map: Res<KeyMap>,
    key_position: Res<KeyPosition>,
    mut current_level: ResMut<LoadedLevel>,
    mut affirmations: ResMut<Affirmations>,
    mut display_affirmation: ResMut<DisplayAffirmation>,
    image_assets: Res<ImageAssets>,
    mut letterboxes: Query<&mut Letterbox>,
    mut weeds_left: ResMut<WeedTracker>,
) {
    for event in events.read() {
        let Some(level) = current_level.0.as_mut() else {
            return;
        };

        let Some(target) = active_key.0 else {
            let level_over = weeds_left.visible == 0 || level.fail_count == level.max_fail;
            if level_over {
                info!("Go away, game is over");
                active_key.reset();
            } else {
                if let Ok(next_key) = active_key.set_random(&key_map, &level) {
                    let position = key_position
                        .0
                        .get(&next_key)
                        .cloned()
                        .unwrap_or(Vec3::new(1000.0, 1000.0, -1000.0));

                    commands.spawn((
                        StateScoped(AppState::Game),
                        Letterbox::new(next_key),
                        Transform::from_translation(position),
                        Sprite {
                            image: image_assets.letterbox.clone(),
                            texture_atlas: Some(TextureAtlas {
                                layout: image_assets.letterbox_layout.clone(),
                                index: 0,
                            }),
                            ..default()
                        },
                    ));
                    info!("rendered");
                };
            }
            return;
        };
        if event.state != ButtonState::Pressed {
            continue;
        }
        // info!(?event);

        let pressed_keycode = event.key_code;

        if target == pressed_keycode {
            for mut letterbox in letterboxes.iter_mut() {
                letterbox.state = ActiveKeyMarker::Out;
            }
            let affimation = affirmations.0.pop_front().unwrap_or("Unbeleafable!".into());
            affirmations.0.push_back(affimation.clone());
            level.timer.reset();
            if weeds_left.visible > 0 {
                commands.send_event(RemoveWeed);
            }
            if weeds_left.visible == 0 {
                // Move on to next level

                active_key.reset();
                commands.send_event(SceneChange(AppState::LoadNextLevel));
            } else {
                display_affirmation.0 = Some(affimation);
                if let Ok(next_key) = active_key.set_random(&key_map, &level) {
                    let position = key_position
                        .0
                        .get(&next_key)
                        .cloned()
                        .unwrap_or(Vec3::new(1000.0, 1000.0, -1000.0));

                    commands.spawn((
                        StateScoped(AppState::Game),
                        Letterbox::new(next_key),
                        Transform::from_translation(position),
                        Sprite {
                            image: image_assets.letterbox.clone(),
                            texture_atlas: Some(TextureAtlas {
                                layout: image_assets.letterbox_layout.clone(),
                                index: 0,
                            }),
                            ..default()
                        },
                    ));
                    info!("rendered");
                };
            }
        } else {
            //
            // render a permanent weed
            //
            commands.send_event(RemoveWeed);
            weeds_left.non_visible += 1;
            level.fail_count += 1;
            if level.fail_count >= level.max_fail {
                active_key.reset();
                info!("Lose");
                commands.send_event(SceneChange(AppState::GameOver));
            }
        }
    }
}

fn remove_weeds(
    mut commands: Commands,
    image_assets: Res<ImageAssets>,
    weeds: Query<(Entity, &Transform), With<Weed>>,
    mut weed_tracker: ResMut<WeedTracker>,
) {
    let mut rng = rand::rng();
    let i = weeds.into_iter().map(|e| e).collect::<Vec<_>>();
    let mut visible_weeds = i.len();

    if let Some((entity, transform)) = i.choose(&mut rng) {
        commands.entity(*entity).despawn();
        visible_weeds -= 1;
    }

    let current_placements = i
        .iter()
        .map(|(_, transform)| transform.translation.x)
        .collect::<Vec<_>>();

    if weed_tracker.non_visible > 0 && visible_weeds <= MAX_VISIBLE_WEEDS as usize - 1 {
        let weeds_to_place = if visible_weeds == MAX_VISIBLE_WEEDS as usize - 1 {
            1
        } else {
            if visible_weeds as u32 + weed_tracker.non_visible <= MAX_VISIBLE_WEEDS {
                weed_tracker.non_visible
            } else {
                5.min(weed_tracker.non_visible)
            }
        };

        // Spawn a bunch of weeds that will later start diminishing
        // as the user whacks them by pressing the correct keys
        for _ in 0..weeds_to_place {
            let mut x: f32;
            loop {
                x = if rng.random_bool(0.5) {
                    rng.random_range(-RESOLUTION_WIDTH / 2.0..=-50.0)
                } else {
                    rng.random_range(50.0..=RESOLUTION_WIDTH / 2.0)
                };

                if current_placements
                    .iter()
                    .all(|weed_x| (weed_x - x as f32).abs() >= MIN_PLACEMENT_DISTANCE)
                {
                    break;
                }
            }

            let weeds = vec![
                image_assets.weed1.clone(),
                image_assets.weed2.clone(),
                image_assets.weed3_1.clone(),
                image_assets.weed3_2.clone(),
                image_assets.weed3_3.clone(),
                image_assets.weed4.clone(),
            ];

            let image = weeds.choose(&mut rng).unwrap();

            let flip_x = rng.random_bool(0.5);

            commands.spawn((
                StateScoped(AppState::Game),
                Weed,
                Transform::from_translation(Vec3::new(x, -20.0, 0.0)),
                Sprite {
                    flip_x: flip_x,
                    image: image.clone(),
                    ..default()
                },
            ));

            visible_weeds += 1;
            if weed_tracker.non_visible > 0 {
                weed_tracker.non_visible -= 1;
            }
        }
    }
    weed_tracker.visible = visible_weeds as u32;
}

#[derive(Event)]
pub struct RemoveWeed;

fn animate_key(
    mut commands: Commands,
    time: Res<Time>,
    image_assets: Res<ImageAssets>,
    mut letterboxes: Query<(Entity, &mut Sprite, &mut Letterbox)>,
) {
    for (entity, mut sprite, mut letterbox) in letterboxes {
        letterbox.timer.tick(time.delta());
        if letterbox.timer.just_finished() {
            if let Some(mut atlas) = sprite.texture_atlas.as_mut() {
                match letterbox.state {
                    ActiveKeyMarker::In => {
                        if atlas.index < 4 {
                            atlas.index += 1;
                        }
                    }
                    ActiveKeyMarker::Out => {
                        if atlas.index < 8 {
                            atlas.index += 1;
                        } else {
                            commands.entity(entity).despawn();
                        }
                    }
                    ActiveKeyMarker::Hold => {
                        atlas.index = 4;
                    }
                }

                if atlas.index == 3 {
                    commands.entity(entity).with_children(|p| {
                        p.spawn(Sprite {
                            image: image_assets.letters.clone(),
                            texture_atlas: Some(TextureAtlas {
                                layout: image_assets.letters_layout.clone(),
                                index: letterbox.key_index,
                            }),
                            ..default()
                        });
                    });
                }
            }
        }
    }
}

#[derive(Component)]
pub struct Letterbox {
    pub timer: Timer,
    pub state: ActiveKeyMarker,
    pub key_index: usize,
}

impl Letterbox {
    pub fn new(c: char) -> Self {
        let idx = match c.to_digit(36) {
            Some(d) => d - 10,
            None => 0,
        };

        Self {
            timer: Timer::from_seconds(0.06, TimerMode::Repeating),
            state: ActiveKeyMarker::In,
            key_index: idx as usize,
        }
    }
}

#[derive(Component)]
pub enum ActiveKeyMarker {
    In,
    Hold,
    Out,
}

fn update_affirmation_display(
    mut commands: Commands,
    time: Res<Time>,
    hud: Res<Hud>,
    mut display_affirmation: ResMut<DisplayAffirmation>,
    mut query: Query<(Entity, &mut AffirmationMarker, &mut Node), With<AffirmationMarker>>,
) {
    let affirmation = display_affirmation.0.clone().unwrap_or_default();

    let mut skip = false;
    for (entity, mut affirmation_timer, mut node) in query {
        affirmation_timer.pos_y -= 100. * time.delta_secs();
        node.top = Val::Px(affirmation_timer.pos_y);

        affirmation_timer.timer.tick(time.delta());
        if affirmation_timer.timer.just_finished() {
            commands.entity(entity).despawn();
        }

        if affirmation_timer.affirmation == *affirmation {
            skip = true;
        }
    }

    if skip {
        return;
    }

    commands.entity(hud.0).with_children(|parent| {
        parent
            .spawn((
                StateScoped(AppState::Game),
                AffirmationMarker::new(affirmation.clone()),
                Node {
                    position_type: PositionType::Absolute,
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    width: Val::Percent(100.0),
                    top: Val::Px(210.0),
                    align_items: AlignItems::Center,
                    ..default()
                },
            ))
            .with_children(|p| {
                p.spawn((
                    TextColor(LIGHT_COLOR),
                    TextFont::from_font(BODY_FONT)
                        .with_font_size(RESOLUTION_HEIGHT * 6. / 8. / 15.),
                    Text(affirmation.to_string()),
                ));
            });
    });

    display_affirmation.reset();
}

fn update_letters_remaining_display(
    mut commands: Commands,
    time: Res<Time>,
    hud: Res<Hud>,
    mut weed_tracker: ResMut<WeedTracker>,
    mut letters_remaining: Query<(&mut Text), With<LettersRemainingDisplay>>,
) {
    let Ok(mut letters_remaining_text) = letters_remaining.single_mut() else {
        return;
    };

    letters_remaining_text.0 = format!("Left: {}", weed_tracker.total());
}

fn update_healthbar_display(
    mut current_level: ResMut<LoadedLevel>,
    mut healthbar: Query<(&mut ImageNode), With<HealthbarDisplay>>,
) {
    let Ok(mut sprite) = healthbar.single_mut() else {
        return;
    };

    let Some(level) = &current_level.0 else {
        return;
    };

    if let Some(atlas) = sprite.texture_atlas.as_mut() {
        let index = level.fail_count as usize + 1;
        if index <= 8 {
            atlas.index = index;
        } else {
            atlas.index = 0;
        }
    }
}

fn update_timeboard(
    current_level: Res<LoadedLevel>,
    mut timeboard: Query<&mut Text, With<Timeboard>>,
) {
    let Ok(mut timeboard_text) = timeboard.single_mut() else {
        return;
    };
    let Some(level) = &current_level.0 else {
        return;
    };

    timeboard_text.0 = level.timer.remaining_secs().ceil().to_string();
}

fn game_over(
    mut reader: EventReader<SceneChange>,
    mut commands: Commands,
    mut pending_scene_change: ResMut<PendingSceneChange>,
    assets: Res<ImageAssets>,
) {
    for event in reader.read() {
        let data = event.0.clone();
        pending_scene_change.0 = Some(data);
        commands.spawn((
            // BackgroundColor(BLACK.into()),
            Transition::new(13, 6),
            Node {
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                ..default()
            },
            ZIndex(99),
            ImageNode {
                image: assets.circle_transition.clone(),
                texture_atlas: Some(TextureAtlas {
                    layout: assets.circle_transition_layout.clone(),
                    index: 0,
                }),

                ..default()
            },
        ));
    }
}

#[derive(Component)]
pub struct FadeOutMusic;

#[derive(Component)]
pub struct FadeInMusic(pub bevy::audio::Volume);

impl FadeInMusic {
    pub fn new(vol: f32) -> Self {
        Self(audio::Volume::Linear(vol))
    }
}

fn fade_out_and_despawn(
    mut commands: Commands,
    music_query: Query<(Entity, &mut AudioSink), With<FadeOutMusic>>,
) {
    for (entity, mut audio_controls) in music_query {
        let current_volume = audio_controls.volume().to_linear();

        if current_volume < 0.01 {
            commands.entity(entity).despawn()
        } else {
            audio_controls.set_volume(bevy::audio::Volume::Linear(current_volume - 0.005));
        }
    }
}

fn fade_in_music(
    mut commands: Commands,
    music_query: Query<(Entity, &mut AudioSink, &FadeInMusic)>,
) {
    for (entity, mut audio_controls, fade_in_volume) in music_query {
        let current_volume = audio_controls.volume().to_linear();

        if current_volume >= fade_in_volume.0.to_linear() {
            commands.entity(entity).remove::<FadeInMusic>();
        } else {
            audio_controls.set_volume(bevy::audio::Volume::Linear(current_volume + 0.001));
        }
    }
}

fn scene_transition(
    mut commands: Commands,
    time: Res<Time>,
    pending_scene_change: Res<PendingSceneChange>,
    mut loading_state: ResMut<NextState<AppState>>,
    mut transition_ui: Query<(Entity, &mut ImageNode, &mut Transition)>,
    mut game_music: Query<Entity, (With<GameMusic>, Without<FadeOutMusic>)>,
    menu_music: Query<Entity, (With<WaitingMusic>, Without<FadeOutMusic>)>,
) {
    let Some(next_scene) = &pending_scene_change.0 else {
        return;
    };

    if *next_scene == AppState::GameOver {
        if let Ok(entity) = game_music.single_mut() {
            commands
                .entity(entity)
                .insert(FadeOutMusic)
                .remove::<MusicVolume>();
        }
    } else if *next_scene == AppState::Game {
        if let Ok(entity) = menu_music.single() {
            commands
                .entity(entity)
                .insert(FadeOutMusic)
                .remove::<MusicVolume>();
        }
    }

    for (entity, mut sprite, mut transition) in transition_ui.iter_mut() {
        transition.timer.tick(time.delta());

        if transition.timer.just_finished() {
            if let Some(atlas) = sprite.texture_atlas.as_mut() {
                if atlas.index == transition.pause_frame - 1 {
                    atlas.index += 1;
                    loading_state.set(next_scene.clone());
                } else if atlas.index < transition.last_frame {
                    atlas.index += 1;
                } else {
                    commands.entity(entity).despawn();
                }
            }
        }
    }
}

#[derive(Component)]
pub struct WaitingMusic;

fn waiting_music(
    mut commands: Commands,

    sound_assets: Res<SoundAssets>,
    music: Query<(), With<WaitingMusic>>,
) {
    if music.single().is_err() {
        commands.spawn((
            WaitingMusic,
            MusicVolume(0.25),
            FadeInMusic::new(0.25),
            PlaybackSettings::LOOP.with_volume(bevy::audio::Volume::Linear(0.0)),
            AudioPlayer(sound_assets.menu_music.clone()),
        ));
    }
}

fn spacer() -> impl Bundle {
    (
        TextColor(LIGHT_COLOR),
        TextFont::from_font(BODY_FONT).with_font_size(RESOLUTION_HEIGHT * 6. / 8. / 25.),
        Text("\n".into()),
    )
}

const NORMAL_BUTTON: Color = LIGHT_COLOR;
const HOVERED_BUTTON: Color = LIGHT_COLOR;
const PRESSED_BUTTON: Color = DARK_COLOR;

fn button(text: String) -> impl Bundle + use<> {
    (
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        children![(
            Button,
            Node {
                width: Val::Px(150.0),
                height: Val::Px(65.0),
                border: UiRect::all(Val::Px(5.0)),
                // horizontally center child text
                justify_content: JustifyContent::Center,
                // vertically center child text
                align_items: AlignItems::Center,
                ..default()
            },
            BorderColor(DARK_COLOR),
            BorderRadius::MAX,
            BackgroundColor(NORMAL_BUTTON),
            children![(
                Text::new(text),
                TextFont::from_font(BODY_FONT).with_font_size(RESOLUTION_HEIGHT * 6. / 8. / 25.),
                TextColor(LIGHT_COLOR),
            )]
        )],
    )
}

fn button_system(
    mut interaction_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            &mut BorderColor,
            &Children,
        ),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut color, mut border_color, _children) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = PRESSED_BUTTON.into();
                border_color.0 = LIGHT_COLOR;
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
                border_color.0 = LIGHT_COLOR;
            }
            Interaction::None => {
                *color = NORMAL_BUTTON.into();
                border_color.0 = DARK_COLOR;
            }
        }
    }
}

pub fn go_to_menu(_: Trigger<Pointer<Click>>, mut commands: Commands) {
    commands.send_event(SceneChange(AppState::Menu));
}

fn setup_credits(mut commands: Commands, hud: Res<Hud>) {
    commands.entity(hud.0).with_children(|parent| {
        parent
            .spawn((
                StateScoped(AppState::Credits),
                Node {
                    position_type: PositionType::Absolute,
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    width: Val::Percent(100.0),
                    top: Val::Px(55.0),
                    left: Val::Px(120.0),
                    align_items: AlignItems::Start,
                    ..default()
                },
            ))
            .with_children(|p| {
                p.spawn((
                    TextColor(LIGHT_COLOR),
                    TextFont::from_font(BODY_FONT)
                        .with_font_size(RESOLUTION_HEIGHT * 6. / 8. / 25.),
                    Text("   Credits\n----------------\n".into()),
                ));
                p.spawn(spacer());
                p.spawn((
                    TextColor(LIGHT_COLOR),
                    TextFont::from_font(BODY_FONT)
                        .with_font_size(RESOLUTION_HEIGHT * 6. / 8. / 25.),
                    Text("Software Development\n--------------------\nIsa Aguilar\n\n".into()),
                ));
                p.spawn(spacer());
                p.spawn((
                    TextColor(LIGHT_COLOR),
                    TextFont::from_font(BODY_FONT)
                        .with_font_size(RESOLUTION_HEIGHT * 6. / 8. / 25.),
                    Text("Artwork\n------\nIsa Aguilar\n\n".into()),
                ));
                p.spawn(spacer());
                p.spawn((
                    TextColor(LIGHT_COLOR),
                    TextFont::from_font(BODY_FONT)
                        .with_font_size(RESOLUTION_HEIGHT * 6. / 8. / 25.),
                    Text("Music\n-----\nIsa Aguilar\n\n".into()),
                ));
            });

        parent
            .spawn((
                StateScoped(AppState::Credits),
                Node {
                    position_type: PositionType::Absolute,
                    height: Val::Px(2.0 * 480. - 100.),
                    width: Val::Px(2.0 * 600. - 200.),

                    ..default()
                },
            ))
            .with_children(|p| {
                p.spawn(button("Menu".into()));
            })
            .observe(go_to_menu);
    });
}

fn setup_load_next_level(
    mut commands: Commands,
    hud: Res<Hud>,
    mut game_state: ResMut<NextState<GameState>>,
    time_spent: Res<TimeSpent>,
    current_level_id: Res<CurrentLevelId>,
    loaded_level: Res<LoadedLevel>,
) {
    game_state.set(GameState::NotRunning);

    let mut score = 0;
    let time_spent_text = match time_spent.0.get(&current_level_id.0) {
        Some(t) => {
            score += (t * 1000.0).round() as u32;
            format!("Time taken: {}s", (t * 1000.).round() / 1000.0)
        }
        None => String::new(),
    };

    let wrong_key_press_count_text = match &loaded_level.0 {
        Some(level) => {
            score += level.fail_count * 1000;
            format!("Wrong keys pressed: {}", level.fail_count)
        }
        None => String::new(),
    };

    let score_text = format!("Your score is: {}\n\n(The lower the better)", score);

    commands.entity(hud.0).with_children(|parent| {
        parent
            .spawn((
                StateScoped(AppState::LoadNextLevel),
                Node {
                    position_type: PositionType::Absolute,
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    width: Val::Percent(100.0),
                    top: Val::Px(55.0),
                    left: Val::Px(120.0),
                    align_items: AlignItems::Start,
                    ..default()
                },
            ))
            .with_children(|p| {
                p.spawn((
                    TextColor(LIGHT_COLOR),
                    TextFont::from_font(BODY_FONT)
                        .with_font_size(RESOLUTION_HEIGHT * 6. / 8. / 25.),
                    Text("Great Job!".into()),
                ));
                p.spawn(spacer());
                p.spawn((
                    TextColor(LIGHT_COLOR),
                    TextFont::from_font(BODY_FONT)
                        .with_font_size(RESOLUTION_HEIGHT * 6. / 8. / 25.),
                    Text("Press any key to continue".into()),
                ));
                p.spawn(spacer());
                p.spawn((
                    TextColor(LIGHT_COLOR),
                    TextFont::from_font(BODY_FONT)
                        .with_font_size(RESOLUTION_HEIGHT * 6. / 8. / 25.),
                    Text(time_spent_text),
                ));
                p.spawn(spacer());
                p.spawn((
                    TextColor(LIGHT_COLOR),
                    TextFont::from_font(BODY_FONT)
                        .with_font_size(RESOLUTION_HEIGHT * 6. / 8. / 25.),
                    Text(wrong_key_press_count_text),
                ));
                p.spawn(spacer());
                p.spawn((
                    TextColor(LIGHT_COLOR),
                    TextFont::from_font(BODY_FONT)
                        .with_font_size(RESOLUTION_HEIGHT * 6. / 8. / 25.),
                    Text("---------------".into()),
                ));
                p.spawn(spacer());
                p.spawn((
                    TextColor(LIGHT_COLOR),
                    TextFont::from_font(BODY_FONT)
                        .with_font_size(RESOLUTION_HEIGHT * 6. / 8. / 25.),
                    Text(score_text),
                ));
            });
    });
}

fn setup_game_over(
    mut commands: Commands,
    hud: Res<Hud>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    game_state.set(GameState::NotRunning);
    commands.entity(hud.0).with_children(|parent| {
        parent
            .spawn((
                StateScoped(AppState::GameOver),
                Node {
                    position_type: PositionType::Absolute,
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    width: Val::Percent(100.0),
                    top: Val::Px(55.0),
                    left: Val::Px(120.0),
                    align_items: AlignItems::Start,
                    ..default()
                },
            ))
            .with_children(|p| {
                p.spawn((
                    TextColor(LIGHT_COLOR),
                    TextFont::from_font(BODY_FONT)
                        .with_font_size(RESOLUTION_HEIGHT * 6. / 8. / 15.),
                    Text("Oh no!".into()),
                ));
                p.spawn(spacer());
                p.spawn((
                    TextColor(LIGHT_COLOR),
                    TextFont::from_font(BODY_FONT)
                        .with_font_size(RESOLUTION_HEIGHT * 6. / 8. / 25.),
                    Text("The weeds took over the garden".into()),
                ));
                p.spawn((
                    TextColor(LIGHT_COLOR),
                    TextFont::from_font(BODY_FONT)
                        .with_font_size(RESOLUTION_HEIGHT * 6. / 8. / 25.),
                    Text("Press any key to continue".into()),
                ));
            });
    });
}

fn load_next_level(
    mut events: EventReader<KeyboardInput>,
    mut current_level_id: ResMut<CurrentLevelId>,
    mut commands: Commands,
) {
    for event in events.read() {
        if event.state != ButtonState::Released {
            current_level_id.0 += 1;
            commands.send_event(SceneChange(AppState::Game));
        }
    }
}

fn load_menu(
    mut events: EventReader<KeyboardInput>,
    mut current_level_id: ResMut<CurrentLevelId>,
    mut commands: Commands,
) {
    for event in events.read() {
        if event.state != ButtonState::Released {
            current_level_id.0 = 0;
            commands.send_event(SceneChange(AppState::Menu));
        }
    }
}

#[derive(Resource, Default, Eq, PartialEq)]
pub enum GameStatus {
    #[default]
    InProgress,
    Lose,
    Win,
}

impl GameStatus {
    fn won(&self) -> bool {
        *self == GameStatus::Win
    }

    fn lost(&self) -> bool {
        *self == GameStatus::Lose
    }
}

#[derive(Event)]
pub struct SceneChange(pub AppState);

#[derive(Resource, Default)]
pub struct PendingSceneChange(pub Option<AppState>);
