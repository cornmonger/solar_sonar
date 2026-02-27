use solar_sonar as sonar;
use pretty_assertions::assert_eq;
use std::{ffi::OsStr, path::{Path, PathBuf}, sync::OnceLock};


const TEST_CONFIG_DIR: &'static str = "$CARGO_MANIFEST_DIR/assets/tests/config";

struct TestConfig {
    characters: &'static [TestCharacterCfg],
    logs_dir: &'static str,
}

fn expand_path<P: AsRef<OsStr>>(path: P) -> PathBuf {
    shellexpand::path::full(path.as_ref()).unwrap().into()
}

impl Into<sonar::Cfg> for TestConfig {
    fn into(self) -> sonar::Cfg {
        sonar::Cfg {
            characters: self.characters.iter()
                .map(|c| c.into())
                .collect::<Vec<_>>(),
            settings: sonar::SettingsCfg {
                logs_dir: expand_path(self.logs_dir),
            },
        }
    }
}

struct TestCharacterCfg {
    alias: &'static str,
    id: u32,
    intel_channels: &'static [&'static str],
}

impl Into<sonar::CharacterCfg> for &TestCharacterCfg {
    fn into(self) -> sonar::CharacterCfg {
        sonar::CharacterCfg {
            alias: self.alias.to_string(),
            id: self.id,
            intel_channels: self.intel_channels.iter()
                .map(|s| s.to_string())
                .collect(),
       }
    }
}

const TEST_CONFIG: TestConfig = TestConfig {
    characters: &[
        TestCharacterCfg {
            alias: "test",
            id: 12345,
            intel_channels: &[
                "test.intel",
            ],
        },
    ],
    logs_dir: "$CARGO_MANIFEST_DIR/assets/tests/logs",
};

fn test_config() -> sonar::CfgParam {
    let cfg: sonar::Cfg = TEST_CONFIG.into();
    cfg.build().expect("build")
}

fn home_config() -> sonar::CfgParam {
    sonar::Cfg::read().expect("home")
}

fn setup() {
    static ONCE: OnceLock<()> = OnceLock::new();
    ONCE.get_or_init(|| { sonar::SolarSonar::init_once().unwrap(); });
}

#[test]
fn test_configs() {
    setup();
    let actual = sonar::Cfg::read_dir(Path::new(TEST_CONFIG_DIR)).unwrap();
    assert_eq!(test_config(), actual);
}

const TEST_SYSTEM: &'static str = "UALX-3";
fn test_system() -> &'static sonar::SolarSystem {
    sonar::StarMap::get().system_named(TEST_SYSTEM)
}

fn test_args(cfg_param: &sonar::CfgParam, sys: &sonar::SolarSystem) -> sonar::ArgParam {
    sonar::Args {
        watch_character_ids: vec![cfg_param.config.characters[0].id],
        watch_system_ids: vec![sys.id],
        jumps: sonar::Args::DEFAULT_JUMPS,
        stdio: false,
        audio: false,
        replay_file: None,
    }.build().expect("args")
}


#[tokio::test]
async fn test_play_fixture() {
    const REPLAY: &'static str = "$CARGO_MANIFEST_DIR/assets/tests/logs/Chatlogs/test.intel_20260217_130052_12345.txt";

    setup();
    let sys = test_system();
    let cfg_param = test_config(); 
    let mut arg_param = test_args(&cfg_param, sys);
    arg_param.args.replay_file = Some(expand_path(REPLAY));

    let (event_io, mut event_rx) = sonar::SolarSonar::make_io();
    let _handle = sonar::start(arg_param, cfg_param, event_io).unwrap();

    loop {
        tokio::select! {
            Some(event) = event_rx.recv() => {
                dbg!(event);
            },
            else => break,
        }
    }
}


#[ignore]
#[tokio::test]
async fn live() {
    setup();
    let sys = test_system();
    let cfg_param = home_config(); 
    let mut arg_param = test_args(&cfg_param, sys);
    arg_param.args.audio = true;
    arg_param.args.stdio = true;
    arg_param.args.jumps = 10;
    let (event_io, mut event_rx) = sonar::SolarSonar::make_io();
    let _handle = sonar::start(arg_param, cfg_param, event_io).unwrap();

    loop {
        tokio::select! {
            Some(event) = event_rx.recv() => {
                dbg!(event);
            },
            else => break,
        }
    }
}
