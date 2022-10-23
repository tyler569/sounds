fn main() {
    let mut generator = ToneGenerator { sample_rate: 44100 };
    let mut buffer = [0; 2048];

    generator.add_waveform(FrequencyComponent::new_simple(340));
    generator.add_waveform(FrequencyComponent::new_simple(450));

    let mut decoder = FftDecoder { sample_rate: 44100 };

    for i in 0..16 {
        generator.generate(&mut buffer);
        let fft = decoder.decode(&buffer);
    }
}