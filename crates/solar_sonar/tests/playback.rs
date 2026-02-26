use solar_sonar as sonar;
use pretty_assertions::assert_eq;
use std::{path::Path, sync::OnceLock};


const TEST_CONFIG_DIR: &'static str = "$CARGO_MANIFEST_DIR/assets/tests/config";

struct TestConfig {
    characters: &'static [TestCharacterConfig],
    logs_dir: &'static str,
}

impl Into<sonar::Config> for TestConfig {
    fn into(self) -> sonar::Config {
        sonar::Config {
            characters: self.characters.into_iter()
                .map(|c| c.into())
                .collect::<Vec<_>>(),
            logs_dir: shellexpand::path::full(self.logs_dir).unwrap().into(),
        }
    }
}

struct TestCharacterConfig {
    alias: &'static str,
    id: u32,
    id_str: &'static str,
    intel_channels: &'static [&'static str],
}

impl Into<sonar::CharacterConfig> for &TestCharacterConfig {
    fn into(self) -> sonar::CharacterConfig {
        sonar::CharacterConfig {
            alias: self.alias.to_string(),
            id: self.id,
            id_str: self.id_str.to_string(),
            intel_channels: self.intel_channels.into_iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>(),
        }
    }
}

const TEST_CONFIG: TestConfig = TestConfig {
    characters: &[
        TestCharacterConfig {
            alias: "test",
            id: 12345,
            id_str: "12345",
            intel_channels: &[
                "test.intel",
            ],
        },
    ],
    logs_dir: "$CARGO_MANIFEST_DIR/assets/tests/logs",
};

fn test_config() -> sonar::Config { TEST_CONFIG.into() }

fn home_config() -> sonar::Config {
    sonar::Config::read().unwrap()
}

fn setup() {
    static ONCE: OnceLock<()> = OnceLock::new();
    ONCE.get_or_init(|| { sonar::SolarSonar::init_once().unwrap(); });
}

#[test]
fn test_configs() {
    setup();
    let actual = sonar::Config::read_dir(Path::new(TEST_CONFIG_DIR)).unwrap();
    assert_eq!(test_config(), actual);
}

const TEST_SYSTEM: &'static str = "UALX-3";
fn test_system() -> &'static sonar::SolarSystem {
    sonar::StarMap::get().system_named(TEST_SYSTEM)
}

fn test_args(cfg: &sonar::Config, sys: &sonar::SolarSystem) -> sonar::Args {
    sonar::Args {
        watch_character_ids: vec![cfg.characters[0].id],
        watch_system_ids: vec![sys.id],
        jumps: sonar::Args::DEFAULT_JUMPS,
        stdio: true,
    }
}

/*
#[tokio::test]
async fn test_play_fixture() {
    setup();
    let sys = test_system();
    let cfg = test_config(); 
    let args = test_args(&cfg, sys);
    let (event_io, mut event_rx) = sonar::SolarSonar::make_io();
    let handle = sonar::start(args, cfg, event_io).unwrap();
    let mut events = Vec::new();
    tokio::time::timeout(Duration::from_secs(10), async {
        let _ = event_rx.recv_many(&mut events, 5_000).await;
    }).await.unwrap();
    dbg!(events);
}*/

#[ignore]
#[tokio::test]
async fn live() {
    setup();
    let sys = test_system();
    let cfg = home_config(); 
    let mut args = test_args(&cfg, sys);
    args.jumps = 10;
    let (event_io, mut event_rx) = sonar::SolarSonar::make_io();
    let _handle = sonar::start(args, cfg, event_io).unwrap();

    loop {
        tokio::select! {
            Some(event) = event_rx.recv() => {
                dbg!(event);
            },
            else => break,
        }
    }
}
