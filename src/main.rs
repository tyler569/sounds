#![allow(unused)]
#![allow(dead_code)]

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
    // let (ostream, commands) = output::output();
    let istream = listen::listen();

    // ui(commands);

    std::thread::sleep(std::time::Duration::from_millis(100000));
}