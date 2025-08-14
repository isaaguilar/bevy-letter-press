use std::collections::VecDeque;

use crate::app::{AppState, DisplayLanguage, RESOLUTION_HEIGHT, RESOLUTION_WIDTH, RUNNING_SPEED};
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

pub(super) fn plugin(app: &mut App) {
    app.init_state::<GameState>()
        .add_event::<SceneChange>()
        .add_plugins((TextInputPlugin, HttpClientPlugin))
        .insert_resource(CurrentLevel::default())
        .insert_resource(GameTimer::default())
        .insert_resource(GameStatus::default())
        .insert_resource(PendingSceneChange::default())
        .insert_resource(SfxMusicVolume::default())
        .insert_resource(ActiveKey::default())
        .insert_resource(Affirmations::default())
        .insert_resource(DisplayAffirmation::default())
        .insert_resource(KeyMap::default())
        .add_systems(Startup, global_volume_set)
        .add_systems(OnEnter(AppState::Game), (sfx_setup, setup))
        .add_systems(Startup, camera::game_camera)
        .add_systems(Update, keypress_events.run_if(on_event::<KeyboardInput>))
        .add_systems(
            Update,
            (
                level_timer_countdown,
                update_timeboard,
                update_active_key_display,
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
                    top: Val::Px(410.0),
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
                    top: Val::Px(440.0),
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
    mut current_level: ResMut<CurrentLevel>,
    levels: Res<Assets<LevelLex>>,
    mut commands: Commands,
    assets: Res<ImageAssets>,
    hud: Res<Hud>,
    mut game_state: ResMut<NextState<GameState>>,

    mut game_timer: ResMut<GameTimer>,
) {
    let level = levels
        .into_inner()
        .iter()
        .map(|(_, data)| data)
        .next()
        .unwrap();

    let Some(level_info) = level.levels.iter().nth(0) else {
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
                        TextFont::from_font(BODY_FONT)
                            .with_font_size(RESOLUTION_HEIGHT * 6. / 8. / 25.),
                        Text("No level data found".into()),
                    ));
                });
        });
        return;
    };

    current_level.0 = Some(Level::new(
        level_info.letters.clone(),
        level_info.time,
        level_info.count,
        level_info.fail_count,
        level_info.id.parse().unwrap(),
    ));

    game_state.set(GameState::NotRunning);
    game_timer.0.reset();

    commands.entity(hud.0).with_children(|parent| {
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
                image: assets.clock.clone(),
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
                    TextFont::from_font(BODY_FONT)
                        .with_font_size(RESOLUTION_HEIGHT * 6. / 8. / 25.),
                    Text("".into()),
                ));

                // Press this key
                p.spawn((
                    ActiveKeyDisplay,
                    TextFont::from_font(BODY_FONT)
                        .with_font_size(RESOLUTION_HEIGHT * 6. / 8. / 25.),
                    Text("".into()),
                ));
            });
    });
}

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
pub struct CurrentLevel(Option<Level>);

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

    pub fn set_random(&mut self, key_map: &KeyMap, level_info: &Level) {
        let mut rng = rand::rng();

        let chars = &level_info.letters;

        let Some(s) = chars.choose(&mut rng) else {
            warn!(?chars, "failed setting char");
            self.reset();
            return;
        };

        let c = s.chars().next().unwrap();

        // let c = rng.random_range(b'a'..=b'z') as char;
        self.0 = key_map.0.get(&c).cloned();
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

// Countdown the letter timer and trigger game over
pub fn level_timer_countdown(
    mut commands: Commands,
    time: Res<Time>,
    mut current_level: ResMut<CurrentLevel>,
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
    mut current_level: ResMut<CurrentLevel>,
    mut affirmations: ResMut<Affirmations>,
    mut display_affirmation: ResMut<DisplayAffirmation>,
) {
    for event in events.read() {
        let Some(level) = current_level.0.as_mut() else {
            return;
        };

        let Some(target) = active_key.0 else {
            let level_over = level.countdown == 0 || level.fail_count == level.max_fail;
            if level_over {
                info!("Go away, game is over");
                active_key.reset();
            } else {
                active_key.set_random(&key_map, &level);
            }
            return;
        };
        if event.state != ButtonState::Pressed {
            continue;
        }
        // info!(?event);

        let pressed_keycode = event.key_code;

        if target == pressed_keycode {
            let affimation = affirmations.0.pop_front().unwrap();
            level.timer.reset();
            if level.countdown > 0 {
                level.countdown -= 1;
            }
            if level.countdown == 0 {
                // Move on to next level

                active_key.reset();
                commands.send_event(SceneChange(AppState::LoadNextLevel));
            } else {
                display_affirmation.0 = Some(affimation.clone());
                affirmations.0.push_back(affimation);
                active_key.set_random(&key_map, &level);
            }
        } else {
            level.fail_count += 1;
            if level.fail_count >= level.max_fail {
                active_key.reset();
                info!("Lose");
            }
        }
    }
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
                    TextFont::from_font(BODY_FONT)
                        .with_font_size(RESOLUTION_HEIGHT * 6. / 8. / 15.),
                    Text(affirmation.to_string()),
                ));
            });
    });

    display_affirmation.reset();
}

fn update_active_key_display(
    active_key: Res<ActiveKey>,
    key_map: Res<KeyMap>,
    mut timeboard: Query<&mut Text, With<ActiveKeyDisplay>>,
) {
    let Ok(mut active_key_display_text) = timeboard.single_mut() else {
        return;
    };

    if let Some(keycode) = active_key.0 {
        if let Some(ch) = key_map
            .0
            .iter()
            .find_map(|(ch, kc)| (*kc == keycode).then_some(ch))
        {
            active_key_display_text.0 = ch.to_string();
        }
    } else {
        active_key_display_text.0 = "".into();
    }
}

fn update_timeboard(
    current_level: Res<CurrentLevel>,
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
        TextFont::from_font(BODY_FONT).with_font_size(RESOLUTION_HEIGHT * 6. / 8. / 25.),
        Text("\n".into()),
    )
}

const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);

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
            BorderColor(Color::BLACK),
            BorderRadius::MAX,
            BackgroundColor(NORMAL_BUTTON),
            children![(
                Text::new(text),
                TextFont::from_font(BODY_FONT).with_font_size(RESOLUTION_HEIGHT * 6. / 8. / 25.),
                TextColor(Color::srgb(0.9, 0.9, 0.9)),
                TextShadow::default(),
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
                border_color.0 = bevy::color::palettes::css::RED.into();
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
                border_color.0 = Color::WHITE;
            }
            Interaction::None => {
                *color = NORMAL_BUTTON.into();
                border_color.0 = Color::BLACK;
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
                    TextFont::from_font(BODY_FONT)
                        .with_font_size(RESOLUTION_HEIGHT * 6. / 8. / 25.),
                    Text("   Credits\n----------------\n".into()),
                ));
                p.spawn(spacer());
                p.spawn((
                    TextFont::from_font(BODY_FONT)
                        .with_font_size(RESOLUTION_HEIGHT * 6. / 8. / 25.),
                    Text("Software Development\n--------------------\nIsa Aguilar\n\n".into()),
                ));
                p.spawn(spacer());
                p.spawn((
                    TextFont::from_font(BODY_FONT)
                        .with_font_size(RESOLUTION_HEIGHT * 6. / 8. / 25.),
                    Text("Artwork\n------\nIsa Aguilar\n\n".into()),
                ));
                p.spawn(spacer());
                p.spawn((
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
