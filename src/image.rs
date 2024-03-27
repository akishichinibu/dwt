use std::{fs::File, io::Read, ops::Deref, path::Path};

use imageproc::image::{load_from_memory, ImageBuffer, Luma, Pixel, Primitive, Rgb, RgbImage};
use ndarray::{Array2, Array3, Axis};

pub fn load_from_path(path: &Path) -> RgbImage {
    let mut data: Vec<u8> = Vec::new();

    let mut f = File::open(path).unwrap();
    f.read_to_end(&mut data).unwrap();
    load_from_memory(&data)
        .expect("Failed to load image")
        .to_rgb8()
}

pub trait RgbImageExt<P, Container>
where
    P: Pixel,
{
    fn to_array(&self) -> Array3<P::Subpixel>;
}

impl<P, Container> RgbImageExt<P, Container> for ImageBuffer<P, Container>
where
    P: Pixel,
    Container: Deref<Target = [P::Subpixel]>,
    P::Subpixel: Send,
{
    fn to_array(&self) -> Array3<P::Subpixel> {
        let (width, height) = self.dimensions();
        let width = width as usize;
        let height = height as usize;

        let mut data =
            Array3::from_shape_simple_fn((width, height, 3), || P::Subpixel::DEFAULT_MAX_VALUE);

        data.indexed_iter_mut().for_each(|((x, y, c), r)| {
            let pixel = self.get_pixel(x as u32, y as u32);
            *r = pixel.channels()[c];
        });

        data
    }
}

pub trait Array3Ext {
    fn to_image(&self) -> ImageBuffer<Rgb<u8>, Vec<u8>>;

    fn to_gray(&self) -> Array2<f64>;
}

impl Array3Ext for Array3<u8> {
    fn to_image(&self) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
        let (width, height, _) = self.dim();
        let mut image: ImageBuffer<Rgb<u8>, Vec<u8>> =
            ImageBuffer::new(width as u32, height as u32);

        let pixels = self.map_axis(Axis(2), |p| {
            let r = p[[0]];
            let g = p[[1]];
            let b = p[[2]];
            Rgb([r, g, b])
        });

        pixels.indexed_iter().for_each(|((x, y), p)| {
            image.put_pixel(x as u32, y as u32, *p);
        });

        image
    }

    fn to_gray(&self) -> Array2<f64> {
        self.map_axis(Axis(2), |p| {
            let r = p[[0]];
            let g = p[[1]];
            let b = p[[2]];
            r as f64 * 0.299 + g as f64 * 0.587 + b as f64 * 0.114
        })
    }
}

pub trait Array2Ext {
    fn to_image(&self) -> ImageBuffer<Rgb<u8>, Vec<u8>>;
    fn to_image2(&self) -> ImageBuffer<Luma<u8>, Vec<u8>>;
}

impl Array2Ext for Array2<u8> {
    fn to_image(&self) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
        let (width, height) = self.dim();
        let mut image: ImageBuffer<Rgb<u8>, Vec<u8>> =
            ImageBuffer::new(width as u32, height as u32);

        self.indexed_iter().for_each(|((x, y), p)| {
            image.put_pixel(x as u32, y as u32, Rgb([*p, *p, *p]));
        });

        image
    }

    fn to_image2(&self) -> ImageBuffer<Luma<u8>, Vec<u8>> {
        let (width, height) = self.dim();
        let mut image: ImageBuffer<Luma<u8>, Vec<u8>> =
            ImageBuffer::new(width as u32, height as u32);

        self.indexed_iter().for_each(|((x, y), p)| {
            image.put_pixel(x as u32, y as u32, Luma([*p]));
        });

        image
    }
}
