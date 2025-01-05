use bevy::{
    // color::palettes::css::*,
    math::ops,
    prelude::*,
    // sprite::Anchor,
    // text::{FontSmoothing, LineBreak, TextBounds},
};

fn main() {
    App::new()
    .add_plugins(DefaultPlugins)
    .add_systems(Startup, setup)
    .add_systems(Update, animate_translation)
    .run();
}

#[derive(Component)]
struct AnimateTranslation;

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load::<Font>("fonts/FiraSans-Bold.ttf");
    let text_font = TextFont {
        font: font.clone(),
        font_size: 50.0,
        ..default()
    };

    let text_justification = JustifyText::Center;

    commands.spawn(Camera2d);

    commands.spawn((
        Text2d::new("translation"),
        text_font.clone(),
        TextLayout::new_with_justify(text_justification),
        AnimateTranslation,
    ));
}

fn animate_translation(
    time: Res<Time>,
    mut query: Query<&mut Transform, (With<Text2d>, With<AnimateTranslation>)>,
) {
    for mut transform in &mut query {
        transform.translation.x = 100.0 * ops::sin(time.elapsed_secs()) - 400.0;
        transform.translation.y = 100.0 * ops::cos(time.elapsed_secs());
    }
}
