use crate::app::*;
use crate::assets::custom::ImageAssets;
use crate::game::PlayerScore;
use crate::menu::LeaderboardName;
use crate::menu::layouts::button_text;
use crate::util::handles::BODY_FONT;
use crate::{app::AppState, game::SceneChange};
use bevy::prelude::*;
use bevy_aspect_ratio_mask::Hud;
use bevy_http_client::{HttpClient, HttpRequest, HttpResponse, HttpResponseError};
use serde::Deserialize;

pub const LEADERBOARD_URL: &'static str = env!("LEADERBOARD_URL");

pub(super) fn plugin(app: &mut App) {
    app.add_event::<GetHighScore>()
        .add_event::<PostHighScore>()
        .insert_resource(LeaderboardLevelSelected::default())
        .insert_resource(HighScores::default())
        .add_systems(
            OnEnter(AppState::LeaderboardSelection),
            setup_leaderboard_selection,
        )
        .add_systems(Update, get_high_score.run_if(on_event::<GetHighScore>))
        .add_systems(Update, post_high_score.run_if(on_event::<PostHighScore>))
        .add_systems(Update, (handle_response, handle_error))
        .add_systems(OnEnter(AppState::Leaderboard), setup_leaderboard)
        .add_systems(
            Update,
            (update_high_scoreboard).run_if(in_state(AppState::Leaderboard)),
        );
}

#[derive(Resource, Default)]
pub struct LeaderboardLevelSelected(pub Option<usize>);

#[derive(Component)]
pub struct LeaderboardLevel(pub usize);

fn setup_leaderboard_selection(
    mut commands: Commands,
    hud: Res<Hud>,
    image_assets: Res<ImageAssets>,
) {
    commands.entity(hud.0).with_children(|parent| {
        parent
            .spawn((
                StateScoped(AppState::LeaderboardSelection),
                Node {
                    position_type: PositionType::Absolute,
                    display: Display::Flex,
                    justify_self: JustifySelf::Center,
                    flex_direction: FlexDirection::Column,
                    width: Val::Px(100.0),

                    border: UiRect::all(Val::Px(2.0)),
                    top: Val::Px(425.0),
                    left: Val::Px(500.0),
                    ..default()
                },
                BorderColor(LIGHT_COLOR),
                BorderRadius::MAX,
            ))
            .with_children(|p| {
                p.spawn(
                    ((
                        Node {
                            width: Val::Percent(100.0),
                            ..default()
                        },
                        BorderRadius::MAX,
                        Pickable::default(),
                        Text::default(),
                        BackgroundColor(DARK_COLOR),
                        TextLayout::default().with_justify(JustifyText::Center),
                        children![(
                            TextColor(LIGHT_COLOR),
                            TextFont::from_font(BODY_FONT)
                                .with_font_size(RESOLUTION_HEIGHT * 6. / 8. / 30.)
                                .with_line_height(bevy::text::LineHeight::RelativeToFont(2.5)),
                            Pickable::IGNORE,
                            TextSpan::new(format!("Back")),
                        )],
                    )),
                )
                .observe(back_to_menu);
            });

        parent
            .spawn((
                StateScoped(AppState::LeaderboardSelection),
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
                    Text("Select a level to view leaders:".into()),
                ));
            });

        parent
            .spawn((
                StateScoped(AppState::LeaderboardSelection),
                Node {
                    position_type: PositionType::Absolute,
                    display: Display::Flex,
                    flex_direction: FlexDirection::Row,
                    width: Val::Percent(100.0),
                    top: Val::Px(100.0),
                    left: Val::Px(120.0),
                    ..default()
                },
            ))
            .with_children(|p| {
                for n in 0..6 {
                    let lvl = (n + 1).to_string();
                    p.spawn((
                        LeaderboardLevel(n),
                        Node {
                            width: Val::Px(64.0),
                            height: Val::Px(64.0),
                            align_content: AlignContent::Center,
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            justify_items: JustifyItems::Center,
                            ..default()
                        },
                        ImageNode {
                            image: image_assets.leaves.clone(),
                            texture_atlas: Some(TextureAtlas {
                                layout: image_assets.leaves_layout.clone(),
                                index: n,
                            }),
                            ..default()
                        },
                    ))
                    .with_child((
                        TextColor(LIGHT_COLOR),
                        TextShadow {
                            offset: Vec2::splat(3.0),
                            color: DARK_COLOR,
                        },
                        TextFont::from_font(BODY_FONT)
                            .with_font_size(RESOLUTION_HEIGHT * 6. / 8. / 14.)
                            .with_line_height(bevy::text::LineHeight::RelativeToFont(2.5)),
                        Pickable::IGNORE,
                        Text::new(lvl),
                    ))
                    .observe(set_leaderboard_level_on_click);
                }
            });

        parent
            .spawn((
                StateScoped(AppState::LeaderboardSelection),
                Node {
                    position_type: PositionType::Absolute,
                    display: Display::Flex,
                    flex_direction: FlexDirection::Row,
                    width: Val::Percent(100.0),
                    top: Val::Px(200.0),
                    left: Val::Px(120.0),

                    ..default()
                },
            ))
            .with_children(|p| {
                for n in 6..12 {
                    let lvl = (n + 1).to_string();
                    p.spawn((
                        LeaderboardLevel(n),
                        Node {
                            width: Val::Px(64.0),
                            height: Val::Px(64.0),
                            align_content: AlignContent::Center,
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            justify_items: JustifyItems::Center,
                            ..default()
                        },
                        ImageNode {
                            image: image_assets.leaves.clone(),
                            texture_atlas: Some(TextureAtlas {
                                layout: image_assets.leaves_layout.clone(),
                                index: n,
                            }),
                            ..default()
                        },
                    ))
                    .with_child((
                        TextColor(LIGHT_COLOR),
                        TextShadow {
                            offset: Vec2::splat(3.0),
                            color: DARK_COLOR,
                        },
                        TextFont::from_font(BODY_FONT)
                            .with_font_size(RESOLUTION_HEIGHT * 6. / 8. / 14.)
                            .with_line_height(bevy::text::LineHeight::RelativeToFont(2.5)),
                        Pickable::IGNORE,
                        Text::new(lvl),
                    ))
                    .observe(set_leaderboard_level_on_click);
                }
            });

        parent
            .spawn((
                StateScoped(AppState::LeaderboardSelection),
                Node {
                    position_type: PositionType::Absolute,
                    display: Display::Flex,
                    flex_direction: FlexDirection::Row,
                    width: Val::Percent(100.0),
                    top: Val::Px(300.0),
                    left: Val::Px(280.0),

                    ..default()
                },
            ))
            .with_children(|p| {
                let n = 12;
                let lvl = (n + 1).to_string();
                p.spawn((
                    LeaderboardLevel(n),
                    Node {
                        width: Val::Px(64.0),
                        height: Val::Px(64.0),
                        align_content: AlignContent::Center,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        justify_items: JustifyItems::Center,
                        ..default()
                    },
                    ImageNode {
                        image: image_assets.leaves.clone(),
                        texture_atlas: Some(TextureAtlas {
                            layout: image_assets.leaves_layout.clone(),
                            index: n,
                        }),
                        ..default()
                    },
                ))
                .with_child((
                    TextColor(LIGHT_COLOR),
                    TextShadow {
                        offset: Vec2::splat(3.0),
                        color: DARK_COLOR,
                    },
                    TextFont::from_font(BODY_FONT)
                        .with_font_size(RESOLUTION_HEIGHT * 6. / 8. / 14.)
                        .with_line_height(bevy::text::LineHeight::RelativeToFont(2.5)),
                    Pickable::IGNORE,
                    Text::new(lvl),
                ))
                .observe(set_leaderboard_level_on_click);
            });
    });
}

fn spacer() -> impl Bundle {
    (
        TextColor(LIGHT_COLOR),
        TextFont::from_font(BODY_FONT).with_font_size(RESOLUTION_HEIGHT * 6. / 8. / 25.),
        Text("\n".into()),
    )
}

pub fn set_leaderboard_level_on_click(
    trigger: Trigger<Pointer<Click>>,
    mut commands: Commands,
    mut button: Query<&LeaderboardLevel>,
    mut leaderboard_level_selected: ResMut<LeaderboardLevelSelected>,
) {
    if let Ok(selection) = button.get_mut(trigger.target) {
        leaderboard_level_selected.0 = Some(selection.0.clone());

        commands.send_event(GetHighScore);
    };
}

#[derive(Event)]
pub struct GetHighScore;

#[derive(Event)]
pub struct PostHighScore;

#[derive(Deserialize, Debug)]
pub struct LeaderboardOutput {
    leaderboard: Vec<HighScoreData>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct HighScoreData {
    name: String,
    score: u32,
    level: usize,
}

#[derive(Component)]
pub struct HighScoreboard;

#[derive(Component)]
pub struct HighScoreboardTopFive;

#[derive(Component)]
pub struct HighScoreboardLevel;

#[derive(Event)]
pub struct RenderHighScores;

#[derive(Resource, Default, Debug)]
pub struct HighScores(pub Vec<HighScoreData>);

pub fn post_high_score(
    mut commands: Commands,
    mut ev_request: EventWriter<HttpRequest>,
    leaderboard_name: Res<LeaderboardName>,
    leaderboard_level_selected: Res<LeaderboardLevelSelected>,
    player_score: Res<PlayerScore>,
) {
    info!("posting high score");

    // The only requirement in this function is the level id. At minimum, this function
    // will fetch the latest scores. If a name and score are provided, the called
    // function will process this and add the new scores to teh database, and then
    // return the latest scores.
    let Some(level_id) = leaderboard_level_selected.0 else {
        warn!("No level_id");
        return;
    };

    let name = match &leaderboard_name.0 {
        Some(name) => name.clone(),
        None => String::new(),
    };

    let score = match player_score.0.get(&level_id) {
        Some(s) => *s,
        None => 0,
    };

    let client = HttpClient::new();
    match client
        .post(LEADERBOARD_URL)
        .json(&serde_json::json!({"level": level_id, "name": name, "score": score, }))
        .try_build()
    {
        Ok(request) => {
            ev_request.write(request);
        }
        Err(e) => error!(?e),
    }
}

pub fn get_high_score(
    mut commands: Commands,
    mut ev_request: EventWriter<HttpRequest>,
    leaderboard_level_selected: Res<LeaderboardLevelSelected>,
) {
    // The only requirement in this function is the level id. At minimum, this function
    // will fetch the latest scores. If a name and score are provided, the called
    // function will process this and add the new scores to teh database, and then
    // return the latest scores.
    let Some(level_id) = leaderboard_level_selected.0 else {
        warn!("No level_id");
        return;
    };

    let client = HttpClient::new();
    match client
        .post(LEADERBOARD_URL)
        .json(&serde_json::json!({"level": level_id, "name": "", "score": 0, }))
        .try_build()
    {
        Ok(request) => {
            ev_request.write(request);
        }
        Err(e) => error!(?e),
    }

    commands.send_event(SceneChange(AppState::Leaderboard));
}

fn handle_response(
    mut ev_resp: EventReader<HttpResponse>,
    mut high_score_data: ResMut<HighScores>,
) {
    for response in ev_resp.read() {
        if let Ok(data) = response.json::<LeaderboardOutput>() {
            let high_scores = data.leaderboard;

            high_score_data.0 = high_scores;
        };
    }
}

fn handle_error(mut ev_error: EventReader<HttpResponseError>) {
    for error in ev_error.read() {
        println!("Error retrieving IP: {}", error.err);
    }
}

fn setup_leaderboard(mut commands: Commands, hud: Res<Hud>) {
    commands.entity(hud.0).with_children(|parent| {
        parent
            .spawn((
                StateScoped(AppState::LeaderboardSelection),
                Node {
                    position_type: PositionType::Absolute,
                    display: Display::Flex,
                    justify_self: JustifySelf::Center,
                    flex_direction: FlexDirection::Column,
                    width: Val::Px(100.0),

                    border: UiRect::all(Val::Px(2.0)),
                    top: Val::Px(425.0),
                    left: Val::Px(500.0),
                    ..default()
                },
                BorderColor(LIGHT_COLOR),
                BorderRadius::MAX,
            ))
            .with_children(|p| {
                p.spawn(
                    ((
                        Node {
                            width: Val::Percent(100.0),
                            ..default()
                        },
                        BorderRadius::MAX,
                        Pickable::default(),
                        Text::default(),
                        BackgroundColor(DARK_COLOR),
                        TextLayout::default().with_justify(JustifyText::Center),
                        children![(
                            TextColor(LIGHT_COLOR),
                            TextFont::from_font(BODY_FONT)
                                .with_font_size(RESOLUTION_HEIGHT * 6. / 8. / 30.)
                                .with_line_height(bevy::text::LineHeight::RelativeToFont(2.5)),
                            Pickable::IGNORE,
                            TextSpan::new(format!("Back")),
                        )],
                    )),
                )
                .observe(back_to_leaderboard_selection);
            });

        parent
            .spawn((
                StateScoped(AppState::Leaderboard),
                Node {
                    position_type: PositionType::Absolute,
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    width: Val::Percent(100.0),
                    top: Val::Px(55.0),
                    align_items: AlignItems::Center,
                    ..default()
                },
                Pickable::IGNORE,
            ))
            .with_children(|p| {
                p.spawn((
                    Pickable::IGNORE,
                    HighScoreboardLevel,
                    TextFont::from_font(BODY_FONT)
                        .with_font_size(RESOLUTION_HEIGHT * 6. / 8. / 25.),
                    Text("".into()),
                ));
                p.spawn((
                    Pickable::IGNORE,
                    HighScoreboard,
                    TextFont::from_font(BODY_FONT)
                        .with_font_size(RESOLUTION_HEIGHT * 6. / 8. / 25.),
                    Text("".into()),
                ));
            });
    });
}

fn back_to_leaderboard_selection(_: Trigger<Pointer<Click>>, mut commands: Commands) {
    commands.send_event(SceneChange(AppState::LeaderboardSelection));
}

fn back_to_menu(_: Trigger<Pointer<Click>>, mut commands: Commands) {
    commands.send_event(SceneChange(AppState::Menu));
}

fn update_high_scoreboard(
    high_score_data: Res<HighScores>,
    leaderboard_level_selected: Res<LeaderboardLevelSelected>,
    mut high_scoreboard_level: Query<&mut Text, With<HighScoreboardLevel>>,
    mut high_scoreboard: Query<&mut Text, (With<HighScoreboard>, Without<HighScoreboardLevel>)>,
) {
    let Ok(mut header_text) = high_scoreboard_level.single_mut() else {
        return;
    };
    let Ok(mut text) = high_scoreboard.single_mut() else {
        return;
    };

    let level_id = match leaderboard_level_selected.0 {
        Some(level_id) => {
            header_text.0 = format!("High Scores - Level {}\n----------------\n", level_id + 1);
            level_id
        }
        None => {
            header_text.0 = "No Level Selected".into();
            return;
        }
    };

    let mut leaders = high_score_data
        .0
        .iter()
        .filter(|data| data.level == level_id)
        .collect::<Vec<_>>();
    leaders.sort_by(|a, b| a.score.cmp(&b.score));

    let display_data = leaders
        .iter()
        .enumerate()
        .filter(|(_, data)| data.level == level_id)
        .filter(|(idx, _data)| *idx < 10)
        .map(|(idx, data)| {
            let name = data.name.chars().take(16).collect::<String>();
            format!("#{} - {}: {}", idx + 1, name, data.score)
        })
        .collect::<Vec<_>>()
        .join("\n\n");

    text.0 = display_data;
}

pub fn update_high_scoreboard_top_five(
    high_score_data: Res<HighScores>,
    leaderboard_level_selected: Res<LeaderboardLevelSelected>,
    mut high_scoreboard: Query<&mut Text, With<HighScoreboardTopFive>>,
) {
    let Some(level_id) = leaderboard_level_selected.0 else {
        return;
    };

    let Ok(mut text) = high_scoreboard.single_mut() else {
        return;
    };

    let mut leaders = high_score_data
        .0
        .iter()
        .filter(|data| data.level == level_id)
        .collect::<Vec<_>>();
    leaders.sort_by(|a, b| a.score.cmp(&b.score));

    let display_data = leaders
        .iter()
        .enumerate()
        .filter(|(_, data)| data.level == level_id)
        .filter(|(idx, _data)| *idx < 5)
        .map(|(idx, data)| {
            let name = data.name.chars().take(16).collect::<String>();
            format!("#{} - {}: {}", idx + 1, name, data.score)
        })
        .collect::<Vec<_>>()
        .join("\n\n");

    text.0 = display_data;
}
