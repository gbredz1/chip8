use std::{
    fmt::{self, Display},
    fs::File,
    io::{Error, Read},
};

pub struct Rom {
    data: Vec<u8>,
    size: usize,
}

impl Rom {
    pub fn new_from(path: &str) -> Result<Self, Error> {
        let mut file = File::open(&path)?;
        let mut data = vec![];

        file.read_to_end(&mut data)?;

        Ok(Self {
            size: data.len(),
            data,
        })
    }

    pub fn read(&self, addr: u16) -> u8 {
        self.data[addr as usize]
    }

    pub fn write(&mut self, addr: u16, data: u8) {
        self.data[addr as usize] = data;
    }

    pub fn size(&self) -> usize {
        self.size
    }
}

impl Display for Rom {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Rom {{")?;
        write!(f, " size: {} Bytes ", self.size)?;
        write!(f, "}}")
    }
}
