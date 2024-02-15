use crate::{ReadError, Device};

/// Represents a datastream that can emit out a `T`
pub trait Get<T> {
    /// Blocks on device until we recieve enough data to parse `T`
    fn get(&mut self) -> Result<T, ReadError>;

    /// Same as get, except gets a String of bytes `T`
    /// If not a primitive type, returns the to_string of the type
    fn get_string(&mut self) -> Result<String, ReadError>;
}

impl Get<f64> for Device {
    //TODO: docs don't mention denormalized. Maybe we should just say floats are LE IEEE-754 and
    //send a link to that
    fn get(&mut self) -> Result<f64, ReadError> {
        let mut rbuff = [0u8; 8];
        self.serialport.read_exact(&mut rbuff)?;
        self.read_bytes += 8;
        self.read_checksum.update(&rbuff);
        Ok(f64::from_be_bytes(rbuff))
    }

    fn get_string(&mut self) -> Result<String, ReadError> {
        Ok(String::from_utf8(
            Get::<f64>::get(self)?.to_be_bytes().into(),
        )?)
    }
}

impl Get<f32> for Device {
    fn get(&mut self) -> Result<f32, ReadError> {
        let mut rbuff = [0u8; 4];
        self.serialport.read_exact(&mut rbuff)?;
        self.read_bytes += 4;
        self.read_checksum.update(&rbuff);
        Ok(f32::from_be_bytes(rbuff))
    }

    fn get_string(&mut self) -> Result<String, ReadError> {
        Ok(String::from_utf8(
            Get::<f32>::get(self)?.to_be_bytes().into(),
        )?)
    }
}

impl Get<i32> for Device {
    fn get(&mut self) -> Result<i32, ReadError> {
        let mut rbuff = [0u8; 4];
        self.serialport.read_exact(&mut rbuff)?;
        self.read_bytes += 4;
        self.read_checksum.update(&rbuff);
        Ok(i32::from_be_bytes(rbuff))
    }

    fn get_string(&mut self) -> Result<String, ReadError> {
        Ok(String::from_utf8(
            Get::<i32>::get(self)?.to_be_bytes().into(),
        )?)
    }
}

impl Get<i16> for Device {
    fn get(&mut self) -> Result<i16, ReadError> {
        let mut rbuff = [0u8; 2];
        self.serialport.read_exact(&mut rbuff)?;
        self.read_bytes += 2;
        self.read_checksum.update(&rbuff);
        Ok(i16::from_be_bytes(rbuff))
    }

    fn get_string(&mut self) -> Result<String, ReadError> {
        Ok(String::from_utf8(
            Get::<i16>::get(self)?.to_be_bytes().into(),
        )?)
    }
}

impl Get<i8> for Device {
    fn get(&mut self) -> Result<i8, ReadError> {
        let mut rbuff = [0u8; 1];
        self.serialport.read_exact(&mut rbuff)?;
        self.read_bytes += 1;
        self.read_checksum.update(&rbuff);
        Ok(i8::from_be_bytes(rbuff))
    }

    fn get_string(&mut self) -> Result<String, ReadError> {
        Ok(String::from_utf8(
            Get::<i8>::get(self)?.to_be_bytes().into(),
        )?)
    }
}

impl Get<u32> for Device {
    fn get(&mut self) -> Result<u32, ReadError> {
        let mut rbuff = [0u8; 4];
        self.serialport.read_exact(&mut rbuff)?;
        self.read_bytes += 4;
        self.read_checksum.update(&rbuff);
        Ok(u32::from_be_bytes(rbuff))
    }

    fn get_string(&mut self) -> Result<String, ReadError> {
        Ok(String::from_utf8(
            Get::<u32>::get(self)?.to_be_bytes().into(),
        )?)
    }
}

impl Get<u16> for Device {
    fn get(&mut self) -> Result<u16, ReadError> {
        let mut rbuff = [0u8; 2];
        self.serialport.read_exact(&mut rbuff)?;
        self.read_bytes += 2;
        self.read_checksum.update(&rbuff);
        Ok(u16::from_be_bytes(rbuff))
    }

    fn get_string(&mut self) -> Result<String, ReadError> {
        Ok(String::from_utf8(
            Get::<u16>::get(self)?.to_be_bytes().into(),
        )?)
    }
}

impl Get<u8> for Device {
    fn get(&mut self) -> Result<u8, ReadError> {
        let mut rbuff = [0u8; 1];
        self.serialport.read_exact(&mut rbuff)?;
        self.read_bytes += 1;
        self.read_checksum.update(&rbuff);
        Ok(rbuff[0])
    }

    fn get_string(&mut self) -> Result<String, ReadError> {
        Ok(String::from_utf8(
            Get::<u8>::get(self)?.to_be_bytes().into(),
        )?)
    }
}

impl Get<bool> for Device {
    fn get(&mut self) -> Result<bool, ReadError> {
        let mut rbuff = [0u8; 1];
        self.serialport.read_exact(&mut rbuff)?;
        self.read_bytes += 1;
        self.read_checksum.update(&rbuff);
        if rbuff[0] == 0 {
            Ok(false)
        } else if rbuff[0] == 1 {
            Ok(true)
        } else {
            Err(ReadError::ParseError(
                "Boolean must be 0 for true, 1 for false and nothing else".to_string(),
            ))
        }
    }

    fn get_string(&mut self) -> Result<String, ReadError> {
        Ok(String::from_utf8(
            Get::<u8>::get(self)?.to_be_bytes().into(),
        )?)
    }
}

/// Contains the device type and revision
#[derive(Debug, Display)]
#[allow(unused)]
#[display(
    fmt = "ModInfoResp {{ device_type: {}, revision: {} }}",
    device_type,
    revision
)]
pub struct ModInfoResp {
    /// Device Type
    pub device_type: String,

    /// Device Version
    pub revision: String,
}
