#![allow(unused)]
#![allow(dead_code)]

use clap::Parser;
use crossbeam::channel;
use rustyline::error::ReadlineError;
use output::soundgen::{FrequencyComponent, SoundCommand};
use std::{f32::consts::PI, process::exit};

mod listen;
mod output;
mod ui;

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
        if let Some(snd) = commands {
            ui::ui(snd);
        } else {
            eprintln!("Cannot do input UI without sound output");
            exit(1);
        }
    } else {
        std::thread::sleep(std::time::Duration::from_secs(10000));
    }

    /*
    let data = "Hello World";
    let mut iter = data.bytes().cycle();

    fn send(carrier: f32, b: u8, c: &channel::Sender<SoundCommand>) {
        let (bh, bl) = (b >> 4, b & 0xF);
        let phase1 = bh as f32 / 16.0 * PI * 2.0;
        let phase2 = bl as f32 / 16.0 * PI * 2.0;
        let amplitude = 1.0;

        // eprintln!("{} {} {}", carrier, phase1, amplitude);

        c.send(SoundCommand::AddWaveform(FrequencyComponent::new(carrier, phase1, amplitude)));
        // c.send(SoundCommand::AddWaveform(FrequencyComponent::new(carrier + 31.25, phase2, amplitude)));
    }

    loop {
        send(14.0 * 31.25, iter.next().unwrap(), &commands);
        send(16.0 * 31.25, iter.next().unwrap(), &commands);
        send(18.0 * 31.25, iter.next().unwrap(), &commands);
        send(20.0 * 31.25, iter.next().unwrap(), &commands);

        commands.send(SoundCommand::TransitionVolume(0.1));
        std::thread::sleep(std::time::Duration::from_millis(200));
        commands.send(SoundCommand::TransitionVolume(0.0));
        std::thread::sleep(std::time::Duration::from_millis(50));
        commands.send(SoundCommand::ClearWaveform);
    }

    */
}
