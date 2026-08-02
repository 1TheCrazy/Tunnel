use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
    path::PathBuf,
};

use serde::{Deserialize, Serialize};
use tunnel_core::{
    state::io_manager,
    structs::{
        client::{ClientSave, ClientServer},
        errors::SaveError,
    },
};

#[derive(Debug, Default, Serialize, Deserialize)]
#[repr(transparent)]
#[serde(transparent)]
pub struct GuiClientSave(pub ClientSave);

#[derive(Default, Serialize, Deserialize)]
pub struct GuiNodeLocations {
    pub locations: HashMap<String, NodeLocation>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct NodeLocation {
    pub latitude: f64,
    pub longitude: f64,
}

impl Deref for GuiClientSave {
    type Target = ClientSave;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for GuiClientSave {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub struct AutosaveMutGuiClientSave {
    save: GuiClientSave,
}

impl Drop for AutosaveMutGuiClientSave {
    fn drop(&mut self) {
        if let Err(err) = write_save(&self.save) {
            eprintln!(
                "Failed to save mutated state. Changes will not saved: {}",
                err
            );
        }
    }
}

impl Deref for AutosaveMutGuiClientSave {
    type Target = GuiClientSave;

    fn deref(&self) -> &Self::Target {
        &self.save
    }
}

impl DerefMut for AutosaveMutGuiClientSave {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.save
    }
}

pub fn get_ref_save() -> GuiClientSave {
    get_save()
}

pub fn get_mut_save() -> AutosaveMutGuiClientSave {
    AutosaveMutGuiClientSave { save: get_save() }
}

pub fn get_active_server(save: &GuiClientSave) -> Option<&ClientServer> {
    if save.active_server_index == -1 {
        return None;
    }

    save.servers.get(save.active_server_index as usize)
}

pub fn get_mut_active_server(save: &mut GuiClientSave) -> Option<&mut ClientServer> {
    if save.active_server_index == -1 {
        return None;
    }

    let active_server_index = save.active_server_index as usize;
    save.servers.get_mut(active_server_index)
}

fn write_save(save: &GuiClientSave) -> Result<(), SaveError> {
    let path = io_manager::CLIENT_SAVE_PATH();
    io_manager::write_save(save, &path)
}

fn get_save() -> GuiClientSave {
    let path = io_manager::CLIENT_SAVE_PATH();
    io_manager::read_save_or_default(&path)
}

pub fn get_node_locations() -> GuiNodeLocations {
    let path = gui_node_locations_save_path();
    io_manager::read_save_or_default(&path)
}

pub fn write_node_locations(locations: &GuiNodeLocations) -> Result<(), SaveError> {
    let path = gui_node_locations_save_path();
    io_manager::write_save(locations, &path)
}

fn gui_node_locations_save_path() -> PathBuf {
    io_manager::save_path().join("gui-node-locations.save")
}
