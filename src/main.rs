#![allow(unused)]
#![allow(dead_code)]

use std::thread;

use crate::io::{
    types::{Fft, FftPoint, Samples},
    output::OutputStream,
    input::InputStream,
};
use bit_org::BitOrg;

mod bit_org;
mod color;
mod io;

#[cfg(test)]
mod test;

const BUFFER_SIZE: usize = 128;

fn main() {
    info();
    rainbow();

    let istream = InputStream::new(48000, BUFFER_SIZE);
    let ichan = istream.channel();

    thread::spawn(|| {
        let ostream = OutputStream::new(48000, BUFFER_SIZE);
        let ochan = ostream.channel();

        for i in (1..=2).cycle() {
            let mut fft = Fft::new(BUFFER_SIZE);
            fft.set_point(i, FftPoint::new(1., 0.));
        }
    });

    let mut offset = 0;
    let mut message = None;

    for samples in ichan.into_iter() {
        println!("{}", samples);

        let fft = samples.into_fft();
        let point = fft.point(1);
        if point.amplitude() > 0.02 {
            let diff = ((1. - point.phase_01()) * BUFFER_SIZE as f32) as usize;
            if diff > 3 && diff < 124 {
                message = Some(diff);
                istream.add_offset(diff as usize);
            }
        }

        print!("{} {:?}", fft, point);
        if let Some(v) = message {
            println!(" (adjusting by {})", v);
        } else {
            println!();
        }

        message = None;
    }
}


fn rainbow() {
    let mut x = 0.0;
    while x < 2. * std::f32::consts::PI {
        let color: color::Color = x.into();
        print!("{}#", color);
        x += 0.1;
    }
    println!("{}", color::Reset);
}

fn info() {
    use cpal::traits::*;

    let hosts = cpal::available_hosts();
    for id in hosts {
        println!("host: {:?}", id);
    }

    let host = cpal::default_host();

    println!("--- output ---");
    let devices = host.output_devices().unwrap();

    for device in devices {
        println!("device: {}", device.name().unwrap());
        let configs = device.supported_output_configs().unwrap();
        for config in configs {
            println!("config: {:?}", config);
        }
    }

    println!("--- input ---");
    let devices = host.input_devices().unwrap();

    for device in devices {
        println!("device: {}", device.name().unwrap());
        let configs = device.supported_input_configs().unwrap();
        for config in configs {
            println!("config: {:?}", config);
        }
    }
}
