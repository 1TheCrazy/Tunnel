use std::ops::{Deref, DerefMut};

use tunnel_core::{state::io_manager, structs::errors::SaveError};

use crate::structs::state::CliClientSave;

pub struct AutosaveMutCliClientSave {
    save: CliClientSave,
}

impl Drop for AutosaveMutCliClientSave {
    fn drop(&mut self) {
        match write_save(&self) {
            Ok(_) => {}
            Err(err) => eprintln!(
                "Failed to save mutated state. Changes will not saved: {}",
                err
            ),
        }
    }
}

impl Deref for AutosaveMutCliClientSave {
    type Target = CliClientSave;

    fn deref(&self) -> &Self::Target {
        &self.save
    }
}

impl DerefMut for AutosaveMutCliClientSave {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.save
    }
}

pub fn get_ref_save() -> CliClientSave {
    get_save()
}

pub fn get_mut_save() -> AutosaveMutCliClientSave {
    let save = get_save();

    AutosaveMutCliClientSave { save }
}

pub fn write_save(save: &CliClientSave) -> Result<(), SaveError> {
    let path = io_manager::CLIENT_SAVE_PATH();
    io_manager::write_save(save, &path)
}

fn get_save() -> CliClientSave {
    let path = io_manager::CLIENT_SAVE_PATH();
    let save: CliClientSave = io_manager::read_save_or_default(&path);

    return save;
}
