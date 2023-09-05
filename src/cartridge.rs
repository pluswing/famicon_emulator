use crate::rom::Rom;
use std::fs::File;
use std::io::Read;
use std::path::Path;

pub fn load_rom(path: &str) -> Rom {
    let mut f = File::open(path).expect("no file found");
    let metadata = std::fs::metadata(path).expect("unable to read metadata");
    let mut buffer = vec![0; metadata.len() as usize];
    f.read(&mut buffer).expect("buffer overflow");
    let mut rom = Rom::new(&buffer).expect("load error");

    let (save_data_file, save_data) = load_save_data(path);
    rom.save_data_file = save_data_file;
    rom.save_data = save_data;

    rom
}

fn load_save_data(rom_path: &str) -> (String, Vec<u8>) {
    let save_data_file = String::from(rom_path) + ".save";
    let p = Path::new(save_data_file.as_str());
    if !p.is_file() {
        return (save_data_file, Vec::new());
    }
    let mut f = File::open(save_data_file.as_str()).expect("no save file found");
    let metadata = std::fs::metadata(save_data_file.as_str()).expect("unable to read metadata");
    let mut buffer = vec![0; metadata.len() as usize];
    f.read(&mut buffer).expect("buffer overflow");
    return (save_data_file, buffer);
}

pub mod test {
    use super::*;

    pub fn snake_rom() -> Rom {
        load_rom("rom/snake.nes")
    }

    pub fn test_rom() -> Rom {
        load_rom("rom/nestest.nes")
    }

    pub fn mario_rom() -> Rom {
        load_rom("rom/Super Mario Bros. (World).nes")
    }

    pub fn alter_ego_rom() -> Rom {
        load_rom("rom/Alter_Ego.nes")
    }
}
