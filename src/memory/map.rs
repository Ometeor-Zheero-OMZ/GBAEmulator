use std::sync::{Arc, Mutex};

use log::{debug, error, warn};

use crate::memory::storage::{
    device::{SaveDevice, SaveType},
    sram::Sram,
    flash::Flash
};

pub struct MemoryMap {
    // 他のメモリ領域
    bios: Vec<u8>,
    wram: Vec<u8>,
    iwram: Vec<u8>,
    // その他の領域がある場合は記載

    // セーブデバイス（動的ディスパッチで実行時に切り替え可能）
    save_device: Box<dyn SaveDevice + Send>
}

impl MemoryMap {
    pub fn new() -> Self {
        // デフォルトはセーブなし
        Self {
            bios: vec![0; 0x4000],
            wram: vec![0; 0x40000],
            iwram: vec![0; 0x8000],
            // その他の初期化
            save_device: Box::new(Sram::default()), // デフォルトは SRAMを設定
        }
    }

    pub fn set_save_device(&mut self, save_type: SaveType, data: Option<Vec<u8>>) {
        self.save_device = match save_type {
            SaveType::None => Box::new(Sram::default()),
            SaveType::SRAM => Box::new(Sram::new(data)),
            SaveType::Flash64K => Box::new(Flash::new(64 * 1024, data)),
            SaveType::Flash128K => Box::new(Flash::new(128 * 1024, data)),
            // その他のタイプ
            _ => {
                warn!("Unsupported save type: {:?}, falling back to SRAM", save_type);
                Box::new(Sram::new(data))
            }
        };
    }

    pub fn read_byte(&self, addr: u32) -> u8 {
        match addr & 0xFF000000 {
            0x0E000000 => self.read_save(addr),
            // その他のメモリ領域
            _ => {
                warn!("Unhandled memory read at 0x{:08X}", addr);
                0xFF
            }
        }
    }

    pub fn write_byte(&mut self, addr: u32, value: u8) {
        match addr & 0xFF000000 {
            0x0E000000 => self.write_save(addr, value),
            // その他のメモリ領域
            _ => {
                warn!("Unhandled memory write at 0x{:08X} = 0x{:02X}", addr, value);
            }
        }
    }

    fn read_save(&self, addr: u32) -> u8 {
        let relative_addr = addr & 0x00FFFFFF;
        self.save_device.read(relative_addr)
    }

    fn write_save(&mut self, addr: u32, value: u8) {
        let relative_addr = addr & 0x00FFFFFF;
        self.save_device.write(relative_addr, value);
    }
}