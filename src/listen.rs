use std::f32::consts::PI;

use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    Stream, InputStreamTimestamp, BufferSize,
};
use rustfft::FftPlanner;
use num_complex::{Complex, ComplexFloat};

fn to_complexes(f: &[f32], channels: usize) -> Vec<Complex<f32>> {
    f.iter().step_by(channels).map(|v| Complex::new(*v, 0.0)).collect()
}

struct FftPoint {
    amplitude: f32,
    complex: Complex<f32>,
    frequency: f32,
    phase: f32, // positive, * 1/PI
}

impl FftPoint {
    fn new(fbucket: f32, value: (usize, Complex<f32>)) -> Self {
        let mut phase = value.1.im.atan2(value.1.re) / PI;
        if phase < 0.0 {
            phase += 1.0
        }

        Self {
            complex: value.1,
            amplitude: value.1.abs(),
            frequency: value.0 as f32 * fbucket,
            phase,
        }
    }

    fn character(&self) -> char {
        if self.amplitude > 20.0 {
            '#'
        } else if self.amplitude > 10.0 {
            '*'
        } else if self.amplitude > 4.0 {
            ':'
        } else if self.amplitude > 1.0 {
            ' '
        } else {
            ' '
        }
    }

    fn color(&self) -> Color {
        let circle = self.phase * PI * 2.0;
        let r = ((circle + 0.0).sin() * 127.0 + 128.0) as u8;
        let g = ((circle + 2.0).sin() * 127.0 + 128.0) as u8;
        let b = ((circle + 4.0).sin() * 127.0 + 128.0) as u8;
        Color(r, g, b)
    }
}
 
impl std::fmt::Debug for FftPoint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if f.alternate() {
            write!(f, "Frequency: {:9.2} Phase: {:5.3} Amplitude: {:6.3}", self.frequency, self.phase, self.amplitude)
        } else {
            write!(f, "f:{:.2} p:{:.3} a:{:.3}", self.frequency, self.phase, self.amplitude)
        }
    }
}

#[derive(Copy, Clone, Debug)]
struct Color(u8, u8, u8);

fn color(f: &mut std::fmt::Formatter<'_>, c: Color) -> std::fmt::Result {
    write!(f, "\x1b[38;2;{};{};{}m", c.0, c.1, c.2)
}

fn reset(f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "\x1b[0m")
}

impl std::fmt::Display for FftPoint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let c = self.character();
        if c == ' ' {
            write!(f, "{}", self.character())
        } else {
            color(f, self.color())?;
            write!(f, "{}", self.character())?;
            reset(f)
        }
    }
}

pub fn listen() -> Stream {
    let host = cpal::default_host();
    let device = host.default_input_device().unwrap();

    let mut config = device
        .default_input_config()
        .unwrap()
        .config();

    // config.buffer_size = BufferSize::Fixed(2048);

    println!("{:?}", config);

    let mut first = true;
    
    let stream = device.build_input_stream(
        &config,
        move |samples: &[f32], info: &cpal::InputCallbackInfo| {

            let mut planner = FftPlanner::<f32>::new();
            let mut buffer = to_complexes(samples, config.channels.into());
            let fft = planner.plan_fft_forward(buffer.len());

            fft.process(&mut buffer);

            let fbucket = config.sample_rate.0 as f32 / 2.0 / (buffer.len() / 2) as f32;

            if first {
                println!("fbucket: {}", fbucket);
                first = false;
            }

            let values = &buffer[0..buffer.len()/2].iter().enumerate()
                .map(|(i, &v)| FftPoint::new(fbucket, (i, v))).collect::<Vec<_>>();

            let peak = values.iter().max_by_key(|v| (v.amplitude * 100000.) as i64).unwrap();

            // for i in 13..17 {
            //     print!("{:?} ", values[i]);
            // }
            // println!();
            // println!("{:#?}", values[14]);
            // println!("{:#?}", peak);

            // println!("{}", fbucket);

            values.iter().take(120).for_each(|v| eprint!("{}", v));
            eprintln!();
        },
        move |err| eprintln!("{:?}", err),
    ).unwrap();

    stream.play().unwrap();

    stream
}