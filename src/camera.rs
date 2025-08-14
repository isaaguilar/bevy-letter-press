use crate::app::{RESOLUTION_HEIGHT, RESOLUTION_WIDTH};

use bevy::prelude::*;
use bevy::render::camera::ScalingMode;

#[derive(Component, Default)]
pub struct GameCamera {
    pub selected_game_level: GameLevelDimensions,
}

#[derive(Component, Default)]
pub struct GameLevelDimensions {
    left: f32,
    right: f32,
    top: f32,
    bottom: f32,
}

pub fn game_camera(
    mut commands: Commands,
    mut camera_query: Query<&mut Transform, With<GameCamera>>,
) {
    if let Ok(_) = camera_query.single_mut() {
        return;
    }

    commands
        .spawn((
            GameCamera {
                selected_game_level: GameLevelDimensions {
                    left: -1000000.,
                    top: 1000000.,
                    right: 1000000.,
                    bottom: -1000000.,
                },
                ..default()
            },
            Camera2d::default(),
            Projection::from(OrthographicProjection {
                scaling_mode: ScalingMode::AutoMin {
                    min_width: RESOLUTION_WIDTH,
                    min_height: RESOLUTION_HEIGHT,
                },
                scale: 1.0,
                near: -1000.,
                far: 1000.,
                ..OrthographicProjection::default_2d()
            }),
        ))
        .insert(Transform::from_xyz(0., 0., 0.));
}
