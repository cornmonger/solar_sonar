use crate::*;
use rodio::source::{SineWave, Source};
use rodio::buffer::SamplesBuffer;
use piper_rs::synth::PiperSpeechSynthesizer;

pub(crate) struct Audible {
    #[allow(unused)]
    stream_handle: rodio::OutputStream,
    sink: rodio::Sink,
    synth: PiperSpeechSynthesizer,
}

impl Audible {
    pub(crate) fn init() -> SolarResult<Self> {
        let mut stream_handle = rodio::OutputStreamBuilder::open_default_stream()
            .map_err(|_| SolarError::msg("Failed to open audio stream"))?;
        stream_handle.log_on_drop(false);
        
        let sink = rodio::Sink::connect_new(&stream_handle.mixer());
        
        let config_filepath = RemoteAssets::VoiceModelConfig.asset().filepath()?;
        let model = piper_rs::from_config_path(&config_filepath)
            .map_err(|_| SolarError::msg(format!("Unable to load voice model config: {}", log_path(&config_filepath))))?;
    
        model.set_speaker(80);
        let synth = PiperSpeechSynthesizer::new(model)
            .map_err(|_| SolarError::msg("Failed to create speech synthesizer"))?;
        
        
        Ok(Self { stream_handle, sink, synth })
    }

    pub(crate) fn play_ping_systems(&self, names: Vec<&str>) -> SolarResult<()> {
        let names = names.into_iter().map(|s| abbreviate(s)).collect::<Vec<_>>();
        self.play_ping(names)
    }
    
    pub(crate) fn play_ping<S: Into<String>>(&self, texts: Vec<S>) -> SolarResult<()> {
        // alert tone: 440hz
        let source = SineWave::new(440.0)
            .take_duration(Duration::from_secs_f32(0.5))
            .amplify(1.);
        self.sink.append(source);
    
        let mut samples: Vec<f32> = Vec::new();
            for text in texts {
            let audio = self.synth.synthesize_parallel(text.into(), None)
                .map_err(|_| SolarError::msg("Failed to pronounce: {text}"))?;
    
            for result in audio {
                let sample = result
                    .map_err(|_| SolarError::msg("Failed to pronounce: {text}"))?;
    
                samples.append(&mut sample.into_vec());
            }
        }
    
        let buf = SamplesBuffer::new(1, 22050, samples);
        self.sink.append(buf);
        self.sink.sleep_until_end();
    
        Ok(())
    }
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

