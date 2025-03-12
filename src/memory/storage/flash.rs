use std::io::{Read, Write};

use log::{debug, error, trace, warn};
use serde::{Deserialize, Serialize};

use super::device::{SaveDevice, SaveType};

#[derive(Deserialize, Serialize)]
pub struct Flash {
    state: State,
    read_mode: ReadMode,
    bank: u32,
    #[serde(with = "serde_bytes")]
    data: Vec<u8>
}

#[derive(Debug, Deserialize, Serialize)]
enum State {
    WaitForCommand(usize, CommandContext),
    WriteSingleByte,
    BankChange
}

#[derive(PartialEq, Eq, Debug, Deserialize, Serialize)]
enum CommandContext {
    None,
    Erase
}

#[derive(PartialEq, Eq, Debug, Deserialize, Serialize)]
enum ReadMode {
    Data,
    ChipId
}

impl Flash {
    pub fn new(size: usize, storage: Option<Vec<u8>>) -> Self {
        let data = if let Some(storage) = storage {
            if storage.len() != size {
                error!(
                    "Invalid storage size: {:?}, expect {} bytes",
                    storage.len(),
                    size
                );
                vec![0xFF; size]
            } else {
                storage
            }
        } else {
            vec![0xFF; size]
        };

        Self {
            state: State::WaitForCommand(0, CommandContext::None),
            read_mode: ReadMode::Data,
            bank: 0,
            data,
        }
    }
}

impl SaveDevice for Flash {
    fn read(&self, addr: u32) -> u8 {
        let addr = addr & 0xFFFF;
        let idx = self.bank as usize * 0x10000 + addr as usize;

        match &self.read_mode {
            ReadMode::ChipId => {
                // ID     Name       Size  Sectors  AverageTimings  Timeouts/ms   Waits
                // D4BFh  SST        64K   16x4K    20us?,?,?       10,  40, 200  3,2
                // 1CC2h  Macronix   64K   16x4K    ?,?,?           10,2000,2000  8,3
                // 1B32h  Panasonic  64K   16x4K    ?,?,?           10, 500, 500  4,2
                // 3D1Fh  Atmel      64K   512x128  ?,?,?           ...40..,  40  8,8
                // 1362h  Sanyo      128K  ?        ?,?,?           ?    ?    ?    ?
                // 09C2h  Macronix   128K  ?        ?,?,?           ?    ?    ?    ?

                if self.data.len() == 64 * 1024 {
                    // SST for 64KB Flash
                    match addr {
                        0x0000 => 0xBF,
                        0x0001 => 0xD4,
                        _ => 0,
                    }
                } else {
                    // Sanyo for 128KB Flash
                    match addr {
                        0x0000 => 0x62,
                        0x0001 => 0x13,
                        _ => 0,
                    }
                }
            }
            ReadMode::Data => {
                if idx >= self.data.len() {
                    warn!(
                        "Flash read out of bounds: addr=0x{:04X}, bank={}, max_index={}",
                        addr, self.bank, self.data.len() - 1
                    );
                    0xFF
                } else {
                    self.data[idx]
                }
            }
        }
    }

    fn write(&mut self, addr: u32, value: u8) {
        let addr = addr & 0xFFFF;
        let idx = self.bank as usize * 0x10000 + addr as usize;

        trace!("Write Flash: 0x{addr:04X} = 0x{value:02X}");

        match &mut self.state {
            State::WaitForCommand(step, ctx) => match (*step, addr, value) {
                (0, 0x5555, 0xAA) => *step = 1,
                (1, 0x2AAA, 0x55) => *step = 2,

                (2, 0x5555, 0x90) if *ctx == CommandContext::None => {
                    debug!("Enter ID mode");
                    self.read_mode = ReadMode::ChipId;
                    self.state = State::WaitForCommand(0, CommandContext::None);
                }
                (2, 0x5555, 0xF0) if *ctx == CommandContext::None => {
                    debug!("Terminate ID mode");
                    if self.read_mode != ReadMode::ChipId {
                        warn!("FLASH: leave ID mode without entering");
                    }
                    self.read_mode = ReadMode::Data;
                    self.state = State::WaitForCommand(0, CommandContext::None);
                }

                (2, 0x5555, 0x80) => {
                    debug!("Enter erase mode");
                    self.state = State::WaitForCommand(0, CommandContext::Erase);
                }
                (2, 0x5555, 0x10) if *ctx == CommandContext::Erase => {
                    debug!("Erase entire chip");
                    self.data.fill(0xFF);
                    self.state = State::WaitForCommand(0, CommandContext::None);
                }
                (2, _, 0x30) if *ctx == CommandContext::Erase => {
                    let sector = (addr >> 12) as usize;
                    debug!("Erase sector {sector}");
                    let start = self.bank as usize * 0x10000 + sector * 0x1000;
                    self.data[start..start + 0x1000].fill(0xFF);
                    self.state = State::WaitForCommand(0, CommandContext::None);
                }

                (2, 0x5555, 0xA0) => {
                    trace!("Write single byte");
                    self.state = State::WriteSingleByte;
                }

                (2, 0x5555, 0xB0) => {
                    trace!("Enter bank change");
                    self.state = State::BankChange;
                }

                _ => {
                    warn!("Invalid command: value=0x{value:02X}, state:{:?}", self.state);
                }
            },

            State::WriteSingleByte => {
                // Only 1 -> 0 write is possible
                if idx >= self.data.len() {
                    warn!(
                        "Flash write out of bounds: addr=0x{:04X}, bank={}, max_index={}",
                        addr, self.bank, self.data.len() - 1
                    );
                    return;
                }

                self.data[idx] &= value;
                debug!("Write single byte: 0x{idx:05X} = 0x{value:02X}");
                self.state = State::WaitForCommand(0, CommandContext::None);
            }

            State::BankChange => {
                assert_eq!(addr, 0);
                let max_banks = (self.data.len() / (64 * 1024)) as u8;

                if value >= max_banks {
                    warn!("Invalid bank selection: {value}, max allowed bank: {}", max_banks - 1);
                    self.state = State::WaitForCommand(0, CommandContext::None);
                    return;
                }

                self.bank = value as u32;
                self.state = State::WaitForCommand(0, CommandContext::None);
            }
        }
    }

    fn get_data(&self) -> &[u8] {
        &self.data
    }

    fn get_type(&self) -> SaveType {
        if self.data.len() == 64 * 1024 {
            SaveType::Flash64K
        } else if self.data.len() == 128 * 1024 {
            SaveType::Flash128K
        } else {
            panic!("Unexpected Flash size: {}", self.data.len())
        }
    }

    fn storage_type(&self) -> &'static str {
        if self.data.len() == 64 * 1024 {
            "FLASH (512K)"
        } else if self.data.len() == 128 * 1024 {
            "FLASH (1M)"
        } else {
            unreachable!()
        }
    }
}