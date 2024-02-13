use serialport::SerialPort;
use std::{error::Error, hash::Hasher, string::FromUtf8Error, time::Duration};
#[macro_use]
extern crate derive_more;

//TODO async
//links in docs
//call endframe for all errors and proxy them up, probably RAII pattern will help here
//nicer wrappers for stuff like calibration (to keep track of sample points) and other higher-level abstractions

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

//TODO: Derive
/// Represents a datastream that can emit out a `T`
pub trait Get<T> {
    /// Blocks on device until we recieve enough data to parse `T`
    fn get(&mut self) -> Result<T, ReadError>;

    /// Same as get, except gets a String of bytes `T`
    /// If not a primitive type, returns the to_string of the type
    fn get_string(&mut self) -> Result<String, ReadError>;
}

/// The type of command being sent/recieved from the device. All frames have a command.
#[repr(u8)]
pub enum Command {
    /// Queries the device’s type and firmware revision.
    GetModInfo = 0x01,

    /// Response to GetModInfo
    GetModInfoResp = 0x02,

    /// Sets the data components to be output
    SetDataComponents = 0x03,

    /// Queries the TargetPoint3 for data
    GetData = 0x04,

    /// Response to GetData
    GetDataResp = 0x05,

    /// Sets internal configurations in TargetPoint3
    SetConfig = 0x06,

    /// Queries TargetPoint3 for the current internal configuration
    GetConfig = 0x07,

    /// Response to GetConfig
    GetConfigResp = 0x08,

    /// Saves the current internal configuration and any new user calibration coefficients to non- volatile memory.
    Save = 0x09,

    /// Commands the TargetPoint3 to start user calibratio
    StartCal = 0x0A,

    /// Commands the TargetPoint3 to stop user calibration
    StopCal = 0x0B,

    /// Sets the FIR filter settings for the magnetometer & accelerometer sensors.
    SetFIRFilters = 0x0C,

    /// Queries for the FIR filter settings for the magnetometer & accelerometer sensors.
    GetFIRFilters = 0x0D,

    /// Contains the FIR filter settings for the magnetometer & accelerometer sensors.
    GetFIRFiltersResp = 0x0E,

    /// Powers down the module
    PowerDown = 0x0F,

    /// Response to kSave
    SaveDone = 0x10,

    /// Sent from the TargetPoint3 after taking a calibration sample point
    UserCalSampleCount = 0x11,

    /// Contains the calibration score
    UserCalScore = 0x12,

    /// Response to SetConfig
    SetConfigDone = 0x13,

    /// Response to SetFIRFilters
    SetFIRFiltersDone = 0x14,

    /// Commands the TargetPoint3 to output data at a fixed interval
    StartContinuousMode = 0x15,

    /// Stops data output when in Continuous Mode
    StopContinuousMode = 0x16,

    /// Confirms the TargetPoint3 has received a signal to power up
    PowerUpDone = 0x17,

    /// Sets the sensor acquisition parameters
    SetAcqParams = 0x18,

    /// Queries for the sensor acquisition parameters
    GetAcqParams = 0x19,

    /// Response to SetAcqParams
    SetAcqParamsDone = 0x1A,

    /// Response to GetAcqParams
    GetAcqParamsResp = 0x1B,

    /// Response to PowerDown
    PowerDownDone = 0x1C,

    /// Resets magnetometer calibration coefficients to original factory-established values
    FactoryMagCoeff = 0x1D,

    /// Response to kFactoryMagCoeff
    FactoryMagCoeffDone = 0x1E,

    /// Commands the TargetPoint3 to take a sample during user calibration
    TakeUserCalSample = 0x1F,

    /// Resets accelerometer calibration coefficients to original factory-established values
    FactorylAccelCoeff = 0x24,

    /// Respond to FactoryAccelCoeff
    FactoryAccelCoeffDone = 0x25,

    /// Copy one set of calibration coefficient to another set
    CopyCoeffSet = 0x2B,

    /// Respond to CopyCoeffSet
    CopyCoeffSetDone = 0x2C,

    /// Request Serial Number of TargetPoint3 unit
    SerialNumber = 0x34,

    /// Respond to SerialNumber
    SerialNumberResp = 0x35,
}

impl Command {
    // [unsafe]: This code pulls the integer representation of the enum, since the enum is repr(u8)
    // and the u8 is the first element in the enum, the pointer cast will work. Additionally, this
    // pattern has been directly copied from the rust documentation for error codes, with modification
    // only to its parameters and return values
    // src: https://github.com/rust-lang/rust/blob/master/compiler/rustc_error_codes/src/error_codes/E0732.md
    fn discriminant(&self) -> u8 {
        unsafe { *(self as *const Self as *const u8) }
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

    /// Sets configuration on device, without saving to volatile memory. These configurations can only be set one at time.
    /// To save these in non-volatile memory, call [TargetPoint3::save].
    /// See also: [TargetPoint3::get_config]
    ///
    /// # Arguments
    /// * `config_option` - Configuration parameter and value to set
    pub fn set_config(&mut self, config_option: ConfigPair) -> Result<(), RWError> {
        let payload = Vec::<u8>::from(config_option);
        self.write_frame(Command::SetConfig, Some(&payload))?;

        let expected_size = Get::<u16>::get(self)?;
        if Get::<u8>::get(self)? == Command::SetConfigDone.discriminant() {
            self.end_frame(expected_size)?;
            Ok(())
        } else {
            let _ = self.end_frame(expected_size);
            Err(RWError::ReadError(ReadError::ParseError(
                "Unexpected response type".to_string(),
            )))
        }
    }

    /// This frame queries the TargetPoint3 for the current internal configuration value.
    ///
    /// # Arguments
    /// * `id` - The configuration parameter to query
    pub fn get_config(&mut self, id: ConfigID) -> Result<ConfigPair, RWError> {
        self.write_frame(Command::GetConfig, Some(&[id.clone() as u8]))?;

        let expected_size = Get::<u16>::get(self)?;
        if Get::<u8>::get(self)? == Command::GetConfigResp.discriminant() {
            match id {
                ConfigID::Declination => {
                    let setting = ConfigPair::Declination(Get::<f32>::get(self)?);
                    self.end_frame(expected_size)?;
                    Ok(setting)
                }
                ConfigID::TrueNorth => {
                    let setting = ConfigPair::TrueNorth(Get::<bool>::get(self)?);
                    self.end_frame(expected_size)?;
                    Ok(setting)
                }
                ConfigID::BigEndian => {
                    let setting = ConfigPair::BigEndian(Get::<bool>::get(self)?);
                    self.end_frame(expected_size)?;
                    Ok(setting)
                }
                ConfigID::MountingRef => {
                    let setting = ConfigPair::MountingRef(Get::<MountingRef>::get(self)?);
                    self.end_frame(expected_size)?;
                    Ok(setting)
                }
                ConfigID::UserCalNumPoints => {
                    let setting = ConfigPair::UserCalNumPoints(Get::<u32>::get(self)?);
                    self.end_frame(expected_size)?;
                    Ok(setting)
                }
                ConfigID::UserCalAutoSampling => {
                    let setting = ConfigPair::UserCalAutoSampling(Get::<bool>::get(self)?);
                    self.end_frame(expected_size)?;
                    Ok(setting)
                }
                ConfigID::BaudRate => {
                    let setting = ConfigPair::BaudRate(Get::<Baud>::get(self)?);
                    self.end_frame(expected_size)?;
                    Ok(setting)
                }
                ConfigID::MilOut => {
                    let setting = ConfigPair::MilOut(Get::<bool>::get(self)?);
                    self.end_frame(expected_size)?;
                    Ok(setting)
                }
                ConfigID::HPRDuringCal => {
                    let setting = ConfigPair::HPRDuringCal(Get::<bool>::get(self)?);
                    self.end_frame(expected_size)?;
                    Ok(setting)
                }
                ConfigID::MagCoeffSet => {
                    let setting = ConfigPair::MagCoeffSet(Get::<u32>::get(self)?);
                    self.end_frame(expected_size)?;
                    Ok(setting)
                }
                ConfigID::AccelCoeffSet => {
                    let setting = ConfigPair::AccelCoeffSet(Get::<u32>::get(self)?);
                    self.end_frame(expected_size)?;
                    Ok(setting)
                }
            }
        } else {
            let _ = self.end_frame(expected_size);
            Err(RWError::ReadError(ReadError::ParseError(
                "Unexpcted response type".to_string(),
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

    /// This frame sets the sensor acquisition parameters in the TargetPoint3.
    ///
    /// # Arguments
    /// * `acq_params` - Parameters to set for next acquisition
    pub fn set_acq_params(&mut self, acq_params: AcqParams) -> Result<(), RWError> {
        self.set_acq_params_reserved(AcqParamsReserved {
            acquisition_mode: acq_params.acquisition_mode,
            flush_filter: acq_params.flush_filter,
            reserved: f32::from_be_bytes([0u8, 0u8, 0u8, 0u8]),
            sample_delay: acq_params.sample_delay,
        })
    }

    /// Like set_acq_parameters, but gives the user the ability to write to the PNI reserved
    /// fields. Note different parameter ordering (done to reflect order inside payload)
    /// Confused? Just use set_acq_parameters
    pub fn set_acq_params_reserved(
        &mut self,
        acq_params: AcqParamsReserved,
    ) -> Result<(), RWError> {
        let mut payload = Vec::<u8>::new();
        payload.push(if acq_params.acquisition_mode { 1 } else { 0 });
        payload.push(if acq_params.flush_filter { 1 } else { 0 });
        payload.extend_from_slice(&acq_params.reserved.to_be_bytes());
        payload.extend_from_slice(&acq_params.sample_delay.to_be_bytes());
        self.write_frame(Command::SetAcqParams, Some(&payload))?;

        let expected_size = Get::<u16>::get(self)?;
        if Get::<u8>::get(self)? == Command::SetAcqParamsDone.discriminant() {
            self.end_frame(expected_size)?;
            Ok(())
        } else {
            let _ = self.end_frame(expected_size);
            Err(RWError::ReadError(ReadError::ParseError(
                "Unexpected response type".to_string(),
            )))
        }
    }

    /// Same as get_acq_params, but instead returns a tuple whose first value are the AcqParams and
    /// whose second value are the reserved bits
    pub fn get_acq_params_reserved(&mut self) -> Result<AcqParamsReserved, RWError> {
        self.write_frame(Command::GetAcqParams, None)?;

        let expected_size = Get::<u16>::get(self)?;
        if Get::<u8>::get(self)? == Command::GetAcqParamsResp.discriminant() {
            let acquisition_mode = Get::<bool>::get(self)?;
            let flush_filter = Get::<bool>::get(self)?;
            let reserved = Get::<f32>::get(self)?;
            let sample_delay = Get::<f32>::get(self)?;
            self.end_frame(expected_size)?;
            Ok(AcqParamsReserved {
                acquisition_mode,
                flush_filter,
                reserved,
                sample_delay,
            })
        } else {
            let _ = self.end_frame(expected_size);
            Err(RWError::ReadError(ReadError::ParseError(
                "Unexpected response type".to_string(),
            )))
        }
    }

    /// This frame queries the unit for acquisition parameters.
    pub fn get_acq_params(&mut self) -> Result<AcqParams, RWError> {
        Ok(self.get_acq_params_reserved()?.into())
    }

    /// This frame defines what data is output when GetData is sent. Table 7-5 in the user manual summarizes the various data components and more detail follows this table. Note that this is not a query for the device's model type and software revision (see GetModInfo). The first byte of the payload indicates the number of data components followed by the data component IDs. Note that the sequence of the data components defined by SetDataComponents will match the output sequence of GetDataResp.
    ///
    /// # Arguments
    ///
    /// * `components` - List of dimensions (measurements) to get back on subsequent get_data
    /// responses, or during continuous mode after the device is rebooted
    pub fn set_data_components(&mut self, components: Vec<DataID>) -> Result<(), RWError> {
        let mut payload = Vec::<u8>::new();
        payload.push(components.len() as u8);
        for component in components.into_iter() {
            payload.push(component as u8);
        }
        self.write_frame(Command::SetDataComponents, Some(&payload))?;
        Ok(())
    }

    /// If the TargetPoint3 is configured to operate in Polled Acquisition Mode (see SetAcqParams), then this frame requests a single measurement data set. The frame has no payload.
    pub fn get_data(&mut self) -> Result<Data, RWError> {
        self.write_frame(Command::GetData, None)?;

        let expected_size = Get::<u16>::get(self)?;
        if Get::<u8>::get(self)? == Command::GetDataResp.discriminant() {
            let data = Get::<Data>::get(self)?;
            self.end_frame(expected_size)?;
            Ok(data)
        } else {
            let _ = self.end_frame(expected_size);
            Err(RWError::ReadError(ReadError::ParseError(
                "Unexpected response type".to_string(),
            )))
        }
    }

    /// If the TargetPoint3 is configured to operate in Continuous Acquisition Mode (see SetAcqParams), then this frame initiates the outputting of data at a relatively fixed data rate, where the data rate is established by the SampleDelay parameter. The frame has no payload.
    /// You must call [TargetPoint3::set_acq_params] and [TargetPoint3::set_data_components] before calling [TargetPoint3::set_continuous_mode], and call [TargetPoint3::save]
    /// and power cycle the device in order to start continuous output
    ///
    /// # Examples
    /// ```
    /// # use targetpoint3::*;
    /// # {
    /// # let mut tp3 = TargetPoint3::connect(None).unwrap();
    /// tp3.set_acq_params(AcqParams { acquisition_mode: false, flush_filter: false, sample_delay: 0.2 }).unwrap();
    /// tp3.set_data_components(vec![DataID::AccelX]).unwrap();
    /// tp3.save().unwrap();
    /// tp3.start_continuous_mode_raw().unwrap();
    /// tp3.power_down().unwrap();
    /// let mut tp3 = TargetPoint3::connect(None).unwrap();
    /// tp3.power_up().unwrap();
    /// tp3.stop_continuous_mode_raw().unwrap();
    /// tp3.save().unwrap();
    /// tp3.power_down().unwrap();
    /// tp3 = TargetPoint3::connect(None).unwrap();
    /// tp3.power_up().unwrap();
    /// # }
    /// ```
    pub fn start_continuous_mode_raw(&mut self) -> Result<(), RWError> {
        self.write_frame(Command::StartContinuousMode, None)?;
        Ok(())
    }

    /// This frame commands the TargetPoint3 to stop data output when in Continuous Acquisition Mode. The frame has no payload.
    /// You must call [TargetPoint3::save] and power cycle the device after calling [TargetPoint3::stop_continuous_mode] to stop continuous output
    pub fn stop_continuous_mode_raw(&mut self) -> Result<(), RWError> {
        self.write_frame(Command::StopContinuousMode, None)?;
        Ok(())
    }

    /// **Note:** Please only use this if your use-case absolutely needs it. We suggest using [TargetPoint3::power_down]
    /// instead
    ///
    /// This frame is used to power-down the module. The frame has no payload. The command will power down all peripherals including the sensors, microprocessor, and RS-232 driver. However, the driver chip has a feature to keep the Rx line enabled. The TargetPoint3 will power up when it receives any signal on the native UART Rx line.
    /// This frame frequently does not recieve a response even when it works, it's suggested that
    /// you ignore ParseErrors
    pub fn power_down_raw(&mut self) -> Result<(), RWError> {
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

    //NOTE: when powering up, we want to connect to the same device in case multiple devices were
    //provided? Otherwise we basically force the end user to deliberately re-choose the new device
    //anyhow by re-constructing tp3. Consuming self in power down also drops the serial port which
    //is desireable
    /// This frame is used to power-down the module. The frame has no payload. The command will power down all peripherals including the sensors, microprocessor, and RS-232 driver. However, the driver chip has a feature to keep the Rx line enabled. The TargetPoint3 will power up when it receives any signal on the native UART Rx line.
    /// Similar to power_down_raw, but ignores common errors due to power down, and takes ownership to hang up the socket and force developer to create a new tp3 object
    /// The very action of reconnecting the device will cause it to power back up.
    pub fn power_down(mut self) -> Result<(), RWError> {
        let ret = match self.power_down_raw() {
            Ok(_) => Ok(()),
            Err(RWError::ReadError(_)) => Ok(()),
            Err(e) => Err(e),
        };
        ret
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

    /// Convenience wrapper around several functions to make it easier to put the device in continuous mode. Simply call [TargetPoint3.iter()] on the returned tp3 struct to get continuous data
    /// If the device is already in continious mode, this and other commands may fail to read
    /// responses. You should call [TargetPoint3::stop_continuous_mode_raw] (then power cycle) or [TargetPoint3::easy_stop_continuous_mode] before trying to issue other commands.
    ///
    /// # Violated Contracts
    /// Calling this will freely change several configuration settings (including AcqParams) to
    /// sensible defaults and save them, along with any other device settings currently in volatile memory to non-volatile memory.
    ///
    /// This function will also re-construct [TargetPoint3] by auto-detecting the serial port,
    /// meaning it is not compatible with your use case if you have multiple devices connected at the same time, or if auto-detection failed and you manually provided a [SerialPort] or provided a serial port descriptor string to the constructor
    ///
    /// # For predictable behavior
    /// If you do not want more predictable behavior that doesn't violate these contracts, you may
    /// use [TargetPoint3::set_acq_params], [TargetPoint3::set_data_components], [TargetPoint3::start_continuous_mode_raw], [TargetPoint3::power_down], and
    /// [TargetPoint3::power_up] in that order. See user manual for more help.
    ///
    /// # Arguments
    /// * `sample_delay` - Time, in seconds, between samples. See SetAcqParams command in user
    /// manual for nuances
    /// * `data_components` - List of data types to acquire from device
    pub fn easy_continuous_mode(
        mut self,
        sample_delay: f32,
        data_components: Vec<DataID>,
    ) -> Result<Self, Box<dyn Error>> {
        self.set_acq_params(AcqParams {
            acquisition_mode: false,
            flush_filter: false,
            sample_delay,
        })?;
        self.set_data_components(data_components)?;
        self.save()?;
        self.start_continuous_mode_raw()?;
        self.power_down()?;
        let mut newtp3 = TargetPoint3::connect(None)?;
        newtp3.power_up()?;

        Ok(newtp3)
    }

    /// Convenience wrapper around several functions to make it easier to take the device out of continuous mode. See [TargetPoint3::easy_continuous_mode]
    ///
    /// # Violated Contracts
    /// Calling this may freely change several configuration settings (including AcqParams) to
    /// sensible defaults and save them, along with any other device settings currently in volatile memory to non-volatile memory.
    ///
    /// This function will also re-construct [TargetPoint3] by auto-detecting the serial port,
    /// meaning it is not compatible with your use case if you have multiple devices connected at the same time, or if auto-detection failed and you manually provided a [SerialPort] or provided a serial port descriptor string to the constructor
    ///
    /// # For predictable behavior
    /// If you do not want more predictable behavior that doesn't violate these contracts, you may
    /// use [TargetPoint3::set_acq_params], TargetPoint3::stop_continuous_mode_raw], [TargetPoint3::power_down], and
    /// [TargetPoint3::power_up] in that order. See user manual for more help.
    pub fn easy_stop_continuous_mode(mut self) -> Result<Self, Box<dyn Error>> {
        //self.set_acq_params(AcqParams { acquisition_mode: true, flush_filter: false, sample_delay: 0f32 })?;
        self.stop_continuous_mode_raw()?;
        self.save()?;
        self.power_down()?;
        let mut newtp3 = TargetPoint3::connect(None)?;
        newtp3.power_up()?;
        Ok(newtp3)
    }

    pub fn iter<'a>(&'a mut self) -> impl Iterator<Item = Result<Data, ReadError>> + 'a {
        ContinuousModeIterator(self)
    }

    /// First, note that in order to perform a user calibration, it is necessary to place the TargetPoint3 in Compass Mode, as discussed in User Manual Section 7.7. Note that TargetPoint3 allows for a maximum of 18 calibration points.
    /// See User Manual for calibration instructions.
    /// This frame commands the TargetPoint3 to start user calibration with the current sensor acquisition parameters, internal configurations, and FIR filter settings.
    ///
    /// Returns the sample count, which should be 0 when starting a calibration
    pub fn start_cal(&mut self, calibration_type: CalOption) -> Result<u32, RWError> {
        self.write_frame(
            Command::StartCal,
            Some(&(calibration_type as u32).to_be_bytes()),
        )?;

        let expected_size = Get::<u16>::get(self)?;
        let resp_command = Get::<u8>::get(self)?;

        if resp_command == Command::UserCalSampleCount.discriminant() {
            let sample_count = Get::<u32>::get(self)?;
            self.end_frame(expected_size)?;
            Ok(sample_count)
        } else {
            let _ = self.end_frame(expected_size);
            Err(RWError::ReadError(ReadError::ParseError(format!(
                "Unexpected response type. Got {}",
                resp_command
            ))))
        }
    }

    /// This frame commands the TargetPoint3 to take a sample during user calibration.
    ///
    /// Returns the sample count, unless this is the last sample point, in which case returns the calibration score.
    /// If the sample was succesful, calibration should return 1 more
    /// than the previous sample count (or return the score)
    pub fn take_user_cal_sample(&mut self) -> Result<UserCalResponse, RWError> {
        self.write_frame(Command::TakeUserCalSample, None)?;

        let expected_size = Get::<u16>::get(self)?;
        let resp_command = Get::<u8>::get(self)?;

        if resp_command == Command::UserCalSampleCount.discriminant() {
            let sample_count = Get::<u32>::get(self)?;
            self.end_frame(expected_size)?;
            Ok(UserCalResponse::SampleCount(sample_count))
        } else if resp_command == Command::UserCalScore.discriminant() {
            let ret = UserCalResponse::UserCalScore {
                mag_cal_score: Get::<f32>::get(self)?,
                reserved: Get::<f32>::get(self)?,
                accel_cal_score: Get::<f32>::get(self)?,
                distribution_error: Get::<f32>::get(self)?,
                tilt_error: Get::<f32>::get(self)?,
                tilt_range: Get::<f32>::get(self)?,
            };
            self.end_frame(expected_size)?;
            Ok(ret)
        } else {
            let _ = self.end_frame(expected_size);
            Err(RWError::ReadError(ReadError::ParseError(format!(
                "Unexpected response type. Got {}",
                resp_command
            ))))
        }
    }

    /// This command aborts the calibration process. The prior calibration results are retained.
    pub fn stop_cal_reserved(&mut self) -> Result<(), WriteError> {
        self.write_frame(Command::StopCal, None)?;
        Ok(())
    }

    /// This frame clears the magnetometer calibration coefficients and loads the original factory-generated coefficients. The frame has no payload. This frame must be followed by the kSave frame to save the change in non-volatile memory.
    pub fn factory_mag_coeff(&mut self) -> Result<(), RWError> {
        self.write_frame(Command::StartCal, None)?;

        let expected_size = Get::<u16>::get(self)?;
        let resp_command = Get::<u8>::get(self)?;

        if resp_command == Command::FactoryMagCoeffDone.discriminant() {
            self.end_frame(expected_size)?;
            Ok(())
        } else {
            let _ = self.end_frame(expected_size);
            Err(RWError::ReadError(ReadError::ParseError(format!(
                "Unexpected response type. Got {}",
                resp_command
            ))))
        }
    }

    /// This frame clears the accelerometer calibration coefficients and loads the original factory-generated coefficients. The frame has no payload. This frame must be followed by the kSave frame to save the change in non-volatile memory.
    pub fn factory_accel_coeff(&mut self) -> Result<(), RWError> {
        self.write_frame(Command::FactorylAccelCoeff, None)?;

        let expected_size = Get::<u16>::get(self)?;
        let resp_command = Get::<u8>::get(self)?;

        if resp_command == Command::FactoryAccelCoeffDone.discriminant() {
            self.end_frame(expected_size)?;
            Ok(())
        } else {
            let _ = self.end_frame(expected_size);
            Err(RWError::ReadError(ReadError::ParseError(format!(
                "Unexpected response type. Got {}",
                resp_command
            ))))
        }
    }

    /// This frame copies one set of calibration coefficients to another. TargetPoint3 supports 8 sets of magnetic calibration coefficients, and 8 sets of accel calibration coefficients. The set index is from 0 to 7. This frame must be followed by the kSave frame to save the change in non-volatile memory.
    ///
    /// # Arguments
    /// * `set_type` - Value 0 to copy magnetic calibration coefficient set (default), 1 to copy accel coefficient set
    /// * `set_indexes` - bit 7 - 4: source coefficient set index from 0 to 7, default 0, bit 0 - 3: destination coefficient set index from 0 to 7, default 0
    pub fn copy_coeff_set(&mut self, set_type: u8, set_indexes: u8) -> Result<(), RWError> {
        self.write_frame(Command::CopyCoeffSet, Some(&[set_type, set_indexes]))?;

        let expected_size = Get::<u16>::get(self)?;
        let resp_command = Get::<u8>::get(self)?;

        if resp_command == Command::CopyCoeffSetDone.discriminant() {
            self.end_frame(expected_size)?;
            Ok(())
        } else {
            let _ = self.end_frame(expected_size);
            Err(RWError::ReadError(ReadError::ParseError(format!(
                "Unexpected response type. Got {}",
                resp_command
            ))))
        }
    }

    /// The TargetPoint3 incorporates a finite impulse response (FIR) filter to provide a more stable heading reading. The number of taps (or samples) represents the amount of filtering to be performed. The number of taps directly affects the time for the initial sample reading, as all the taps must be populated before data is output.  The TargetPoint3 can be configured to clear, or flush, the filters after each measurement, as discussed in Section 7.5.1. Flushing the filter clears all tap values, thus purging old data.  This can be useful if a significant change in heading has occurred since the last reading, as the old heading data would be in the filter. Once the taps are cleared, it is necessary to fully repopulate the filter before data is output. For example, if 32 FIR-tap is set, 32 new samples must be taken before a reading will be output. The length of the delay before outputting data is directly correlated to the number of FIR taps.
    ///
    /// For recommended taps, see User Manual Table 7-6
    pub fn set_fir_filters(&mut self, taps: Vec<f64>) -> Result<(), RWError> {
        let mut payload =
            taps.into_iter()
                .map(|tap| tap.to_be_bytes())
                .fold(Vec::new(), |mut vec, tap| {
                    vec.extend(tap);
                    vec
                });

        // From manual: Byte 1 should be set to 3 and Byte 2 should be set to 1. Payload is
        // 1-indexed in docs
        payload.insert(0, 3);
        payload.insert(1, 1);
        self.write_frame(Command::SetFIRFilters, Some(&payload))?;

        let expected_size = Get::<u16>::get(self)?;
        let resp_command = Get::<u8>::get(self)?;

        if resp_command == Command::SetFIRFiltersDone.discriminant() {
            self.end_frame(expected_size)?;
            Ok(())
        } else {
            let _ = self.end_frame(expected_size);
            Err(RWError::ReadError(ReadError::ParseError(format!(
                "Unexpected response type. Got {}",
                resp_command
            ))))
        }
    }

    /// This frame queries the FIR filter settings for the sensors.
    /// For recommended taps, see User Manual Table 7-6
    pub fn get_fir_filters(&mut self) -> Result<Vec<f64>, RWError> {
        // From manual: Byte 1 should be set to 3 and Byte 2 should be set to 1.
        self.write_frame(Command::GetFIRFilters, Some(&[3, 1]))?;

        let expected_size = Get::<u16>::get(self)?;
        let resp_command = Get::<u8>::get(self)?;

        if resp_command == Command::SetFIRFiltersDone.discriminant() {
            let _byte_1 = Get::<u8>::get(self)?;
            let _byte_2 = Get::<u8>::get(self)?;

            let count = Get::<u8>::get(self)?;
            let mut taps = Vec::<f64>::new();
            for _ in 0..count {
                taps.push(Get::<f64>::get(self)?);
            }

            self.end_frame(expected_size)?;
            Ok(taps)
        } else {
            let _ = self.end_frame(expected_size);
            Err(RWError::ReadError(ReadError::ParseError(format!(
                "Unexpected response type. Got {}",
                resp_command
            ))))
        }
    }
}

pub enum UserCalResponse {
    /// The calibration score is automatically sent upon taking the final calibration point.
    UserCalScore {
        /// Represents the over-riding indicator of the quality of the magnetometer calibration.  Acceptable scores will be ≤1 for full range calibration, ≤2 for other methods. Note that it is possible to get acceptable scores for DistributionError and TiltError and still have a rather high MagCalScore value. The most likely reason for this is the TargetPoint3 is close to a source of local magnetic distortion that is not fixed with respect to the device.
        mag_cal_score: f32,

        /// Reserved for PNI use.
        reserved: f32,

        /// Represents the over-riding indicator of the quality of the accelerometer calibration.  An acceptable score is ≤1.
        accel_cal_score: f32,

        /// Indicates if the distribution of sample points is good, with an emphasis on the heading distribution. The score should be 0. Significant clumping or a lack of sample points in a particular section can result in a poor score.
        distribution_error: f32,

        /// Indicates if the TargetPoint3 experienced sufficient tilt during the calibration, taking into account the calibration method. The score should be 0.
        tilt_error: f32,

        /// This reports half the full pitch range of sample points. For example, if the device is pitched +25º to -15º, the TiltRange value would be 20º (as derived from [+25º - {-15º}]/2). For Full-Range Calibration and Hard-Iron-Only Calibration, this should be ≥30°. For 2D Calibration, ideally this should be ≈2°. For Limited-Tilt Calibration the value should be as large a possible given the user’s constraints.
        tilt_range: f32,
    },

    /// This frame is sent from the TargetPoint3 after taking a calibration sample point. The payload contains the sample count with the range of 0 to 32. Payload 0 is sent from TargetPoint3 after StartCal is received by TargetPoint3, it indicates user calibration start, and TargetPoint3 is ready to take samples. Payload 1 to 32 indicates each point sampled successfully.  SampleCount(u32)
    SampleCount(u32),
}

/// Type of calibration to use when calibrating device
#[derive(Debug, Display)]
pub enum CalOption {
    /// Default. Recommended calibration method when >30° of pitch is possible. Can be used for between 20° and 30° of pitch, but accuracy will not be as good
    FullRange = 10,

    /// Recommended when the available tilt range is limited to ≤5° . Can be used for 5° to 10° of tilt, but accuracy will not be as good.
    TwoDimensional = 20,

    /// Recalibrates the hard-iron offset for a prior calibration. If the local field hard-iron distortion has changed, this calibration can bring the TargetPoint3 back into specification.
    HardIronOnly = 30,

    /// Recommended calibration method when >5° of tilt calibration is available, but tilt is restricted to <30°. (i.e. full range calibration is not possible.)
    LimitedTilt = 40,

    /// Select this when an accelerometer calibration will be performed.
    AccelOnly = 100,

    /// Selected when magnetic and accelerometer calibration will be done simultaneously.
    MagAndAccel = 110,
}

impl Default for CalOption {
    fn default() -> Self {
        CalOption::FullRange
    }
}

pub struct ContinuousModeIterator<'a>(&'a mut TargetPoint3);

impl<'a> Iterator for ContinuousModeIterator<'a> {
    type Item = Result<Data, ReadError>;

    fn next(&mut self) -> Option<Self::Item> {
        let expected_size = match Get::<u16>::get(self.0) {
            Ok(size) => size,
            Err(ReadError::PipeError(ioerr)) if ioerr.kind() == std::io::ErrorKind::TimedOut => {
                return None;
            }
            Err(e) => {
                return Some(Err(e));
            }
        };

        let resp_command = match Get::<u8>::get(self.0) {
            Ok(command) => command,
            Err(e) => {
                return Some(Err(e));
            }
        };

        if resp_command == Command::GetDataResp.discriminant() {
            let data = match Get::<Data>::get(self.0) {
                Ok(command) => command,
                Err(e) => {
                    return Some(Err(e));
                }
            };
            match self.0.end_frame(expected_size) {
                Ok(_) => (),
                Err(e) => {
                    return Some(Err(e));
                }
            };

            Some(Ok(data))
        } else {
            let _ = self.0.end_frame(expected_size);
            Some(Err(ReadError::ParseError(
                "Unexpected response type".to_string(),
            )))
        }
    }
}

// for better developer experience, chose large struct with optionals instead of Vec<> of
// DataComponent's. Ths is memory inefficient.
/// Represents a data record from TP3. Use [TargetPoint3::set_data_components] to control which
/// fields to populate
#[derive(Debug, Display)]
#[display(
    fmt = "Data {{ heading: {:?}, pitch: {:?}, roll: {:?}, temperature: {:?}, distortion: {:?}, cal_status: {:?}, accel_x: {:?}, accel_y: {:?}, accel_z: {:?}, mag_x: {:?}, mag_y: {:?}, mag_z: {:?}, mag_accuracy: {:?} }}",
    heading,
    pitch,
    roll,
    temperature,
    distortion,
    cal_status,
    accel_x,
    accel_y,
    accel_z,
    mag_x,
    mag_y,
    mag_z,
    mag_accuracy
)]
pub struct Data {
    /// The heading range is 0.0˚ to +359.9˚
    pub heading: Option<f32>,

    /// The pitch range is -90.0˚ to +90.0
    pub pitch: Option<f32>,

    /// The roll range is to -180.0˚ to +180.0˚
    pub roll: Option<f32>,

    /// This value is provided in °C by the device’s internal temperature sensor. Its value is in degrees Celsius and has an accuracy of ±3° C.
    pub temperature: Option<f32>,

    /// This flag indicates at least one magnetometer axis reading is beyond ±150 µT.
    pub distortion: Option<bool>,

    /// This flag indicates the user calibration status. False means it is not user calibrated and this is the default value
    pub cal_status: Option<bool>,

    /// Accel Sensor Data, normalized to g (Earth's gravitational force)
    pub accel_x: Option<f32>,

    /// Accel Sensor Data, normalized to g (Earth's gravitational force)
    pub accel_y: Option<f32>,

    /// Accel Sensor Data, normalized to g (Earth's gravitational force)
    pub accel_z: Option<f32>,

    /// Mag sensor data in µT (micro-teslas)
    pub mag_x: Option<f32>,

    /// Mag sensor data in µT (micro-teslas)
    pub mag_y: Option<f32>,

    /// Mag sensor data in µT (micro-teslas)
    pub mag_z: Option<f32>,

    /// This value represents (in degrees) the approximate current magnetic accuracy of the system.  This should correspond to the RMS heading accuracy expected in a given location at a given time. When no user cal has been performed, the accuracy of this measurement is significantly reduced. This value combines the estimated accuracy of the most recent magnetic user calibration (cal score), change in the magnetic field since the last user cal, and any observed short-term transients observed in the background. This measurement is more accurate if the system is held somewhat still (as opposed to waving the unit around quickly), and may take some time to learn the ambient field (5-10s). Allowing the unit to see different orientations and pitch/rolls in an area will give a better background measurement of relative accuracy. Values are in degrees of heading. Because this measurement is based on post-fit residual measurements, it is not always a perfect indicator of true accuracy.  This score should be a good indicator of relative accuracy, i.e., if one location has a high score, and a second location has a lower score, the second location is more likely to have a clean field.  
    pub mag_accuracy: Option<f32>,
}

pub enum DataID {
    /// The heading range is 0.0˚ to +359.9˚
    Heading = 5,

    /// The pitch range is -90.0˚ to +90.0
    Pitch = 24,

    /// The roll range is to -180.0˚ to +180.0˚
    Roll = 25,

    /// This value is provided in °C by the device’s internal temperature sensor. Its value is in degrees Celsius and has an accuracy of ±3° C.
    Temperature = 7,

    /// This flag indicates at least one magnetometer axis reading is beyond ±150 µT.
    Distortion = 8,

    /// This flag indicates the user calibration status. False means it is not user calibrated and this is the default value
    CalStatus = 9,

    /// Accel Sensor Data, normalized to g (Earth's gravitational force)
    AccelX = 21,

    /// Accel Sensor Data, normalized to g (Earth's gravitational force)
    AccelY = 22,

    /// Accel Sensor Data, normalized to g (Earth's gravitational force)
    AccelZ = 23,

    /// Mag sensor data in µT (micro-teslas)
    MagX = 27,

    /// Mag sensor data in µT (micro-teslas)
    MagY = 28,

    /// Mag sensor data in µT (micro-teslas)
    MagZ = 29,

    /// This value represents (in degrees) the approximate current magnetic accuracy of the system.  This should correspond to the RMS heading accuracy expected in a given location at a given time. When no user cal has been performed, the accuracy of this measurement is significantly reduced. This value combines the estimated accuracy of the most recent magnetic user calibration (cal score), change in the magnetic field since the last user cal, and any observed short-term transients observed in the background. This measurement is more accurate if the system is held somewhat still (as opposed to waving the unit around quickly), and may take some time to learn the ambient field (5-10s). Allowing the unit to see different orientations and pitch/rolls in an area will give a better background measurement of relative accuracy. Values are in degrees of heading. Because this measurement is based on post-fit residual measurements, it is not always a perfect indicator of true accuracy.  This score should be a good indicator of relative accuracy, i.e., if one location has a high score, and a second location has a lower score, the second location is more likely to have a clean field.  
    MagAccuracy = 88,
}

impl TryFrom<u8> for DataID {
    type Error = ReadError;
    fn try_from(value: u8) -> Result<Self, ReadError> {
        use DataID::*;
        match value {
            5 => Ok(Heading),
            24 => Ok(Pitch),
            25 => Ok(Roll),
            7 => Ok(Temperature),
            8 => Ok(Distortion),
            9 => Ok(CalStatus),
            21 => Ok(AccelX),
            22 => Ok(AccelY),
            23 => Ok(AccelZ),
            27 => Ok(MagX),
            28 => Ok(MagY),
            29 => Ok(MagZ),
            88 => Ok(MagAccuracy),
            79 => Err(ReadError::ParseError("Unknown DataID from device: 79. This ID is usually detected when set_data_components is not called before calling get_data. You must specify what data you want from the device before parsing data back from the device.".to_string())),
            _ => Err(ReadError::ParseError(format!("Unknown DataID from device: {}", value)))
        }
    }
}

pub struct AcqParamsReserved {
    /// This flag sets whether output will be presented in Continuous or Polled Acquisition Mode. Poll Mode is TRUE and should be selected when the host system will poll the TargetPoint3 for each data set. Continuous Mode is FALSE and should be selected if the user will have the TargetPoint3 output data to the host system at a relatively fixed rate. Poll Mode is the default.
    pub acquisition_mode: bool,

    /// This is only relevant in Compass Mode. Setting this flag to TRUE results in the FIR filter being flushed (cleared) after every measurement. The default is FALSE.  Flushing the filter clears all tap values, thus purging old data. This can be useful if a significant change in heading has occurred since the last reading, as the old heading data would be in the filter. Once the taps are cleared, it is necessary to fully repopulate the filter before data is output. For example, if 32 FIR taps is set, 32 new samples must be taken before a reading will be output. The length of the delay before outputting data is directly correlated to the number of FIR taps.
    pub flush_filter: bool,

    /// Reserved for PNI Use
    pub reserved: f32,

    /// The SampleDelay is relevant when the Continuous Acquisition Mode is selected.  It is the time delay, in seconds, between completion of TargetPoint3 sending one set of data and the start of sending the next data set. The default is 0 seconds, which means TargetPoint3 will send new data as soon as the previous data set has been sent. Note that the inverse of the SampleDelay is somewhat greater than the actual sample rate, since the SampleDelay does not include actual acquisition time.
    pub sample_delay: f32,
}

impl From<AcqParamsReserved> for AcqParams {
    fn from(value: AcqParamsReserved) -> Self {
        AcqParams {
            acquisition_mode: value.acquisition_mode,
            flush_filter: value.flush_filter,
            sample_delay: value.sample_delay,
        }
    }
}

pub struct AcqParams {
    /// This flag sets whether output will be presented in Continuous or Polled Acquisition Mode. Poll Mode is TRUE and should be selected when the host system will poll the TargetPoint3 for each data set. Continuous Mode is FALSE and should be selected if the user will have the TargetPoint3 output data to the host system at a relatively fixed rate. Poll Mode is the default.
    pub acquisition_mode: bool,

    /// This is only relevant in Compass Mode. Setting this flag to TRUE results in the FIR filter being flushed (cleared) after every measurement. The default is FALSE.  Flushing the filter clears all tap values, thus purging old data. This can be useful if a significant change in heading has occurred since the last reading, as the old heading data would be in the filter. Once the taps are cleared, it is necessary to fully repopulate the filter before data is output. For example, if 32 FIR taps is set, 32 new samples must be taken before a reading will be output. The length of the delay before outputting data is directly correlated to the number of FIR taps.
    pub flush_filter: bool,

    /// The SampleDelay is relevant when the Continuous Acquisition Mode is selected.  It is the time delay, in seconds, between completion of TargetPoint3 sending one set of data and the start of sending the next data set. The default is 0 seconds, which means TargetPoint3 will send new data as soon as the previous data set has been sent. Note that the inverse of the SampleDelay is somewhat greater than the actual sample rate, since the SampleDelay does not include actual acquisition time.
    pub sample_delay: f32,
}

/// Represents the device mounting orientation
#[derive(Debug, Display)]
pub enum MountingRef {
    Std0 = 1,
    XUp0,
    YUp0,
    Std90,
    Std180,
    Std270,
    ZDown0,
    XUp90,
    XUp180,
    XUp270,
    YUp90,
    YUp180,
    YUp270,
    ZDown90,
    ZDown180,
    ZDown270,
}

/// Baud rates supported by tp3
#[derive(Debug, Display)]
pub enum Baud {
    B2400 = 4,
    B3600,
    B4800,
    B7200,
    B9600,
    B14400,
    B19200,
    B28800,
    B38400,
    B57600,
    B115200,
}

/// Represents a configuration parameter ID only. See also: ConfigParam, which represents ID +
/// value
#[derive(Debug, Display, Clone)]
pub enum ConfigID {
    /// This sets the declination angle to determine True North heading.
    /// Positive declination is easterly declination and negative is westerly declination.  This is not applied unless TrueNorth is set to TRUE.
    /// Range: [-180, 180]. Sensor Default: 0
    Declination = 1,

    /// Flag to set compass heading output to true north heading by adding the declination angle to the magnetic north heading.
    /// Sesnsor Default: false
    TrueNorth = 2,

    /// Sets the Endianness of packets. TRUE is Big-Endian. FALSE is Little-Endian.
    /// Currently, this library is hard-coded for big endian. Do not change this value.
    /// Sensor Default: true
    BigEndian = 6,

    /// This sets the reference orientation for the TargetPoint3. Please refer to Figure 4-2 in the user manual for additional information.
    /// Sensor Default: [MountingRef::Std0]
    MountingRef = 10,

    /// The user must select the number of points to take during a calibration. Table 7-4 in user manual provides the “Minimum Recommended” number of sample points, as well as the full “Allowable Range”. The “Minimum Recommended” number of samples normally is sufficient to meet the TargetPoint3’s heading accuracy specification, while less than this may make it difficult to meet specification. See Section 5 in user manual for additional information.
    /// Range: [4, 18]. Sensor Default: 12
    UserCalNumPoints = 12,

    /// This flag is used during user calibration. If set to TRUE, the TargetPoint3 automatically takes calibration sample points once the minimum change and stability requirements are met. If set to FALSE, the device waits for TakeUserCalSample to take a sample with the condition that a magnetic field vector component delta is greater than 5 µT from the last sample point. If the user wants to have maximum control over when the calibration sample points are taken, this flag should be set to FALSE.
    /// Sensor Default: true
    UserCalAutoSampling = 13,

    /// Baud rate index value. A power-down, power-up cycle is required when changing the baud rate. Additionally, you will need to re-construct the tp3 object and provide a [SerialPort] with the chosen baud.
    /// Library & Sensor Default = 38400. Range: One of { 2400, 3600, 4800, 7200, 9600, 14400, 19200, 28800, 38400, 57600, 115200 }
    BaudRate = 14,

    /// Sets the output units as mils (TRUE) or degrees (FALSE).
    /// Sensor Default: false
    MilOut = 15,

    /// This flag sets whether or not heading, pitch, and roll data are output simultaneously while the TargetPoint3 is being calibrated. FALSE disables simultaneous output.
    /// Sensor Default: true
    HPRDuringCal = 16,

    /// This command provides the flexibility to store up to eight (8) sets of magnetometer calibration coefficients in the TargetPoint3. The default is set number 0. To store a set of coefficients, first establish the set number (number 0 to 7) using MagCoeffSet, then perform the magnetometer calibration. The coefficient values will be stored in the defined set number. This feature is useful if the compass will be placed in multiple locations that have different local magnetic field properties.
    /// Sensor Default: 0. Range: 0 - 7
    MagCoeffSet = 18,

    /// This command provides the flexibility to store up to eight (8) sets of accelerometer calibration coefficients in the TargetPoint3. The default is set number 0. To store a set of coefficients, first establish the set number (number 0 to 7) using AccelCoeffSet, then perform the accelerometer calibration. The coefficient values will be stored in the defined set number.
    /// Sensor Default: 0. Range: 0 - 7
    AccelCoeffSet = 19,
}

/// Represents a configuration parameter and setting. See also: [ConfigID] for the name of a
/// configuration parameter only
#[repr(u8)]
pub enum ConfigPair {
    /// This sets the declination angle to determine True North heading.
    /// Positive declination is easterly declination and negative is westerly declination.  This is not applied unless TrueNorth is set to TRUE.
    /// Range: [-180, 180]. Sensor Default: 0
    Declination(f32) = 1,

    /// Flag to set compass heading output to true north heading by adding the declination angle to the magnetic north heading.
    /// Sesnsor Default: false
    TrueNorth(bool) = 2,

    /// Sets the Endianness of packets. TRUE is Big-Endian. FALSE is Little-Endian.
    /// Currently, this library is hard-coded for big endian. Do not change this value.
    /// Sensor Default: true
    BigEndian(bool) = 6,

    /// This sets the reference orientation for the TargetPoint3. Please refer to Figure 4-2 in the user manual for additional information.
    /// Sensor Default: [MountingRef::Std0]
    MountingRef(MountingRef) = 10,

    /// The user must select the number of points to take during a calibration. Table 7-4 in user manual provides the “Minimum Recommended” number of sample points, as well as the full “Allowable Range”. The “Minimum Recommended” number of samples normally is sufficient to meet the TargetPoint3’s heading accuracy specification, while less than this may make it difficult to meet specification. See Section 5 in user manual for additional information.
    /// Range: [4, 18]. Sensor Default: 12
    UserCalNumPoints(u32) = 12,

    /// This flag is used during user calibration. If set to TRUE, the TargetPoint3 automatically takes calibration sample points once the minimum change and stability requirements are met. If set to FALSE, the device waits for TakeUserCalSample to take a sample with the condition that a magnetic field vector component delta is greater than 5 µT from the last sample point. If the user wants to have maximum control over when the calibration sample points are taken, this flag should be set to FALSE.
    /// Sensor Default: true
    UserCalAutoSampling(bool) = 13,

    /// Baud rate index value. A power-down, power-up cycle is required when changing the baud rate. Additionally, you will need to re-construct the tp3 object and provide a [SerialPort] with the chosen baud.
    /// Library & Sensor Default = 38400. Range: One of { 2400, 3600, 4800, 7200, 9600, 14400, 19200, 28800, 38400, 57600, 115200 }
    BaudRate(Baud) = 14,

    /// Sets the output units as mils (TRUE) or degrees (FALSE).
    /// Sensor Default: false
    MilOut(bool) = 15,

    /// This flag sets whether or not heading, pitch, and roll data are output simultaneously while the TargetPoint3 is being calibrated. FALSE disables simultaneous output.
    /// Sensor Default: true
    HPRDuringCal(bool) = 16,

    /// This command provides the flexibility to store up to eight (8) sets of magnetometer calibration coefficients in the TargetPoint3. The default is set number 0. To store a set of coefficients, first establish the set number (number 0 to 7) using MagCoeffSet, then perform the magnetometer calibration. The coefficient values will be stored in the defined set number. This feature is useful if the compass will be placed in multiple locations that have different local magnetic field properties.
    /// Sensor Default: 0. Range: 0 - 7
    MagCoeffSet(u32) = 18,

    /// This command provides the flexibility to store up to eight (8) sets of accelerometer calibration coefficients in the TargetPoint3. The default is set number 0. To store a set of coefficients, first establish the set number (number 0 to 7) using AccelCoeffSet, then perform the accelerometer calibration. The coefficient values will be stored in the defined set number.
    /// Sensor Default: 0. Range: 0 - 7
    AccelCoeffSet(u32) = 19,
}

impl ConfigPair {
    // [unsafe]: This code pulls the integer representation of the enum, since the enum is repr(u8)
    // and the u8 is the first element in the enum, the pointer cast will work. Additionally, this
    // pattern has been directly copied from the rust documentation for error codes, with modification
    // only to its parameters and return values
    // src: https://github.com/rust-lang/rust/blob/master/compiler/rustc_error_codes/src/error_codes/E0732.md
    fn discriminant(&self) -> u8 {
        unsafe { *(self as *const Self as *const u8) }
    }
}

impl From<ConfigPair> for Vec<u8> {
    fn from(param: ConfigPair) -> Self {
        use ConfigPair::*;
        let mut vec = Vec::<u8>::new();
        vec.push(param.discriminant());

        match param {
            Declination(val) => {
                vec.extend_from_slice(&val.to_be_bytes());
            }
            TrueNorth(val) => {
                // not using 'as' since don't trust transmutation on bool to meet doc spec
                // requiring exactly 0 as false and exactly 1 as true
                if val {
                    vec.push(1);
                } else {
                    vec.push(0);
                }
            }
            BigEndian(val) => {
                if val {
                    vec.push(1);
                } else {
                    vec.push(0);
                }
            }
            MountingRef(mr) => {
                vec.push(mr as u8);
            }
            UserCalNumPoints(val) => vec.extend_from_slice(&val.to_be_bytes()),
            UserCalAutoSampling(val) => {
                if val {
                    vec.push(1);
                } else {
                    vec.push(0);
                }
            }
            BaudRate(val) => vec.push(val as u8),
            MilOut(val) => {
                if val {
                    vec.push(1);
                } else {
                    vec.push(0);
                }
            }
            HPRDuringCal(val) => {
                if val {
                    vec.push(1);
                } else {
                    vec.push(0);
                }
            }
            MagCoeffSet(val) => vec.extend_from_slice(&val.to_be_bytes()),
            AccelCoeffSet(val) => vec.extend_from_slice(&val.to_be_bytes()),
        };

        vec
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
    device_type: String,

    /// Device Version
    revision: String,
}

impl Get<f64> for TargetPoint3 {
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

impl Get<f32> for TargetPoint3 {
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

impl Get<i32> for TargetPoint3 {
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

impl Get<i16> for TargetPoint3 {
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

impl Get<i8> for TargetPoint3 {
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

impl Get<u32> for TargetPoint3 {
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

impl Get<u16> for TargetPoint3 {
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

impl Get<u8> for TargetPoint3 {
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

impl Get<bool> for TargetPoint3 {
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

impl Get<MountingRef> for TargetPoint3 {
    fn get(&mut self) -> Result<MountingRef, ReadError> {
        use MountingRef::*;
        let mut rbuff = [0u8; 1];
        self.serialport.read_exact(&mut rbuff)?;
        self.read_bytes += 1;
        self.read_checksum.update(&rbuff);
        match rbuff[0] {
            1 => Ok(Std0),
            2 => Ok(XUp0),
            3 => Ok(YUp0),
            4 => Ok(Std90),
            5 => Ok(Std180),
            6 => Ok(Std270),
            7 => Ok(ZDown0),
            8 => Ok(XUp90),
            9 => Ok(XUp180),
            10 => Ok(XUp270),
            11 => Ok(YUp90),
            12 => Ok(YUp180),
            13 => Ok(YUp270),
            14 => Ok(ZDown90),
            15 => Ok(ZDown180),
            16 => Ok(ZDown270),
            _ => Err(ReadError::ParseError(
                "MountingRef must be within [1, 16]".to_string(),
            )),
        }
    }

    fn get_string(&mut self) -> Result<String, ReadError> {
        Ok(Get::<MountingRef>::get(self)?.to_string())
    }
}

impl Get<Baud> for TargetPoint3 {
    fn get(&mut self) -> Result<Baud, ReadError> {
        use Baud::*;
        let mut rbuff = [0u8; 1];
        self.serialport.read_exact(&mut rbuff)?;
        self.read_bytes += 1;
        self.read_checksum.update(&rbuff);
        match rbuff[0] {
            4 => Ok(B2400),
            5 => Ok(B3600),
            6 => Ok(B4800),
            7 => Ok(B7200),
            8 => Ok(B9600),
            9 => Ok(B14400),
            10 => Ok(B19200),
            11 => Ok(B28800),
            12 => Ok(B38400),
            13 => Ok(B57600),
            14 => Ok(B115200),
            _ => Err(ReadError::ParseError(
                "Baud descriptor from device must be one of [4,14], the only supported bauds"
                    .to_string(),
            )),
        }
    }

    fn get_string(&mut self) -> Result<String, ReadError> {
        Ok(Get::<Baud>::get(self)?.to_string())
    }
}

impl Get<Data> for TargetPoint3 {
    fn get(&mut self) -> Result<Data, ReadError> {
        let mut data_struct = Data {
            heading: None,
            pitch: None,
            roll: None,
            temperature: None,
            distortion: None,
            cal_status: None,
            accel_x: None,
            accel_y: None,
            accel_z: None,
            mag_x: None,
            mag_y: None,
            mag_z: None,
            mag_accuracy: None,
        };

        let id_count = Get::<u8>::get(self)?;

        for _ in 0..id_count {
            let data_id = Get::<u8>::get(self)?;

            match DataID::try_from(data_id)? {
                DataID::Heading => {
                    data_struct.heading = Some(Get::<f32>::get(self)?);
                }
                DataID::Pitch => {
                    data_struct.pitch = Some(Get::<f32>::get(self)?);
                }
                DataID::Roll => {
                    data_struct.roll = Some(Get::<f32>::get(self)?);
                }
                DataID::Temperature => {
                    data_struct.temperature = Some(Get::<f32>::get(self)?);
                }
                DataID::Distortion => {
                    data_struct.distortion = Some(Get::<bool>::get(self)?);
                }
                DataID::CalStatus => {
                    data_struct.cal_status = Some(Get::<bool>::get(self)?);
                }
                DataID::AccelX => {
                    data_struct.accel_x = Some(Get::<f32>::get(self)?);
                }
                DataID::AccelY => {
                    data_struct.accel_y = Some(Get::<f32>::get(self)?);
                }
                DataID::AccelZ => {
                    data_struct.accel_z = Some(Get::<f32>::get(self)?);
                }
                DataID::MagX => {
                    data_struct.mag_x = Some(Get::<f32>::get(self)?);
                }
                DataID::MagY => {
                    data_struct.mag_y = Some(Get::<f32>::get(self)?);
                }
                DataID::MagZ => {
                    data_struct.mag_z = Some(Get::<f32>::get(self)?);
                }
                DataID::MagAccuracy => {
                    data_struct.mag_accuracy = Some(Get::<f32>::get(self)?);
                }
            };
        }

        Ok(data_struct)
    }

    fn get_string(&mut self) -> Result<String, ReadError> {
        Ok(Get::<Data>::get(self)?.to_string())
    }
}

// NOTE: when testing or writing doctests, be sure to put everything in its own scope so that the
// serialport is dropped afte each test
#[cfg(test)]
mod tests {
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
