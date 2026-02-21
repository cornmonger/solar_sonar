use crate::*;
use rodio::source::{SineWave, Source};
use rodio::buffer::SamplesBuffer;
use piper_rs::synth::PiperSpeechSynthesizer;

pub(crate) fn play_ping_systems(names: Vec<&str>) -> SolarResult<()> {
    let names = names.into_iter().map(|s| abbreviate(s)).collect::<Vec<_>>();
    play_ping(names)
}

pub(crate) fn play_ping<S: Into<String>>(texts: Vec<S>) -> SolarResult<()> {
    let mut stream_handle = rodio::OutputStreamBuilder::open_default_stream()
        .map_err(|_| SolarError::msg("Failed to open audio stream"))?;

    stream_handle.log_on_drop(false);
    let sink = rodio::Sink::connect_new(&stream_handle.mixer());

    // alert tone: 440hz
    let source = SineWave::new(440.0)
        .take_duration(Duration::from_secs_f32(0.5))
        .amplify(1.);
    sink.append(source);
    sink.sleep_until_end();

    let config_filepath = RemoteAssets::VoiceModelConfig.get().filepath()?;
    let model = piper_rs::from_config_path(&config_filepath)
        .map_err(|_| SolarError::msg(format!("Unable to load voice model config: {}", log_path(&config_filepath))))?;

    model.set_speaker(80);
    let synth = PiperSpeechSynthesizer::new(model)
        .map_err(|_| SolarError::msg("Failed to create speech synthesizer"))?;
    let mut samples: Vec<f32> = Vec::new();

    for text in texts {
        let audio = synth.synthesize_parallel(text.into(), None)
            .map_err(|_| SolarError::msg("Failed to pronounce: {text}"))?;

        for result in audio {
            let sample = result
                .map_err(|_| SolarError::msg("Failed to pronounce: {text}"))?;

            samples.append(&mut sample.into_vec());
        }
    }

    let buf = SamplesBuffer::new(1, 22050, samples);
    sink.append(buf);
    sink.sleep_until_end();

    Ok(())
}

fn abbreviate(s: &str) -> String {
    let s = s.chars()
        .map(|c| match c {
            '-' => "Tac".to_string(),
            _ => c.to_uppercase().to_string()
        })
        .collect::<Vec<_>>()
        .join(" ")
        + ".";
        
    s
}

