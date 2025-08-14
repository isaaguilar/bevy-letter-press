use crate::{
    app::AppState,
    assets::lexi::{LexiCollection, Lexicon},
};
use bevy::prelude::*;
use bevy_common_assets::json::JsonAssetPlugin;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(JsonAssetPlugin::<LevelLex>::new(&[".json"]));
    app.add_systems(OnEnter(AppState::Preload), preload);
}

fn preload(mut commands: Commands, asset_server: Res<AssetServer>) {
    info!("Loading level data");
    commands.insert_resource(LexiCollection::<LevelLex>::new(
        &asset_server,
        vec![
            "lexi/levels/levels.json",
            // ...more level things here,
        ],
    ));
}

#[derive(serde::Deserialize, Asset, TypePath, Debug, Default, Clone)]
pub struct LevelLex {
    pub levels: Vec<LevelInfo>,
}

#[derive(serde::Deserialize, Asset, TypePath, Debug, Default, Clone)]
pub struct LevelInfo {
    pub id: String,
    pub letters: Vec<String>,
    pub time: f32,
    pub count: u32,
    pub fail_count: u32,
}
