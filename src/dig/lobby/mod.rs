use bevy::prelude::*;
use bevy_steam_p2p::{
    networked_events::{event::Networked, register::NetworkedEvents},
    LobbyJoined, NetworkData, OtherJoined, SteamP2PClient,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::{
    dig::terrain::spawn_terrain,
    voxel::chunks_manager::{ChunksInfo, ChunksManager, DigAction},
    PlayerSpawned,
};

use super::{player::spawn_player, terrain::FinishedGenerating};

#[derive(Clone, Event, Serialize, Deserialize)]
pub struct GenerateChunks {
    pub dig_history: Vec<DigAction>,
}

pub struct DigLobbyPlugin;
impl Plugin for DigLobbyPlugin {
    fn build(&self, app: &mut App) {
        app.add_networked_event::<GenerateChunks>().add_systems(
            Update,
            (menu, on_lobby_join, on_other_joined, handle_player_spawn),
        );
    }
}

fn menu(client: ResMut<SteamP2PClient>, keys: Res<ButtonInput<KeyCode>>) {
    if keys.just_pressed(KeyCode::KeyC) {
        client.create_lobby(8);
    }
}

fn on_lobby_join(
    mut join_r: EventReader<LobbyJoined>,
    client: ResMut<SteamP2PClient>,
    chunks_manager: ChunksManager,
    mut test_w: EventWriter<GenerateChunks>,
) {
    if !join_r.is_empty() {
        if client.is_lobby_owner().is_ok_and(|b| b) {
            test_w.send(GenerateChunks {
                dig_history: Vec::new(),
            });
            println!("Di");
        }
        join_r.clear();
        println!("Joined lobby");
    }
}

fn on_other_joined(
    client: ResMut<SteamP2PClient>,
    mut joined_r: EventReader<OtherJoined>,
    maybe_chunks_info: Option<Res<ChunksInfo>>,
    mut test_w: EventWriter<Networked<GenerateChunks>>,
) {
    let Some(chunks_info) = maybe_chunks_info else {
        return;
    };
    for joined in joined_r.read() {
        if client.is_lobby_owner().is_ok_and(|b| b) {
            test_w.send(Networked::new_only_others(GenerateChunks {
                dig_history: chunks_info.dig_history.clone(),
            }));
        }
    }
    joined_r.clear();
}

fn handle_player_spawn(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut generation_r: EventReader<FinishedGenerating>,
    maybe_player_spawned: Option<Res<PlayerSpawned>>,
) {
    let read = generation_r.read();
    if read.len() > 0 && maybe_player_spawned.is_none() {
        commands.insert_resource(PlayerSpawned);
        spawn_player(&mut commands, &mut meshes, &mut materials);
    }
}
