use log::{error, warn};
use serde::{Deserialize, Serialize};

use super::device::{SaveDevice, SaveType};

#[derive(Serialize, Deserialize)]
pub struct Sram {
    #[serde(with = "serde_bytes")]
    data: Vec<u8>
}

impl Sram {
    pub fn new(storage: Option<Vec<u8>>) -> Self {
        Self {
            data: storage.map_or_else(
                || vec![0; 0x10000],
                |data| {
                    if data.len() != 0x10000 {
                        error!("Invalid storage size: {:?}, expect 64KB", data.len());
                        vec![0; 0x10000]
                    } else {
                        data
                    }
                }
            )
        }
    }
}

impl SaveDevice for Sram {
    fn read(&self, addr: u32) -> u8 {
        if addr < 0x10000 {
            self.data[addr as usize]
        } else {
            warn!("SRAM: Out of bounds read at 0x{:08X}", addr);
            0xFF
        }
    }

    fn write(&mut self, addr: u32, data: u8) {
        if addr < 0x10000 {
            self.data[addr as usize] = data;
        } else {
            warn!("SRAM: Out of bounds write at 0x{:09X}", addr);
        }
    }

    fn get_data(&self) -> &[u8] {
        &self.data
    }

    fn get_type(&self) -> SaveType {
        SaveType::SRAM
    }

    fn storage_type(&self) -> &'static str {
        "SRAM (64K)"
    }
}

impl Default for Sram {
    fn default() -> Self {
        Self { data: vec![0; 0x10000] }
    }
}