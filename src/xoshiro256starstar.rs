use rand_core::{
    Error, RngCore, impls::fill_bytes_via_next, le::read_u64_into,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Xoshiro256StarStar {
    s: [u64; 4],
}

#[allow(dead_code)]
impl Xoshiro256StarStar {
    pub fn to_state(&self) -> [u64; 4] { self.s }

    pub fn from_state(state: &[u64; 4]) -> Self { Self { s: *state } }

    pub fn to_data(&self) -> [u8; 32] {
        let state = self.s;
        let mut data = [0; 32];
        for i in 0..4 {
            data[i * 8..(i + 1) * 8].copy_from_slice(&state[i].to_le_bytes());
        }
        data
    }

    pub fn from_data(data: &[u8; 32]) -> Self {
        let mut s = [0; 4];
        read_u64_into(data, &mut s);
        Self { s }
    }

    pub fn next_byte(&mut self) -> u8 { self.next_u64() as u8 }

    pub fn next_bytes(&mut self, len: usize) -> Vec<u8> {
        (0..len).map(|_| self.next_byte()).collect()
    }
}

macro_rules! starstar_u64 {
    ($x:expr) => {
        $x.wrapping_mul(5).rotate_left(7).wrapping_mul(9)
    };
}

macro_rules! impl_xoshiro_u64 {
    ($self:expr) => {
        let t = $self.s[1] << 17;

        $self.s[2] ^= $self.s[0];
        $self.s[3] ^= $self.s[1];
        $self.s[1] ^= $self.s[2];
        $self.s[0] ^= $self.s[3];

        $self.s[2] ^= t;

        $self.s[3] = $self.s[3].rotate_left(45);
    };
}

impl RngCore for Xoshiro256StarStar {
    #[inline]
    fn next_u32(&mut self) -> u32 {
        // The lowest bits have some linear dependencies, so we use the
        // upper bits instead.
        (self.next_u64() >> 32) as u32
    }

    #[inline]
    fn next_u64(&mut self) -> u64 {
        let result_starstar = starstar_u64!(self.s[1]);
        impl_xoshiro_u64!(self);
        result_starstar
    }

    #[inline]
    fn fill_bytes(&mut self, dest: &mut [u8]) {
        fill_bytes_via_next(self, dest);
    }

    #[inline]
    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), Error> {
        self.fill_bytes(dest);
        Ok(())
    }
}
