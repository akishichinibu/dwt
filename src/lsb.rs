use crate::{
    image::Array2Ext,
    watermark::{self, Watermark},
};
use bitvec::{slice::BitSlice, vec::BitVec};
use imageproc::image::{GrayImage, Luma};

use ndarray::{Array1, Array2, ArrayView3, ArrayViewMut3, Axis};
use qrcode::{Color, QrCode};
use rayon::iter::{ParallelBridge, ParallelIterator};

pub const QR_VERSION: usize = 40;
pub const QR_MODULE_SIZE: usize = QR_VERSION * 4 + 17;

pub fn to_qrcode<'a>(data: &'a [u8]) -> Array2<bool> {
    let wm = QrCode::with_version(
        data,
        qrcode::Version::Normal(QR_VERSION as i16),
        qrcode::EcLevel::H,
    )
    .unwrap();

    Array1::from_iter(wm.into_colors().iter().map(|r| *r == Color::Dark))
        .into_shape((QR_MODULE_SIZE, QR_MODULE_SIZE))
        .unwrap()
}

pub fn lsb_encode<'a, S: Into<&'a [u8]>>(image: &mut ArrayViewMut3<u8>, data: S) {
    let (width, height, _) = image.dim();
    let data: &'a [u8] = data.into();

    let wm_data = to_qrcode(data)
        .into_shape((QR_MODULE_SIZE * QR_MODULE_SIZE,))
        .unwrap();

    let mut wm_index = Array2::from_shape_fn((width, height), |(x, y)| {
        (x % QR_MODULE_SIZE) * QR_MODULE_SIZE + y % QR_MODULE_SIZE
    });
    wm_index.par_mapv_inplace(|r| if wm_data[r] { 1 } else { 0 });

    *image &= 0b1111_1110;
    *image |= &wm_index.mapv(|r| r as u8).insert_axis(Axis(2));
}

// fn window_decode(chunk: &BitSlice<u8>) -> Vec<Vec<u8>> {
//     let decoder = reed_solomon::Decoder::new(ECC_LENGTH);
//     chunk
//         .windows(BLOCK_SIZE * 8)
//         .par_bridge()
//         .map(|window| {
//             let bitvec = window.to_bitvec();
//             let slice = &bitvec.as_raw_slice()[0..BLOCK_SIZE];
//             match decoder.correct(slice, None) {
//                 Ok(ecc) => Some(ecc.data().to_vec()),
//                 Err(e) => None,
//             }
//         })
//         .filter_map(|r| r)
//         .collect()
// }

pub fn lsb_decode(image: &ArrayView3<u8>) {
    let (width, height, _) = image.dim();
    let bit_buffer: BitVec<u8> = image
        .map_axis(Axis(2), |p| {
            let r = p[[0]];
            let g = p[[1]];
            let b = p[[2]];
            r & 0b1 == 1
        })
        .iter()
        .collect();

    let n = bit_buffer.len();
    println!("bit buffer length: {}", n);

    let pixels = Array1::from_iter(bit_buffer.iter())
        .mapv(|r| if *r { 255u8 } else { 0u8 })
        .into_shape((width, height))
        .unwrap();

    let code = pixels.to_image2();

    code.save("/Users/akishichinibu/Documents/dwt/testy.png")
        .unwrap();

    let mut code = rqrr::PreparedImage::prepare(code);
    let grids = code.detect_grids();

    for r in grids {
        let (meta, content) = r.decode().unwrap();
        println!("{:?} {:?}", meta, content);
    }
}
