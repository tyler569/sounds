#![allow(unused)]
#![allow(dead_code)]

use bit_org::BitOrg;
use clap::Parser;
use cpal::{traits::HostTrait, Device};
use crossbeam::channel;
use io::input::input;
use num_complex::{Complex, ComplexFloat};
use rustfft::FftPlanner;
use rustyline::error::ReadlineError;
use io::SoundRead;
use std::{f32::consts::PI, io::{Write, Read}, process::exit, thread::sleep, time::Duration};

use crate::fft::FftPoint;

mod bit_org;
mod config;
mod fft;
mod io;
mod decode;
mod encode;

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

    let mut input = crate::io::input::input();
    let mut viz = crate::fft::FftVisualizer::new(&mut input, 0..150);

    loop {
        let mut buffer = [0f32; 4096];
        viz.read(&mut buffer);
    }




    std::process::exit(0);

    let args = Args::parse();
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