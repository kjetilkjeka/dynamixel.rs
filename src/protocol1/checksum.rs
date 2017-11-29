pub(crate) struct Checksum(u8);

impl Checksum {
    pub fn calc(data: &[u8]) -> Self {
        let mut sum: u8 = 0;
        
        for b in data {
            sum = sum.wrapping_add(*b);
        }

        Checksum(!sum)
    }
}

impl From<Checksum> for u8 {
    fn from(s: Checksum) -> u8 {
        s.0
    }
}
