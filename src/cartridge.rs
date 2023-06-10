use crate::rom::Rom;
use std::fs::File;
use std::io::Read;

pub fn load_rom(path: &str) -> Rom {
    let mut f = File::open(path).expect("no file found");
    let metadata = std::fs::metadata(path).expect("unable to read metadata");
    let mut buffer = vec![0; metadata.len() as usize];
    f.read(&mut buffer).expect("buffer overflow");
    let rom = Rom::new(&buffer).expect("load error");
    rom
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

    pub fn dq3_rom() -> Rom {
        load_rom("rom/Dragon Quest III - Soshite Densetsu e... (Japan).nes")
    }

    pub fn dq2_rom() -> Rom {
        load_rom("rom/Dragon Quest II - Akuryou no Kamigami (Japan).nes")
    }
}
