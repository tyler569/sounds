#![allow(unused)]
#![allow(dead_code)]

use std::f32::consts::PI;

use rustyline::error::ReadlineError;
use sounds::{FrequencyComponent, SoundCommand};
use crossbeam::channel;

mod listen;
mod output;
mod sounds;

fn ui(snd: channel::Sender<SoundCommand>) {
    loop {
        let mut rl = rustyline::Editor::<()>::new().unwrap();
        let readline = match rl.readline(">> ") {
            Ok(line) => line,
            Err(ReadlineError::Interrupted) => break,
            Err(_) => continue,
        };

        let words: Vec<_> = readline.trim().split_ascii_whitespace().collect();
        if words.len() == 0 {
            continue;
        }

        match words[0] {
            "clear" => snd.send(SoundCommand::ClearWaveform).unwrap(),
            "add" => {
                let component = FrequencyComponent::new_simple(words[1].parse().unwrap());
                snd.send(SoundCommand::AddWaveform(component)).unwrap()
            }
            "remove" => snd.send(SoundCommand::RemoveWaveform(words[1].parse().unwrap())).unwrap(),
            "volume" => snd.send(SoundCommand::TransitionVolume(words[1].parse().unwrap())).unwrap(),
            "Volume" => snd.send(SoundCommand::SetVolume(words[1].parse().unwrap())).unwrap(),
            "exit" => return,
            _ => eprintln!("Unsupported command!"),
        }
    }
}

fn main() {
    let (ostream, commands) = output::output();
    let istream = listen::listen();

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

    // ui(commands);

    std::thread::sleep(std::time::Duration::from_secs(100000));
}