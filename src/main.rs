#![allow(unused)]
#![allow(dead_code)]

use clap::Parser;
use crossbeam::channel;
use output::{
    differential_encode::DifferentialEncoder,
    encode::Encoder,
    soundgen::{FrequencyComponent, SoundCommand, SoundGenerator},
};
use rustyline::error::ReadlineError;
use std::{f32::consts::PI, io::Write, process::exit, thread::sleep, time::Duration};

mod config;
mod fft;
mod listen;
mod output;
mod traits;
mod ui;

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
