use crate::command::Command;
use crate::responses::Get;
use crate::{RWError, ReadError, Device};

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

impl Get<Baud> for Device {
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

impl Get<MountingRef> for Device {
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

impl Device {
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
}
