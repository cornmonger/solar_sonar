use crate::*;

#[derive(Debug)]
pub(crate) struct SonarAudio {
    play_tx: r::tokio::mpsc::UnboundedSender<DataEvent>
}

impl SonarAudio {
    pub fn init() -> SolarResult<Self> {
        let (play_tx, mut play_rx) = r::tokio::mpsc::unbounded_channel();
        std::thread::spawn(move || {
            let Ok(audible) = Audible::init() else {
                panic!("Failed to initialize audio");
            };
            
            std::thread::sleep(Duration::from_secs(1)); // give rodio time to catch up 
            
            while let Some(play) = play_rx.blocking_recv() {
                let result = match play {
                    DataEvent::PingFortune => on_ping_fortune(&audible),
                    DataEvent::PingSystems { system_ids } => on_ping_systems(&audible, system_ids),
                    _ => Ok(()),
                };
                
                if result.is_err() {
                    break;
                }
            }
        });
        
        Ok(Self { play_tx })
    }
}

impl SonarListener for SonarAudio {
    async fn on_cancel(&self, _run: &Running) {}
    
    async fn on_event(&self, _run: &Running, event: &DataEvent) -> SolarResult<()> {
        match event {
            DataEvent::PingFortune => {
                self.play_tx.send(DataEvent::PingFortune)
                    .map_err(|_| SolarError::msg("closed"))
            },
            DataEvent::PingSystems { system_ids } => {
                self.play_tx.send(DataEvent::PingSystems { system_ids: system_ids.clone() })
                    .map_err(|_| SolarError::msg("closed"))
            },
            _ => Ok(()),
        }
    }
}

fn on_ping_fortune(audible: &Audible) -> SolarResult<()> {
    audible.play_ping(vec![fortune()])
}

fn on_ping_systems(audible: &Audible, system_ids: Vec<SolarId>) -> SolarResult<()> {
    let names = STAR_MAP.systems(system_ids)
        .into_iter()
        .map(|sys| sys.name)
        .collect::<Vec<_>>();
    
    audible.play_ping_systems(names)
}
