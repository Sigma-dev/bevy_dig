use bevy::prelude::*;
use player::DigPlayerPlugin;
use terrain::DigTerrainPlugin;

use crate::sky::SkyPlugin;

pub mod player;
pub mod terrain;

pub struct DigPlugin;
impl Plugin for DigPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((DigPlayerPlugin, DigTerrainPlugin, SkyPlugin));
    }
}
