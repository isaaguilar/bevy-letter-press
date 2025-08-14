use crate::app::AppState;

use crate::game::GameState;
use crate::game::GameStatus;

use crate::game::SceneChange;

use bevy::dev_tools::states::log_transitions;
use bevy::input::common_conditions::input_just_pressed;
use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, log_transitions::<AppState>)
        .add_systems(Update, restart)
        .add_systems(
            PostUpdate,
            draw_aabb_gizmos.run_if(in_state(AppState::Game)),
        );
}

fn lose(
    mut commands: Commands,
    mut game_status: ResMut<GameStatus>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    *game_status = GameStatus::Lose;
    game_state.set(GameState::NotRunning);
    commands.send_event(SceneChange(AppState::GameOver));
}

fn win(
    mut commands: Commands,
    mut game_status: ResMut<GameStatus>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    *game_status = GameStatus::Win;
    game_state.set(GameState::NotRunning);
    commands.send_event(SceneChange(AppState::GameOver));
}

fn restart(
    mut loading_state: ResMut<NextState<AppState>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    if keyboard_input.just_pressed(KeyCode::F1) {
        loading_state.set(AppState::Game);
    }
}

pub fn draw_aabb_gizmos(mut gizmos: Gizmos) {
    //
}
