use std::{io::Error, path::PathBuf, str::FromStr};

use structopt::StructOpt;

use crate::{
    fft::fft_encode,
    image::{load_from_path, Array3Ext, RgbImageExt},
    lsb::{lsb_decode, lsb_encode},
};

#[derive(Debug, PartialEq)]
enum Algorithms {
    Lsb,
    Fft,
}

impl FromStr for Algorithms {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "lsb" => Ok(Self::Lsb),
            "fft" => Ok(Self::Fft),
            _ => Err(Error::new(std::io::ErrorKind::Other, "")),
        }
    }
}

#[derive(Debug, StructOpt)]
#[structopt(name = "encode")]
struct Encode {
    #[structopt(long, short = "i", parse(from_os_str))]
    input: PathBuf,

    #[structopt(long, short = "o")]
    output: PathBuf,

    #[structopt(long, short = "m")]
    message: String,

    #[structopt(long, short = "a")]
    algorithms: Vec<Algorithms>,
}

#[derive(Debug, StructOpt)]
#[structopt(name = "decode")]
struct Decode {
    #[structopt(long, short = "i", parse(from_os_str))]
    input: PathBuf,

    #[structopt(long, short = "a")]
    algorithm: Algorithms,

    #[structopt(long, short = "o")]
    output: PathBuf,
}

#[derive(Debug, StructOpt)]
#[structopt(name = "fft")]
enum Dwt {
    Encode(Encode),
    Decode(Decode),
}

pub fn run() {
    let opt = Dwt::from_args();
    println!("{:?}", opt);

    match opt {
        Dwt::Encode(encode) => {
            let image = load_from_path(encode.input.as_path());
            let mut image_array = image.to_array();
            println!(
                "read image from {:?}, shape: {:?}",
                encode.input,
                image_array.dim()
            );
            encode.algorithms.iter().for_each(|a| match a {
                Algorithms::Lsb => {
                    println!("apply lsb method");
                    lsb_encode(&mut image_array.view_mut(), encode.message.as_bytes());
                }
                Algorithms::Fft => {
                    println!("apply fft method");
                    fft_encode(&mut image_array.view_mut(), encode.message.as_bytes())
                }
            });
            image_array.to_image().save(encode.output).unwrap();
        }
        Dwt::Decode(decode) => {
            let image = load_from_path(decode.input.as_path());
            let image_array = image.to_array();
            match decode.algorithm {
                Algorithms::Lsb => lsb_decode(&image_array.view()),
                Algorithms::Fft => {}
            }
        } // match encode {
          //     Lsb::Encode { input } => {
          //         let mut image = load_from_path(input.as_path());
          //         lsb_encode(&image, "yangdi dashabiyangdi dashabiyangdi dashabiyangdi dashabiyangdi dashabiyangdi dashabiyangdi dashabiyangdi dashabiyangdi dashabiyangdi dashabiyangdi dashabiyangdi dashabiyangdi dashabiyangdi dashabiyangdi dashabiyangdi dashabiyangdi dashabiyangdi dashabiyangdi dashabiyangdi dashabiyangdi dashabiyangdi dashabiyangdi dashabiyangdi dashabiyangdi dashabiyangdi dashabiyangdi dashabiyangdi dashabiyangdi dashabiyangdi dashabiyangdi dashabiyangdi dashabiyangdi dashabiyangdi dashabiyangdi dashabiyangdi dashabi".as_bytes());
          //         image.write_to("")
          //     }
          //     Lsb::Decode { input } => {
          //         let mut image = Image::from_path(input.as_path());
          //         lsb_decode(&image);
          //     }
          // },
          // Dwt::Fft(fft) => match fft {
          //     Fft::Encode { input } => {
          //         let mut image = Image::from_path(input.as_path());
          //         fft_encode(&mut image, "hello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello world".as_bytes());
          //         image.write()
          //     }
          //     _ => {}
          // },
    }
}
