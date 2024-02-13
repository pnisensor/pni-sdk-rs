use crate::command::Command;
use crate::responses::Get;
use crate::{RWError, ReadError, TargetPoint3, WriteError};

impl TargetPoint3 {
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
    fn take_user_cal_sample_impl(&mut self) -> Result<UserCalResponseReserved, RWError> {
        self.write_frame(Command::TakeUserCalSample, None)?;

        let expected_size = Get::<u16>::get(self)?;
        let resp_command = Get::<u8>::get(self)?;

        if resp_command == Command::UserCalSampleCount.discriminant() {
            let sample_count = Get::<u32>::get(self)?;
            self.end_frame(expected_size)?;
            Ok(UserCalResponseReserved::SampleCount(sample_count))
        } else if resp_command == Command::UserCalScore.discriminant() {
            let ret = UserCalResponseReserved::UserCalScore {
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

    /// This frame commands the TargetPoint3 to take a sample during user calibration.
    ///
    /// Returns the sample count, unless this is the last sample point, in which case returns the calibration score.
    /// If the sample was succesful, calibration should return 1 more
    /// than the previous sample count (or return the score)
    #[cfg(feature = "reserved")]
    pub fn take_user_cal_sample_reserved(&mut self) -> Result<UserCalResponseReserved, RWError> {
        self.take_user_cal_sample_impl()
    }

    pub fn take_user_cal_sample(&mut self) -> Result<UserCalResponse, RWError> {
        Ok(self.take_user_cal_sample_impl()?.into())
    }

    /// This command aborts the calibration process. The prior calibration results are retained.
    pub fn stop_cal(&mut self) -> Result<(), WriteError> {
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

impl From<UserCalResponseReserved> for UserCalResponse {
    fn from(value: UserCalResponseReserved) -> Self {
        match value {
            UserCalResponseReserved::SampleCount(c) => UserCalResponse::SampleCount(c),
            UserCalResponseReserved::UserCalScore { mag_cal_score, reserved: _, accel_cal_score, distribution_error, tilt_error, tilt_range } => UserCalResponse::UserCalScore { mag_cal_score, accel_cal_score, distribution_error, tilt_error, tilt_range}
        }
    }
}

pub enum UserCalResponseReserved {
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
