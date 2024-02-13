/// Centered around the [Get] trait
pub mod responses;

/// The second byte of a frame
pub mod command;

/// Configuration + config options
pub mod config;

/// Acquisition of data
pub mod acquisition;

/// User + factory device calibration
pub mod calibration;

use serialport::SerialPort;
use std::{error::Error, hash::Hasher, string::FromUtf8Error, time::Duration};
#[macro_use]
extern crate derive_more;

use command::Command;
use responses::{Get, ModInfoResp};


/// Error that ocurred while reading data back from the device
#[derive(Debug, Display)]
pub enum ReadError {
    /// IO Error when communicating with device on serial port.
    PipeError(std::io::Error),

    /// Error parsing response/data from device
    ParseError(String),

    /// Checksum for frame didn't match
    #[display(
        fmt = "ChecksumMismatch {{ expected: {}, actual: {} }}",
        expected,
        actual
    )]
    ChecksumMismatch { expected: u16, actual: u16 }, // in case of misaligned read, return the
    // actual checksum for easy debugging
    /// Frame length was different from expected length, check device compatibility or library
    /// version. Size mismatches result in a PipeError if the frame was shorter than expected
    /// and a read timed out
    #[display(fmt = "SizeMismatch {{ expected: {}, actual: {} }}", expected, actual)]
    SizeMismatch { expected: u16, actual: u16 },
}

impl Error for ReadError {}

impl From<std::io::Error> for ReadError {
    fn from(value: std::io::Error) -> Self {
        Self::PipeError(value)
    }
}

impl From<FromUtf8Error> for ReadError {
    fn from(e: FromUtf8Error) -> Self {
        Self::ParseError(format!("UTF8 String couldn't be parsed: {}", e))
    }
}

/// Error that ocurred while writing data to the device
#[derive(Debug, Display)]
pub enum WriteError {
    /// IO Error when writing to device
    PipeError(std::io::Error),
}

impl Error for WriteError {}

impl From<std::io::Error> for WriteError {
    fn from(value: std::io::Error) -> Self {
        Self::PipeError(value)
    }
}

#[derive(Debug, Display)]
pub enum RWError {
    /// Error occurred when reading/parsing data from serial
    ReadError(ReadError),

    /// Error occurred when writing/serializing data to serial
    WriteError(WriteError),

    /// Device indicated error status
    DeviceError(String),
}

impl Error for RWError {}

impl From<WriteError> for RWError {
    fn from(value: WriteError) -> Self {
        Self::WriteError(value)
    }
}

impl From<ReadError> for RWError {
    fn from(value: ReadError) -> Self {
        Self::ReadError(value)
    }
}

/// Represents a connected TargetPoint3 device
///
/// # Examples
///
/// ```
/// # {
/// use targetpoint3::{TargetPoint3, DataID};
/// let mut tp3 = targetpoint3::TargetPoint3::connect(None).expect("Couldn't Auto-Detect connected TargetPoint3");
/// tp3.set_data_components(vec![DataID::AccelX]);
/// println!("Accel X: {}", tp3.get_data().unwrap().accel_x.unwrap());
/// # }
/// ```
pub struct TargetPoint3 {
    serialport: Box<dyn SerialPort>,

    /// Checksum of the current frame so far
    read_checksum: crc16::State<crc16::XMODEM>,

    /// # of bytes read since the frame started
    read_bytes: u16,
}

impl TargetPoint3 {
    /// Creates a new TargetPoint3 with provided serialport
    pub fn new(serialport: impl Into<Box<dyn SerialPort>>) -> Self {
        Self {
            serialport: serialport.into(),
            read_checksum: crc16::State::<crc16::XMODEM>::new(),
            read_bytes: 0,
        }
    }

    /// Creates and connects to a TargetPoint3, auto-detecting the serial port, and choosing the
    /// default baud rate of 38400
    ///
    /// # Arguments
    ///
    /// * `port` - If [Some], uses the given serial port string. If [None], tries to auto-detect
    ///
    /// # Examples
    ///
    /// ```
    /// # {
    /// let tp3 = targetpoint3::TargetPoint3::connect(None).expect("Auto-Detect connected TargetPoint3");
    /// # }
    /// ```
    pub fn connect(port: Option<String>) -> Result<Self, Box<dyn Error>> {
        let ports = serialport::available_ports()?;

        let port = if let Some(provided_port) = port {
            provided_port
        } else {
            match ports.into_iter().fold(None, |chosen, port| {
                if port.port_name.contains("usb") {
                    Some(port)
                } else {
                    chosen
                }
            }) {
                Some(port) => port.port_name,
                None => {
                    return Err(Box::new(serialport::Error::new(
                        serialport::ErrorKind::NoDevice,
                        "Could not auto-detect serial port",
                    )))
                }
            }
        };

        println!("Using port {}", port);

        Ok(TargetPoint3::new(
            serialport::new(port, 38400)
                .data_bits(serialport::DataBits::Eight)
                .stop_bits(serialport::StopBits::One)
                .parity(serialport::Parity::None)
                .timeout(Duration::new(1, 0))
                .open()?,
        ))
    }

    /// Sends the given command and payload to the device, with appropriate CRC and sizing
    pub fn write_frame(
        &mut self,
        command: Command,
        payload: Option<&[u8]>,
    ) -> Result<(), WriteError> {
        let payload_length = if let Some(payload) = payload {
            payload.len() as u16
        } else {
            0
        };

        // offset of 5 comes from 2 length bytes, 1 command byte, 2 crc bytes
        let size = (payload_length + 5u16).to_be_bytes();
        let command = command.discriminant().to_be_bytes();

        // if you are porting this to another language, note the CRC algorithm XMODEM may also be
        // called CCITT or ITU, but is different from CCITT-FALSE and AUG-CCITT
        let mut crc = crc16::State::<crc16::XMODEM>::new();

        // write packet size
        self.serialport.write(&size)?;
        crc.update(&size);

        // write command
        self.serialport.write(&command)?;
        crc.update(&command);

        if let Some(payload_bytes) = payload {
            // write payload
            self.serialport.write(payload_bytes)?;
            crc.update(payload_bytes);
        }

        // finish and write CRC
        let crc = &(crc.finish() as u16).to_be_bytes();
        self.serialport.write(crc)?;

        Ok(())
    }

    /// Reads, checks then resets checksum when reading a frame.
    /// Must be called at the end of every frame to reset counters and crc
    fn end_frame(&mut self, expected_frame_len: u16) -> Result<(), ReadError> {
        // must compute expected sum before reading the checksum, since reading the checksum
        // updates the hasher
        let expected_sum = self.read_checksum.finish() as u16;
        let checksum: u16 = Get::<u16>::get(self)?;

        // reset checksum (though it should auto-reset to zero...).
        self.read_checksum = crc16::State::<crc16::XMODEM>::new();

        if expected_sum == checksum && self.read_bytes == expected_frame_len {
            self.read_bytes = 0;
            Ok(())
        } else if self.read_bytes != expected_frame_len {
            let read_bytes = self.read_bytes;
            self.read_bytes = 0;
            Err(ReadError::SizeMismatch {
                expected: expected_frame_len,
                actual: read_bytes,
            })
        } else {
            self.read_bytes = 0;
            Err(ReadError::ChecksumMismatch {
                expected: expected_sum,
                actual: checksum,
            })
        }
    }

    /// Returns device type and revision
    pub fn get_mod_info(&mut self) -> Result<ModInfoResp, RWError> {
        self.write_frame(Command::GetModInfo, None)?;
        let expected_size = Get::<u16>::get(self)?;
        if Get::<u8>::get(self)? == Command::GetModInfoResp.discriminant() {
            let device_type = Get::<u32>::get_string(self)?;
            let revision = Get::<u32>::get_string(self)?;
            self.end_frame(expected_size)?;
            Ok(ModInfoResp {
                device_type,
                revision,
            })
        } else {
            let _ = self.end_frame(expected_size);
            Err(RWError::ReadError(ReadError::ParseError(
                "Unexpected response type".to_string(),
            )))
        }
    }

    /// Returns device serial number, which can also be found on the front sticker
    pub fn serial_number(&mut self) -> Result<u32, RWError> {
        self.write_frame(Command::SerialNumber, None)?;
        let expected_size = Get::<u16>::get(self)?;
        if Get::<u8>::get(self)? == Command::SerialNumberResp.discriminant() {
            let serial_number = Get::<u32>::get(self)?;
            self.end_frame(expected_size)?;
            Ok(serial_number)
        } else {
            let _ = self.end_frame(expected_size);
            Err(RWError::ReadError(ReadError::ParseError(
                "Unexpected response type".to_string(),
            )))
        }
    }

    /// This frame commands the TargetPoint3 to save internal configurations and user calibration to non-volatile memory. Internal configurations and user calibration are restored on power up. The frame has no payload. This is the ONLY command that causes the device to save information to non-volatile memory.
    /// See also: [TargetPoint3::get_config], [TargetPoint3::set_config]
    pub fn save(&mut self) -> Result<(), RWError> {
        self.write_frame(Command::Save, None)?;

        let expected_size = Get::<u16>::get(self)?;
        if Get::<u8>::get(self)? == Command::SaveDone.discriminant() {
            let error_code = Get::<u16>::get(self)?;
            self.end_frame(expected_size)?;
            if error_code != 0 {
                return Err(RWError::DeviceError(
                    "Recieved error code from device, settings not saved succesfully".to_string(),
                ));
            }
            Ok(())
        } else {
            let _ = self.end_frame(expected_size);
            Err(RWError::ReadError(ReadError::ParseError(
                "Unexpected response type".to_string(),
            )))
        }
    }

    /// "Powers up" the device by sending data over serial (asks for SerialPort) Consumes the power up packet emitted by the device, useful to call after you call
    /// power_down and reconnect the device
    pub fn power_up(&mut self) -> Result<(), RWError> {
        self.write_frame(Command::SerialNumber, None)?;

        let expected_size = Get::<u16>::get(self)?;
        let resp_command = Get::<u8>::get(self)?;

        if resp_command == Command::PowerUpDone.discriminant() {
            self.end_frame(expected_size)?;
            Ok(())
        } else if resp_command == Command::SerialNumberResp.discriminant() {
            // if the device is already powered up or if it did buffering of the wake-up command,
            // we might actually get the serial number back!
            Get::<u32>::get(self)?;
            self.end_frame(expected_size)?;
            Ok(())
        } else {
            let _ = self.end_frame(expected_size);
            Err(RWError::ReadError(ReadError::ParseError(
                "Unexpected response type".to_string(),
            )))
        }
    }

    /// This frame is used to power-down the module. The frame has no payload. The command will power down all peripherals including the sensors, microprocessor, and RS-232 driver. However, the driver chip has a feature to keep the Rx line enabled. The TargetPoint3 will power up when it receives any signal on the native UART Rx line.
    /// This frame frequently does not recieve a response even when it works, it's suggested that
    /// you ignore ParseErrors
    fn power_down_impl(&mut self) -> Result<(), RWError> {
        self.write_frame(Command::PowerDown, None)?;

        let expected_size = Get::<u16>::get(self)?;
        if Get::<u8>::get(self)? == Command::PowerDownDone.discriminant() {
            self.end_frame(expected_size)?;
            Ok(())
        } else {
            let _ = self.end_frame(expected_size);
            Err(RWError::ReadError(ReadError::ParseError(
                "Unexpected response type".to_string(),
            )))
        }
    }
    
    /// You should consider using [Self::power_down] instead of [Self::power_down_raw] to avoid
    /// weird serialport behavior
    ///
    /// This frame is used to power-down the module. The frame has no payload. The command will power down all peripherals including the sensors, microprocessor, and RS-232 driver. However, the driver chip has a feature to keep the Rx line enabled. The TargetPoint3 will power up when it receives any signal on the native UART Rx line.
    /// This frame frequently does not recieve a response even when it works, it's suggested that
    /// you ignore ParseErrors
    #[cfg(feature = "reserved")]
    pub fn power_down_raw(&mut self) -> Result<(), RWError> {
        self.power_down_impl()
    }

    //NOTE: when powering up, we want to connect to the same device in case multiple devices were
    //provided? Otherwise we basically force the end user to deliberately re-choose the new device
    //anyhow by re-constructing tp3. Consuming self in power down also drops the serial port which
    //is desireable
    /// This frame is used to power-down the module. The frame has no payload. The command will power down all peripherals including the sensors, microprocessor, and RS-232 driver. However, the driver chip has a feature to keep the Rx line enabled. The TargetPoint3 will power up when it receives any signal on the native UART Rx line.
    /// Similar to power_down_raw, but ignores common errors due to power down, and takes ownership to hang up the socket and force developer to create a new tp3 object
    /// The very action of reconnecting the device will cause it to power back up.
    pub fn power_down(mut self) -> Result<(), RWError> {
        let ret = match self.power_down_impl() {
            Ok(_) => Ok(()),
            Err(RWError::ReadError(_)) => Ok(()),
            Err(e) => Err(e),
        };
        ret
    }
}

// NOTE: when testing or writing doctests, be sure to put everything in its own scope so that the
// serialport is dropped afte each test
#[cfg(test)]
mod tests {
    use crate::acquisition::*;
    use crate::*;

    #[test]
    fn continuous_mode() {
        let tp3 = TargetPoint3::connect(None).expect("connects to device");
        let mut tp3 = tp3
            .easy_continuous_mode(0.25, vec![DataID::AccelX])
            .expect("got into cont mode");
        {
            let mut iter = tp3.iter();
            for _ in 0..16 {
                assert!(match iter.next() { Some(Ok(Data { accel_x: Some(_accel_measurement), ..})) => true, _ => false }, "Calling next on interator in continuous mode should yield the data we asked for");
            }
        }

        let mut tp3 = tp3.easy_stop_continuous_mode().unwrap();
        {
            let mut iter = tp3.iter();
            assert!(
                match iter.next() {
                    None => true,
                    _ => false,
                },
                "Stop continious mode should leave continuous mode"
            )
        }
    }
}
