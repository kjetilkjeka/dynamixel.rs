use bit_field::BitField;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Error {
    Timeout,
    Format(FormatError),
    Processing(ProcessingError),
}

impl From<::Error> for Error {
    fn from(e: ::Error) -> Error {
        match e {
            ::Error::Timeout => Error::Timeout
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum FormatError {
    ID,
    Header,
    CRC,
    Length,
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub struct ProcessingError(u8);

impl From<ProcessingError> for u8 {
    fn from(e: ProcessingError) -> u8 {
        e.0
    }
}

impl ProcessingError {
    pub fn decode(v: u8) -> Result<Option<Self>, ()> {
        if v == 0 {
            Ok(None)
        } else if v.get_bit(7) {
            Err(())
        } else {
            Ok(Some(ProcessingError(v)))
        }
    }
    
    pub fn instruction_error(&self) -> bool {
        self.0.get_bit(6)
    }

    pub fn overload_error(&self) -> bool {
        self.0.get_bit(5)
    }

    pub fn checksum_error(&self) -> bool {
        self.0.get_bit(4)
    }

    pub fn range_error(&self) -> bool {
        self.0.get_bit(3)
    }

    pub fn overheating_error(&self) -> bool {
        self.0.get_bit(3)
    }

    pub fn angle_limit_error(&self) -> bool {
        self.0.get_bit(1)
    }

    pub fn input_voltage_error(&self) -> bool {
        self.0.get_bit(0)
    }
}

impl ::lib::fmt::Debug for ProcessingError {
    fn fmt(&self, f: &mut ::lib::fmt::Formatter) -> ::lib::fmt::Result {
        write!(f, "The current ProcessingError, {:?}, decodes to the following errors: [", self.0)?;
        if self.instruction_error() {write!(f, "instruction_error")?;}
        if self.overload_error() {write!(f, "overload_error")?;}
        if self.checksum_error() {write!(f, "checksum_error")?;}
        if self.range_error() {write!(f, "range_error")?;}
        if self.overheating_error() {write!(f, "overheating_error")?;}
        if self.angle_limit_error() {write!(f, "angle_limit_error")?;}
        if self.input_voltage_error() {write!(f, "input_voltage_error")?;}
        write!(f, "]")
    }
}
