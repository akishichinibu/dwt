use rayon::iter::{ParallelBridge, ParallelIterator};

pub const MAGIC_NUMBER: &[u8] = b"\x0b\x2d\x0edwt\x01\x09";
pub const MAGIC_NUMBER_LENGTH: usize = MAGIC_NUMBER.len();

#[derive(Debug)]
pub struct Watermark {
    pub message: Vec<u8>,
}

impl Watermark {
    pub fn new<'a, S: Into<&'a [u8]>>(message: S) -> Watermark {
        Watermark {
            message: message.into().to_vec(),
        }
    }
}

impl Into<Vec<u8>> for &Watermark {
    fn into(self) -> Vec<u8> {
        let encoder = reed_solomon::Encoder::new(32);
        self.message
            .chunks(223)
            .par_bridge()
            .flat_map(|b| {
                let ecc = encoder.encode(b);
                ecc.to_vec()
            })
            .collect()
    }
}
