use bevy::{asset::AssetMetaCheck, prelude::*, window::WindowResolution};
use bevy_aspect_ratio_mask::{AspectRatioMask, AspectRatioPlugin, Resolution};

const TITLE: &str = "Whack-A-Weed";

pub const RESOLUTION_WIDTH: f32 = 640.0;
pub const RESOLUTION_HEIGHT: f32 = 480.0;
pub const AFTER_LOADING_STATE: AppState = AppState::Menu;
pub const RUNNING_SPEED: f32 = 250.0;

use crate::{assets, game, leaderboard, menu, util};

pub const DARK_COLOR: Color = Color::srgb(24. / 255., 51. / 255., 5. / 255.);
pub const LIGHT_COLOR: Color = Color::srgb(214. / 255., 227. / 255., 195. / 255.);
pub const CLEAR_COLOR: Color = Color::srgba(0.0, 0.0, 0.0, 0.0);

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
#[states(scoped_entities)]
pub enum AppState {
    #[default]
    Preload,
    Loading,
    LeaderboardSelection,
    Leaderboard,
    Menu,
    Game,
    GameOver,
    HighScores,
    Credits,
    LoadNextLevel,
}

pub fn start() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: TITLE.into(),
                        name: Some(TITLE.into()),
                        resolution: WindowResolution::new(RESOLUTION_WIDTH, RESOLUTION_HEIGHT),
                        fit_canvas_to_parent: true,

                        ..default()
                    }),
                    ..default()
                })
                .set(AssetPlugin {
                    meta_check: AssetMetaCheck::Never,
                    ..default()
                }),
        )
        .insert_resource(ClearColor(DARK_COLOR))
        .init_state::<AppState>()
        .insert_resource(DisplayLanguage::default())
        .insert_resource(InteractionRateLimit::default())
        .add_plugins((
            AspectRatioPlugin {
                resolution: Resolution {
                    width: RESOLUTION_WIDTH,
                    height: RESOLUTION_HEIGHT,
                },
                mask: AspectRatioMask {
                    color: Color::srgba(24. / 255., 51. / 255., 5. / 255., 0.0),
                },
                ..default()
            },
            menu::Menu,
            assets::plugin,
            game::plugin,
            leaderboard::plugin,
            util::plugin,
            #[cfg(feature = "dev")]
            crate::dev_tools::plugin,
        ))
        .run();
}

#[derive(Resource, Deref, DerefMut)]
pub struct DisplayLanguage(pub String);

impl DisplayLanguage {
    fn default() -> Self {
        Self("english".into())
    }
}

#[derive(Component)]
pub struct DialogDisplay(pub String);

#[derive(Resource)]
pub struct InteractionRateLimit(pub Timer);

impl Default for InteractionRateLimit {
    fn default() -> Self {
        Self(Timer::from_seconds(0.20, TimerMode::Once))
    }
}
