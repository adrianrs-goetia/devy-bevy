pub mod exit_game;
pub mod input_manager;

pub struct CorePlugin;
impl bevy::prelude::Plugin for CorePlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins((exit_game::ExitGamePlugin, input_manager::InputManagerPlugin));
    }
}
