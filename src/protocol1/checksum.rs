use bit_field::BitField;

pub(crate) struct Checksum(u8);

impl Checksum {
    pub fn calc(data: &[u8]) -> Self {
        let mut sum: u8 = 0;
        
        for b in data {
            if 255 - b >= sum {
                sum += b;
            } else {
                sum = sum.get_bits(0..7) + b.get_bits(0..7);
            }
        }

        Checksum(!sum)
    }
}

impl From<Checksum> for u8 {
    fn from(s: Checksum) -> u8 {
        s.0
    }
}
