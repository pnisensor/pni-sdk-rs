use crate::command::Command;
use crate::responses::Get;
use crate::{RWError, ReadError, TargetPoint3};

use std::error::Error;

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

impl TargetPoint3 {
    /// This frame sets the sensor acquisition parameters in the TargetPoint3.
    ///
    /// # Arguments
    /// * `acq_params` - Parameters to set for next acquisition
    pub fn set_acq_params(&mut self, acq_params: AcqParams) -> Result<(), RWError> {
        self.set_acq_params_impl(AcqParamsReserved {
            acquisition_mode: acq_params.acquisition_mode,
            flush_filter: acq_params.flush_filter,
            reserved: f32::from_be_bytes([0u8, 0u8, 0u8, 0u8]),
            sample_delay: acq_params.sample_delay,
        })
    }

    /// Like set_acq_parameters, but gives the user the ability to write to the PNI reserved
    /// fields. Note different parameter ordering (done to reflect order inside payload)
    /// Confused? Just use set_acq_parameters
    pub fn set_acq_params_impl(
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
    
    /// Like set_acq_parameters, but gives the user the ability to write to the PNI reserved
    /// fields. Note different parameter ordering (done to reflect order inside payload)
    /// Confused? Just use set_acq_parameters
    #[cfg(feature = "reserved")]
    pub fn set_acq_params_reserved(
        &mut self,
        acq_params: AcqParamsReserved,
    ) -> Result<(), RWError> {
        self.set_acq_params_impl(acq_params)
    }
    
    /// Same as get_acq_params, but instead returns a tuple whose first value are the AcqParams and
    /// whose second value are the reserved bits
    #[cfg(feature = "reserved")]
    pub fn get_acq_params_reserved(&mut self) -> Result<AcqParamsReserved, RWError> {
        self.get_acq_params_impl()
    }

    /// Same as get_acq_params, but instead returns a tuple whose first value are the AcqParams and
    /// whose second value are the reserved bits
    pub fn get_acq_params_impl(&mut self) -> Result<AcqParamsReserved, RWError> {
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
        Ok(self.get_acq_params_impl()?.into())
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
    pub fn start_continuous_mode(&mut self) -> Result<(), RWError> {
        self.write_frame(Command::StartContinuousMode, None)?;
        Ok(())
    }

    /// This frame commands the TargetPoint3 to stop data output when in Continuous Acquisition Mode. The frame has no payload.
    /// You must call [TargetPoint3::save] and power cycle the device after calling [TargetPoint3::stop_continuous_mode] to stop continuous output
    pub fn stop_continuous_mode(&mut self) -> Result<(), RWError> {
        self.write_frame(Command::StopContinuousMode, None)?;
        Ok(())
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
        self.start_continuous_mode()?;
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
        self.stop_continuous_mode()?;
        self.save()?;
        self.power_down()?;
        let mut newtp3 = TargetPoint3::connect(None)?;
        newtp3.power_up()?;
        Ok(newtp3)
    }

    pub fn iter<'a>(&'a mut self) -> impl Iterator<Item = Result<Data, ReadError>> + 'a {
        ContinuousModeIterator(self)
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
