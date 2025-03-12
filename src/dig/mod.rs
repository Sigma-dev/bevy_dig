use bevy::prelude::*;
use lobby::DigLobbyPlugin;
use player::DigPlayerPlugin;
use terrain::DigTerrainPlugin;

pub mod lobby;
pub mod player;
pub mod terrain;

pub struct DigPlugin;
impl Plugin for DigPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((DigPlayerPlugin, DigTerrainPlugin, DigLobbyPlugin));
    }
}
