use crate::{
    image::Array3Ext,
    lsb::{to_qrcode, QR_MODULE_SIZE},
};
use ndarray::{
    stack, Array2, Array3, ArrayBase, ArrayView2, ArrayView3, ArrayViewMut3, Axis, DataMut, Zip,
};
use ndrustfft::{ndfft_par, ndifft_par, Complex, FftHandler};

trait ArrayBaseExt<A, S>
where
    A: DataMut<Elem = f64>,
    S: ndarray::Dimension,
{
    fn max(&self) -> f64;
    fn normilize(&mut self, factor: f64);
}

impl<A, S> ArrayBaseExt<A, S> for ArrayBase<A, S>
where
    A: DataMut<Elem = f64>,
    S: ndarray::Dimension,
{
    fn max(&self) -> f64 {
        self.fold(f64::NEG_INFINITY, |max, r| {
            if r.partial_cmp(&max).map(|r| r.is_ge()).unwrap_or(false) {
                *r
            } else {
                max
            }
        })
    }

    fn normilize(&mut self, factor: f64) {
        let m = self.max();
        self.par_mapv_inplace(|r| r / m * factor);
    }
}

fn single_channel_fft2d(channel: &Array2<f64>) -> Array2<Complex<f64>> {
    let input = channel.mapv(|r| Complex::new(r, 0.0));
    let mut vhat = Array2::<Complex<f64>>::zeros(channel.raw_dim());

    let (width, height) = channel.dim();

    let mut handler_ax0 = FftHandler::<f64>::new(width);
    let mut handler_ax1 = FftHandler::<f64>::new(height);

    {
        let mut work: Array2<Complex<f64>> = Array2::zeros(channel.raw_dim());
        ndfft_par(&input, &mut work, &mut handler_ax1, 1);
        ndfft_par(&work, &mut vhat, &mut handler_ax0, 0);
    }

    vhat
}

fn single_channel_ifft2d(input: &ArrayView2<Complex<f64>>) -> Array2<f64> {
    let mut output: Array2<Complex<f64>> = Array2::zeros(input.raw_dim());

    let (width, height) = input.dim();
    let mut handler_ax0 = FftHandler::<f64>::new(width);
    let mut handler_ax1 = FftHandler::<f64>::new(height);

    {
        let mut work: Array2<Complex<f64>> = Array2::zeros(input.raw_dim());
        ndifft_par(&input, &mut work, &mut handler_ax0, 0);
        ndifft_par(&work, &mut output, &mut handler_ax1, 1);
    }
    output.mapv(|r| r.re)
}

trait ImageFft {
    fn fft2d(&self) -> Array3<Complex<f64>>;
}

impl<'a> ImageFft for ArrayView3<'a, u8> {
    fn fft2d(&self) -> Array3<Complex<f64>> {
        let layers = self.mapv(|r| (r as f64) / 255f64);

        let layers: Vec<Array2<Complex<f64>>> = layers
            .axis_iter(Axis(2))
            .map(|channel| single_channel_fft2d(&channel.to_owned()))
            .collect();

        stack(
            Axis(2),
            &[
                (&layers[0]).into(),
                (&layers[1]).into(),
                (&layers[2]).into(),
            ],
        )
        .unwrap()
    }
}

fn ifft2d(layers: &Array3<Complex<f64>>) -> Array3<f64> {
    let layers: Vec<Array2<f64>> = layers
        .axis_iter(Axis(2))
        .map(|channel| single_channel_ifft2d(&channel))
        .collect();
    stack(
        Axis(2),
        &[
            (&layers[0]).into(),
            (&layers[1]).into(),
            (&layers[2]).into(),
        ],
    )
    .unwrap()
}

pub fn fft_encode<'a, S: Into<&'a [u8]>>(mut image: &mut ArrayViewMut3<u8>, data: S) {
    let mut fft_layers = image.view().fft2d();
    let mut norm = fft_layers.mapv(|r| r.norm());

    norm.par_mapv_inplace(|r| r.log10());

    let max_norm = norm.max();
    norm.normilize(255.0);

    // draw mark
    let (width, height, _) = norm.dim();
    let wm_data = to_qrcode(data.into());
    let wm_data = wm_data.mapv(|r| if r { 32f64 } else { 0f64 });
    let wm_data = wm_data
        .into_shape((QR_MODULE_SIZE * QR_MODULE_SIZE,))
        .unwrap();
    let mut wm_index = Array2::from_shape_fn((width, height), |(x, y)| {
        (x % QR_MODULE_SIZE) * QR_MODULE_SIZE + y % QR_MODULE_SIZE
    });
    let wm_data = wm_index.mapv(|r| wm_data[r]);

    norm += &wm_data.insert_axis(Axis(2));
    norm.normilize(max_norm);
    norm.par_mapv_inplace(|r| 10f64.powf(r));

    Zip::from(&mut fft_layers)
        .and(&norm)
        .par_for_each(|r, norm| *r = Complex::from_polar(*norm, r.arg()));

    let mut new = ifft2d(&fft_layers);

    new.normilize(255.0);
    let new = new.mapv(|r| r as u8);

    new.to_image()
        .save("/Users/akishichinibu/Documents/dwt/test_out41.png")
        .unwrap();
}
