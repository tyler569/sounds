#![allow(unused)]
#![allow(dead_code)]

use clap::Parser;
use cpal::{traits::HostTrait, Device};
use crossbeam::channel;
use num_complex::{Complex, ComplexFloat};
use output::{
    differential_encode::DifferentialEncoder,
    encode::Encoder,
    soundgen::{FrequencyComponent, SoundCommand, SoundGenerator},
};
use rustfft::FftPlanner;
use rustyline::error::ReadlineError;
use std::{f32::consts::PI, io::Write, process::exit, thread::sleep, time::Duration};

use crate::fft::FftPoint;

mod config;
mod fft;
// mod io2;
mod listen;
mod ringbuf;
mod output;
mod traits;
mod ui;

mod inverse_fft;

#[cfg(test)]
mod test;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value_t = 30.0)]
    target_fbucket: f32,

    #[arg(short, long)]
    input: bool,

    #[arg(short, long)]
    output: bool,

    #[arg(short, long)]
    ui: bool,
}

fn main() {

    use cpal::traits::*;
    use std::io::Read;

    let host = cpal::default_host();
    let device = host.default_input_device().unwrap();
    let config = device
        .supported_input_configs()
        .unwrap()
        .max_by_key(|c| c.max_sample_rate())
        .unwrap()
        .with_max_sample_rate()
        .config();

    let mut buffer = [Complex::new(0.0, 0.0); 4096];
    let mut n = 0;

    let stream = device.build_input_stream(
        &config,
        move |data, _info| {
            n += 1;
            if n % 50 != 0 {
                return
            }

            data.iter().enumerate().for_each(|(i, &v)| buffer[i].re = v);

            let mut planner = FftPlanner::new();
            let fft = planner.plan_fft_forward(4096);
            fft.process(&mut buffer);

            print!("[");
            buffer[..2048].iter().enumerate().for_each(|(i, &v)| {
                let point = FftPoint::new(i, v);
                print!("{}", point);
            });
            println!("]");
        },
        |err| {
            eprintln!("audio stream error: {:?}", err);
        }).unwrap();

    stream.play();

    std::thread::sleep(std::time::Duration::from_secs(100));
    std::process::exit(0);

    let args = Args::parse();

    let mut istream = None;
    let mut ostream = None;
    let mut commands = None;
    let mut fbucket = None;

    if args.input {
        let i = listen::listen(args.target_fbucket);
        (istream, fbucket) = (Some(i.0), Some(i.1));
    }

    if args.output {
        let o = output::output();
        (ostream, commands) = (Some(o.0), Some(o.1));
    }

    if args.ui {
        if let Some(ref snd) = commands {
            ui::ui(snd.clone());
        } else {
            eprintln!("Cannot do input UI without sound output");
            exit(1);
        }
    }

    if let Some(ref commands) = commands {
        let mut encoder = DifferentialEncoder::new(31.25, 4, commands.clone());

        write!(encoder, "Hello World");
    }

    sleep(Duration::from_secs(1000));
}





fn info() {
    use cpal::traits::*;

    let hosts = cpal::available_hosts();
    for id in hosts {
        println!("host: {:?}", id);
    }

    let host = cpal::default_host();

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