use bevy::color::Color;
use bevy::color::palettes::basic::{AQUA, LIME};
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::hierarchy::{BuildChildren, ChildBuild};
use bevy::prelude::{default, Alpha, BackgroundColor, Commands, Component, Entity, GlobalZIndex, Node, PositionType, Res, Single, Text, TextColor, TextFont, TextSpan, TextUiWriter, UiRect, Val, With};

#[derive(Component)]
pub struct StatsText;

pub fn counter_system(
    diagnostics: Res<DiagnosticsStore>,
    query: Single<Entity, With<StatsText>>,
    mut writer: TextUiWriter,
) {
    let text = *query;

    if let Some(fps) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS) {
        if let Some(raw) = fps.value() {
            *writer.text(text, 2) = format!("{raw:.2}");
        }
        if let Some(sma) = fps.average() {
            *writer.text(text, 4) = format!("{sma:.2}");
        }
        if let Some(ema) = fps.smoothed() {
            *writer.text(text, 6) = format!("{ema:.2}");
        }
    };
}

pub fn init_framerate_screen(mut commands: Commands, ) {
    let font = TextFont {
        font_size: 15.0,
        ..Default::default()
    };

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                padding: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(Color::BLACK.with_alpha(0.55)),
            GlobalZIndex(i32::MAX),
        ))
        .with_children(|p| {
            p.spawn((Text::default(), StatsText)).with_children(|p| {
                p.spawn((
                    TextSpan::new("FPS (raw): "),
                    font.clone(),
                    TextColor(LIME.into()),
                ));
                p.spawn((TextSpan::new(""), font.clone(), TextColor(AQUA.into())));
                p.spawn((
                    TextSpan::new("\nFPS (SMA): "),
                    font.clone(),
                    TextColor(LIME.into()),
                ));
                p.spawn((TextSpan::new(""), font.clone(), TextColor(AQUA.into())));
                p.spawn((
                    TextSpan::new("\nFPS (EMA): "),
                    font.clone(),
                    TextColor(LIME.into()),
                ));
                p.spawn((TextSpan::new(""), font.clone(), TextColor(AQUA.into())));
            });
        });
}
