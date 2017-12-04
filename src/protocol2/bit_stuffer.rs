#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub(crate) enum BitStufferError {
    ExpectedFirstHeaderByte,
    ExpectedSecondHeaderByte,
    ExpectedThirdHeaderByte,
    ExpectedReservedByte,
    ExpectedStuffByte,
}

impl From<BitStufferError> for ::protocol2::FormatError {
    fn from(e: BitStufferError) -> Self {
        match e {
            BitStufferError::ExpectedFirstHeaderByte => ::protocol2::FormatError::Header,
            BitStufferError::ExpectedSecondHeaderByte => ::protocol2::FormatError::Header,
            BitStufferError::ExpectedThirdHeaderByte => ::protocol2::FormatError::Header,
            BitStufferError::ExpectedReservedByte => ::protocol2::FormatError::Header,
            BitStufferError::ExpectedStuffByte => ::protocol2::FormatError::StuffByte,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub(crate) enum BitStufferState {
    /// Next byte must be first header byte (0xff).
    H0,
    /// Next byte must be second header byte (0xff).
    H1,
    /// Next byte must be third header byte (0xfd).
    H2,
    /// Next byte must be reserved byte (0x00).
    R,
    /// No bytes in header sequence received.
    B0,
    /// One byte in header sequence received (0xff).
    B1,
    /// Two bytes in header sequence received (0xff - 0xff).
    B2,
    /// Three bytes in header sequence received (0xff - 0xff - 0xfd). Next byte needs to be a "stuffed byte".
    B3,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub(crate) struct BitStuffer {
    state: BitStufferState,
}

impl BitStuffer {
    pub fn new() -> Self {
        BitStuffer{
            state: BitStufferState::H0,
        }
    }
    
    pub fn stuff_next(&self) -> bool {
        if self.state == BitStufferState::B3 {
            true
        } else {
            false
        }
    }
    
    pub fn add_byte(self, byte: u8) -> Result<Self, BitStufferError> {
        match self.state {
            BitStufferState::H0 => {
                if byte == 0xff {
                    Ok(BitStuffer{state: BitStufferState::H1})
                } else {
                    Err(BitStufferError::ExpectedFirstHeaderByte)
                }
            },
            BitStufferState::H1 => {
                if byte == 0xff {
                    Ok(BitStuffer{state: BitStufferState::H2})
                } else {
                    Err(BitStufferError::ExpectedSecondHeaderByte)
                }
            },
            BitStufferState::H2 => {
                if byte == 0xfd {
                    Ok(BitStuffer{state: BitStufferState::R})
                } else {
                    Err(BitStufferError::ExpectedThirdHeaderByte)
                }
            },
            BitStufferState::R => {
                if byte == 0x00 {
                    Ok(BitStuffer{state: BitStufferState::B0})
                } else {
                    Err(BitStufferError::ExpectedReservedByte)
                }
            },
            BitStufferState::B0 => {
                if byte == 0xff {
                    Ok(BitStuffer{state: BitStufferState::B1})
                } else {
                    Ok(BitStuffer{state: BitStufferState::B0})
                }
            },
            BitStufferState::B1 => {
                if byte == 0xff {
                    Ok(BitStuffer{state: BitStufferState::B2})
                } else {
                    Ok(BitStuffer{state: BitStufferState::B0})
                }
            },
            BitStufferState::B2 => {
                if byte == 0xfd {
                    Ok(BitStuffer{state: BitStufferState::B3})
                } else {
                    Ok(BitStuffer{state: BitStufferState::B0})
                }
            },
            BitStufferState::B3 => {
                if byte == 0xfd {
                    Ok(BitStuffer{state: BitStufferState::B0})
                } else {
                    Err(BitStufferError::ExpectedStuffByte)
                }
            },
        }
    }
}
