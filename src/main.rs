#![allow(unused)]
#![allow(dead_code)]

use clap::Parser;
use cpal::{traits::HostTrait, Device};
use crossbeam::channel;
use io2::input::input;
use num_complex::{Complex, ComplexFloat};
use output::{
    differential_encode::DifferentialEncoder,
    encode::Encoder,
    soundgen::{FrequencyComponent, SoundCommand, SoundGenerator},
};
use rustfft::FftPlanner;
use rustyline::error::ReadlineError;
use traits::SoundRead;
use std::{f32::consts::PI, io::Write, process::exit, thread::sleep, time::Duration};

use crate::fft::FftPoint;

mod config;
mod fft;
mod io2;
mod listen;
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



    let mut input = input();
    let mut buffer = [0f32; 96000];
    loop {
        let len = input.read(&mut buffer).unwrap();
        println!("{:?} {:?}", &buffer[..2], &buffer[95998..]);
    }








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