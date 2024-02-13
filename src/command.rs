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
    pub(crate) fn discriminant(&self) -> u8 {
        unsafe { *(self as *const Self as *const u8) }
    }
}
