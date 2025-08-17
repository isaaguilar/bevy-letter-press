use crate::app::AppState;
use crate::app::DARK_COLOR;
use crate::app::DialogDisplay;
use crate::app::DisplayLanguage;
use crate::app::LIGHT_COLOR;
use crate::app::RESOLUTION_HEIGHT;
use crate::app::RESOLUTION_WIDTH;
use crate::assets::custom::ImageAssets;
use crate::assets::lexi::menu::{Choice, MenuData};
use crate::util::handles::BODY_FONT;
use bevy::ecs::entity;
use bevy::input::keyboard::KeyboardInput;
use bevy::prelude::*;
use bevy_aspect_ratio_mask::Hud;
use bevy_simple_text_input::TextInput;
use bevy_simple_text_input::TextInputPlugin;
use bevy_simple_text_input::TextInputTextColor;
use bevy_simple_text_input::TextInputTextFont;
use bevy_simple_text_input::TextInputValue;

mod actions;
mod inputs;
pub mod layouts;

pub struct Menu;

impl Plugin for Menu {
    fn build(&self, app: &mut App) {
        app.insert_resource(ActiveMenu::default());

        app.add_plugins((TextInputPlugin))
            .insert_resource(LeaderboardName::default())
            .add_systems(
                Update,
                leaderboard_name.run_if(in_state(AppState::Menu).and(on_event::<KeyboardInput>)),
            );

        app.add_event::<ChangeMenu>();

        app.add_systems(OnEnter(AppState::Menu), menu_setup)
            .add_systems(Update, move_choice_marker.run_if(in_state(AppState::Menu)))
            .add_systems(
                Update,
                change_menu.run_if(on_event::<ChangeMenu>.and(in_state(AppState::Menu))),
            )
            .add_systems(OnExit(AppState::Menu), leave_menu);

        app.add_plugins(inputs::plugin);
    }
}

#[derive(Resource, Default, Clone)]
pub struct ActiveMenu {
    pub opt: Option<MenuData>,
}

impl ActiveMenu {
    pub fn reset(&mut self) {
        *self = Self { ..default() };
    }
}

#[derive(Component)]
struct SelectionMarker(Choice);

#[derive(Resource, Debug, Clone, Deref, DerefMut)]
struct CurrentSelection(Option<Choice>);

impl Default for CurrentSelection {
    fn default() -> Self {
        Self(Default::default())
    }
}

#[derive(Event)]
struct ChangeMenu(String);

impl ChangeMenu {
    fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }
}

#[derive(Component)]
struct Language(String);

impl Language {
    fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }
}

#[derive(Component)]
struct GoToMenu(String);

impl GoToMenu {
    fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }
}

fn menu_setup(
    mut commands: Commands,
    mut bg: ResMut<ClearColor>,
    display_language: ResMut<DisplayLanguage>,
) {
    info!("Menu");
    info!(language = display_language.0);
    bg.0 = DARK_COLOR;
    // commands.spawn((StateScoped(AppState::Menu), Camera2d::default()));

    commands.send_event(ChangeMenu::new("main menu"));
    return;
}

fn change_menu(
    mut changes: EventReader<ChangeMenu>,
    mut commands: Commands,
    display_language: ResMut<DisplayLanguage>,
    mut dialog_message: ResMut<ActiveMenu>,
    menu_data: Res<Assets<MenuData>>,
    dialog_display_query: Query<(Entity, &DialogDisplay), With<DialogDisplay>>,
    hud: Res<Hud>,
    image_assets: Res<ImageAssets>,
    mut leaderboard_name: ResMut<LeaderboardName>,
) {
    info!("Spawning menu");
    let hud_entity = hud.0;

    let Some(event) = changes.read().next() else {
        return;
    };

    let menu_id = &event.0;

    dialog_message.opt = menu_data
        .into_inner()
        .iter()
        .filter(|(_, data)| data.id == menu_id.clone())
        .map(|(_, data)| data.clone())
        .next();

    let dialog = match &dialog_message.opt {
        Some(d) => d,
        None => {
            for (entity, _) in dialog_display_query.iter() {
                commands.entity(entity).despawn();
            }
            return;
        }
    };

    for (entity, dialog_display) in dialog_display_query.iter() {
        if dialog_display.0 != dialog.id {
            commands.entity(entity).despawn();
        } else {
            return;
        }
    }

    let padding_x = RESOLUTION_WIDTH / 32.0;
    commands.entity(hud_entity).with_children(|parent| {
        if *menu_id == "main menu" {
            parent.spawn((
                StateScoped(AppState::Menu),
                DialogDisplay(dialog.id.clone()),
                Node {
                    position_type: PositionType::Relative,
                    height: Val::Px(324.0),
                    width: Val::Px(450.0),
                    left: Val::Px(85.),
                    top: Val::Px(-25.),
                    // margin: UiRect::new(Val::Px(0.0), Val::Px(2.0), Val::Px(5.0), Val::Px(-120.0)),
                    ..default()
                },
                ImageNode {
                    image: image_assets.title.clone(),
                    ..default()
                },
            ));

            parent.spawn((
                StateScoped(AppState::Menu),
                DialogDisplay(dialog.id.clone()),
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(15.),
                    top: Val::Px(436.),
                    // margin: UiRect::new(Val::Px(0.0), Val::Px(2.0), Val::Px(5.0), Val::Px(-120.0)),
                    ..default()
                },
                TextColor(LIGHT_COLOR),
                TextFont::from_font(BODY_FONT)
                    .with_font_size(RESOLUTION_HEIGHT * 6. / 8. / 35.)
                    .with_line_height(bevy::text::LineHeight::RelativeToFont(2.5)),
                Text::new("Enter name to compete on Leaderboard:"),
            ));

            match &leaderboard_name.0 {
                Some(name) => {
                    parent.spawn((
                        StateScoped(AppState::Menu),
                        ShowSetName,
                        DialogDisplay(dialog.id.clone()),
                        Node {
                            position_type: PositionType::Absolute,
                            width: Val::Px(220.0),
                            height: Val::Px(30.0),
                            left: Val::Px(400.0),
                            top: Val::Px(436.0),
                            // border: UiRect::all(Val::Px(5.0)),
                            padding: UiRect::all(Val::Px(7.0)),
                            ..default()
                        },
                        TextColor(LIGHT_COLOR),
                        TextFont::from_font(BODY_FONT)
                            .with_font_size(RESOLUTION_HEIGHT * 6. / 8. / 35.)
                            .with_line_height(bevy::text::LineHeight::RelativeToFont(2.5)),
                        Text::new(name),
                    ));

                    parent.spawn((
                        StateScoped(AppState::Menu),
                        ShowSetName,
                        DialogDisplay(dialog.id.clone()),
                        Node {
                            position_type: PositionType::Absolute,
                            width: Val::Px(220.0),
                            height: Val::Px(30.0),
                            left: Val::Px(400.0),
                            top: Val::Px(455.0),
                            // border: UiRect::all(Val::Px(5.0)),
                            padding: UiRect::all(Val::Px(7.0)),
                            ..default()
                        },
                        TextColor(LIGHT_COLOR),
                        TextFont::from_font(BODY_FONT)
                            .with_font_size(RESOLUTION_HEIGHT * 6. / 8. / 35.)
                            .with_line_height(bevy::text::LineHeight::RelativeToFont(2.5)),
                        Text::new("--------------------"),
                    ));

                    parent
                        .spawn((
                            StateScoped(AppState::Menu),
                            ShowSetName,
                            DialogDisplay(dialog.id.clone()),
                            Node {
                                position_type: PositionType::Absolute,
                                width: Val::Px(30.0),
                                height: Val::Px(30.0),
                                left: Val::Px(570.0),
                                top: Val::Px(433.0),
                                // border: UiRect::all(Val::Px(5.0)),
                                // padding: UiRect::all(Val::Px(7.0)),
                                ..default()
                            },
                            Pickable::default(),
                            BorderColor(LIGHT_COLOR),
                            BorderRadius::MAX,
                            BackgroundColor(LIGHT_COLOR),
                            TextColor(DARK_COLOR),
                            TextLayout::default().with_justify(JustifyText::Center),
                            TextFont::from_font(BODY_FONT)
                                .with_font_size(RESOLUTION_HEIGHT * 6. / 8. / 25.)
                                .with_line_height(bevy::text::LineHeight::RelativeToFont(2.5)),
                            Text::new("X"),
                        ))
                        .observe(remove_name);
                }
                None => {
                    parent.spawn((
                        StateScoped(AppState::Menu),
                        DialogDisplay(dialog.id.clone()),
                        Node {
                            position_type: PositionType::Absolute,
                            width: Val::Px(220.0),
                            height: Val::Px(30.0),
                            left: Val::Px(410.0),
                            top: Val::Px(433.0),
                            // border: UiRect::all(Val::Px(5.0)),
                            padding: UiRect::all(Val::Px(7.0)),
                            ..default()
                        },
                        BorderColor(LIGHT_COLOR),
                        BorderRadius::MAX,
                        BackgroundColor(LIGHT_COLOR),
                        TextInput,
                        TextInputTextFont(
                            TextFont::from_font(BODY_FONT)
                                .with_font_size(RESOLUTION_HEIGHT * 6. / 8. / 25.),
                        ),
                        TextInputTextColor(DARK_COLOR.into()),
                    ));
                }
            }
        }

        parent
            .spawn((
                StateScoped(AppState::Menu),
                DialogDisplay(dialog.id.clone()),
                layouts::menu_layout(RESOLUTION_WIDTH - 2. * padding_x),
            ))
            .with_children(|p| {
                let text = dialog.lex.from_language(&display_language.0);

                p.spawn(layouts::header_layout(&text));

                match &dialog.choices {
                    Some(choices) => {
                        for (_index, choice) in choices.iter().enumerate() {
                            let text = choice.choice.lex.from_language(&display_language.0);

                            p.spawn((layouts::button_layout(&text, choice.clone())))
                                .with_children(|p| {
                                    let mut button =
                                        p.spawn(layouts::button_text(&text, choice.clone()));
                                    button.observe(inputs::mouse_move);
                                    button.observe(inputs::mouse_over);
                                    match &choice.choice.action {
                                        Some(action) => match action.as_str() {
                                            "start_game" => {
                                                button.observe(inputs::click_start_game);
                                            }
                                            "show_credits" => {
                                                button.observe(inputs::click_show_credits);
                                            }
                                            "show_leaderboard" => {
                                                button.observe(inputs::click_show_leaderboard);
                                            }

                                            "english" | "spanish" => {
                                                button
                                                    .insert(Language::new(action))
                                                    .observe(inputs::click_language_selection);
                                            }
                                            _ => {}
                                        },
                                        None => {}
                                    }

                                    match &choice.choice.next_id {
                                        Some(id) => {
                                            button
                                                .insert(GoToMenu::new(id))
                                                .observe(inputs::click_menu_selection);
                                        }
                                        None => {}
                                    }
                                });
                        }
                    }
                    None => {}
                }
            });
    });

    return;
}

#[derive(Component)]
struct ShowSetName;

fn remove_name(
    _: Trigger<Pointer<Click>>,
    mut leaderboard_name: ResMut<LeaderboardName>,
    query: Query<Entity, With<ShowSetName>>,
    mut commands: Commands,
    hud: Res<Hud>,
) {
    leaderboard_name.clear();

    for entity in query {
        commands.entity(entity).despawn();
    }

    commands.entity(hud.0).with_children(|parent| {
        parent.spawn((
            StateScoped(AppState::Menu),
            Node {
                position_type: PositionType::Absolute,
                width: Val::Px(220.0),
                height: Val::Px(30.0),
                left: Val::Px(410.0),
                top: Val::Px(433.0),
                // border: UiRect::all(Val::Px(5.0)),
                padding: UiRect::all(Val::Px(7.0)),
                ..default()
            },
            BorderColor(LIGHT_COLOR),
            BorderRadius::MAX,
            BackgroundColor(LIGHT_COLOR),
            TextInput,
            TextInputTextFont(
                TextFont::from_font(BODY_FONT).with_font_size(RESOLUTION_HEIGHT * 6. / 8. / 25.),
            ),
            TextInputTextColor(DARK_COLOR.into()),
        ));
    });
}

fn leaderboard_name(
    mut events: EventReader<KeyboardInput>,
    text_input_query: Query<&TextInputValue>,
    mut leaderboard_name: ResMut<LeaderboardName>,
) {
    for event in events.read() {
        if event.key_code == KeyCode::Enter {
            return;
        }
        leaderboard_name.0 = match text_input_query.single() {
            Ok(t) => Some(t.0.clone()),
            Err(_) => None,
        }
    }
}

#[derive(Resource, Default)]
pub struct LeaderboardName(pub Option<String>);

impl LeaderboardName {
    pub fn clear(&mut self) {
        self.0 = None;
    }
}

fn move_choice_marker(
    display_language: ResMut<DisplayLanguage>,
    dialog_message: Res<ActiveMenu>,
    current_selection: Res<CurrentSelection>,
    mut button: Query<&mut BackgroundColor>,
    mut selections: Query<(&mut TextSpan, &mut TextColor, &ChildOf), With<layouts::MenuOption>>,
) {
    let Some(current_choice) = &current_selection.0 else {
        return;
    };

    let dialog = match &dialog_message.opt {
        Some(d) => d,
        None => {
            return;
        }
    };

    let choices = match &dialog.choices {
        Some(choices) => {
            if choices.is_empty() {
                return;
            }
            choices
        }
        None => return,
    };

    for (idx, choice) in choices.iter().enumerate() {
        let text = choice.choice.lex.from_language(&display_language.0);

        if current_choice.id == choice.id.clone() {
            for (text_idx, (mut text_span, mut text_color, parent)) in
                selections.iter_mut().enumerate()
            {
                if idx == text_idx {
                    *text_span = TextSpan::new(format!("> {}", text.clone()));
                    if let Ok(mut bg) = button.get_mut(parent.0) {
                        bg.0 = DARK_COLOR;
                        text_color.0 = LIGHT_COLOR;
                    }
                }
            }
        } else {
            for (text_idx, (mut text_span, mut text_color, parent)) in
                selections.iter_mut().enumerate()
            {
                if idx == text_idx {
                    *text_span = TextSpan::new(format!(" {}", text.clone()));
                    if let Ok(mut bg) = button.get_mut(parent.0) {
                        bg.0 = LIGHT_COLOR;
                        text_color.0 = DARK_COLOR;
                    }
                }
            }
        }
    }
}

fn leave_menu(mut dialog_message: ResMut<ActiveMenu>) {
    dialog_message.reset();
}
