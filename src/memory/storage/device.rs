use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum SaveType {
    None,
    SRAM,
    Flash64K,
    Flash128K,
    EEPROM4K,
    EEPROM64K
}

pub trait SaveDevice {
    fn read(&self, addr: u32) -> u8;
    fn write(&mut self, addr: u32, value: u8);
    fn get_data(&self) -> &[u8];
    fn get_type(&self) -> SaveType;
    fn storage_type(&self) -> &'static str;
}