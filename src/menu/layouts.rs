use super::{Choice, SelectionMarker};
use crate::app::{DARK_COLOR, LIGHT_COLOR, RESOLUTION_HEIGHT};
use crate::util::handles::BODY_FONT;
use bevy::{prelude::*, ui::Val::*};

#[derive(Component)]
pub struct MenuOption;

// -- Menu Component Bundles --
pub fn menu_layout(width: f32) -> impl Bundle {
    // menu with ~ 25%

    (
        // BackgroundColor(RED.into()),
        Name::new("MenuLayout"),
        Node {
            position_type: PositionType::Absolute,

            display: Display::Flex,
            flex_direction: FlexDirection::Column,

            width: Val::Px(width),
            // always center
            top: Px(295.0),
            align_content: AlignContent::Start,
            justify_content: JustifyContent::Start,

            // margin: UiRect::all(Px(15.)),
            // padding: UiRect::bottom(Px(15.))
            //     .with_left(Px(15.))
            //     .with_right(Px(15.)),
            ..default()
        },
    )
}

pub fn button_layout(text: &str, choice: Choice) -> impl Bundle {
    (
        // BackgroundColor(DARK_ORCHID.into()),
        Name::new(format!("Button {}", text)),
        Node {
            position_type: PositionType::Relative,
            justify_self: JustifySelf::Center,
            width: Val::Px(200.0),
            height: Val::Px(30.0),
            left: Val::Px(200.),
            // border: UiRect::all(Val::Px(2.0)),
            margin: UiRect::new(Val::Px(0.0), Val::Px(2.0), Val::Px(5.0), Val::Px(5.0)),
            // padding: UiRect::all(Val::Px(0.0)),
            // margin: UiRect::all(Val::Px(0.0)),
            border: UiRect::all(Val::Px(2.0)),
            // margin: UiRect::vertical(Px(1.0)),
            ..default()
        },
        BorderColor(LIGHT_COLOR),
        BorderRadius::MAX,
    )
}

pub fn button_text(text: &str, choice: Choice) -> impl Bundle {
    (
        Node {
            width: Val::Percent(100.0),
            ..default()
        },
        BorderRadius::MAX,
        Pickable::default(),
        SelectionMarker(choice),
        Text::default(),
        BackgroundColor(DARK_COLOR),
        TextLayout::default().with_justify(JustifyText::Center),
        children![(
            MenuOption,
            TextColor(LIGHT_COLOR),
            TextFont::from_font(BODY_FONT)
                .with_font_size(RESOLUTION_HEIGHT * 6. / 8. / 30.)
                .with_line_height(bevy::text::LineHeight::RelativeToFont(2.5)),
            Pickable::IGNORE,
            TextSpan::new(format!("{}", text)),
        )],
    )
}

pub fn header_layout(text: &str) -> impl Bundle {
    (
        // BackgroundColor(ORANGE_700.into()),
        Name::new("Menu Title"),
        Node {
            position_type: PositionType::Relative,
            justify_self: JustifySelf::Start,
            margin: UiRect::vertical(Px(1.0)),
            ..default()
        },
        Pickable::default(),
        TextLayout::default().with_justify(JustifyText::Center),
        Text::default(),
        children![(
            TextColor(LIGHT_COLOR),
            TextFont::from_font(BODY_FONT)
                .with_font_size(RESOLUTION_HEIGHT * 6. / 8. / 30.)
                .with_line_height(bevy::text::LineHeight::RelativeToFont(2.5)),
            Pickable::IGNORE,
            TextSpan::new(format!("{}", text)),
        )],
    )
}
