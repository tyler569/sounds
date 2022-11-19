#![allow(unused)]
#![allow(dead_code)]

use crate::config::SoundRange::*;
use crate::fft::FftPoint;
use bit_org::BitOrg;
use clap::Parser;
use cpal::{traits::HostTrait, Device};
use crossbeam::channel;
use io::input::input;
use io::SoundRead;
use num_complex::{Complex, ComplexFloat};
use rustfft::FftPlanner;
use rustyline::error::ReadlineError;
use std::{
    f32::consts::PI,
    io::{Read, Write},
    process::exit,
    thread::sleep,
    time::Duration,
};

mod bit_org;
mod color;
mod config;
mod decode;
mod encode;
mod fft;
mod io;

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
    info();

    let mut x = 0.0;
    while x < 10.0 {
        let color: color::Color = x.into();
        print!("{}#{}", color, color::Reset);
        x += 0.1;
    }
    println!();

    std::thread::scope(|s| {
        s.spawn(|| io::input4::main());
        s.spawn(|| io::output4::main());
    });

    //let mut input = crate::io::input::input(96000);
    //let mut viz = crate::fft::FftVisualizer::new(&mut input, 32 * 1024, Frequencies(2000..2050));

    //loop {
    //    let mut buffer = [0f32; 4096];
    //    if viz.read(&mut buffer).unwrap() == 0 {
    //        break;
    //    }
    //}
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
