use solar_sonar as sonar;
use pretty_assertions::assert_eq;
use std::{ffi::OsStr, path::{Path, PathBuf}, sync::OnceLock};

const TEST_STANDARD_CONFIG_DIR: &'static str = "tests/assets/standard/config";
const EXPECTED_RON_FILE_STANDARD: &'static str = "tests/assets/standard/generated/expected/solar_sonar.log.ron";

const TEST_STANDARD_CONFIG: sonar::CfgConst = sonar::CfgConst {
    characters: &[
        sonar::CharacterCfgConst {
            alias: "test",
            id: 12345,
            name: "Test",
        },
    ],
    settings: sonar::SettingsCfgConst {
        logs_dir: "$CARGO_MANIFEST_DIR/tests/assets/logs/intel",
        modes: &["stand", "crab", "fleet"],
    },
    logs: &[
        sonar::LogCfgConst {
            kind: sonar::LogKind::Group,
            name: Some("test.intel"),
            characters: &["test"],
            modes: &["stand", "crab", "fleet"],
            analysis: &[sonar::AnalysisKind::Intel],
        },
    ],
    server: sonar::ServerCfgConst {
        serve: &[
            sonar::ServeCfgConst {
                name: "test",
                tls: sonar::TlsCfgConst {
                    ip: "127.0.0.1",
                    port: 3232,
                },
            } 
        ],
    },
    client: sonar::ClientCfgConst {
        connect: &[
            sonar::ConnectCfgConst {
                name: "test",
                tls: sonar::TlsCfgConst {
                    ip: "127.0.0.1",
                    port: 3232,
                },
            },
        ],
    },
    modes: &[
        sonar::NamedModeCfgConst {
            name: "crab",
            combat_analysis: sonar::CombatAnalysisModeCfgConst {
                cease_time: None,
                engagement: None,
                targeted: None,
                yellowboxers: None,
            },
        },
    ],
};

fn make_args(cfg: &sonar::Cfg, sys: &sonar::StarSystem) -> sonar::Args {
    sonar::Args {
        watch_character_ids: vec![cfg.characters[0].id],
        watch_system_ids: vec![sys.id],
        jumps: sonar::Args::DEFAULT_JUMPS,
        stdio: false,
        audio: false,
        replay_path: None,
        server_profile: None,
        client_profile: None,
        mode: "crab".to_string(),
        relog: None,
    }
}

const TEST_SYSTEM: &'static str = "UALX-3";
fn standard_starsys() -> &'static sonar::StarSystem {
    sonar::StarMap::get().system_named(TEST_SYSTEM)
}

fn expand_path<P: AsRef<OsStr>>(path: P) -> PathBuf {
    shellexpand::path::full(path.as_ref()).unwrap().into()
}

fn read_expected_data_standard() -> sonar::DataLog {
    sonar::DataLog::read_ron(Path::new(EXPECTED_RON_FILE_STANDARD)).expect("exists")
}

fn setup_test() {
    static ONCE: OnceLock<()> = OnceLock::new();
    ONCE.get_or_init(|| { sonar::SolarSonar::init_once().unwrap(); });
}

#[test]
fn test_cfg() {
    setup_test();
    let expected = sonar::Cfg::from(&TEST_STANDARD_CONFIG);
    let actual = sonar::Cfg::read_dir(Path::new(TEST_STANDARD_CONFIG_DIR)).unwrap();
    assert_eq!(expected, actual);
}

#[tokio::test]
async fn test_replay_dir() {
    const REPLAY_DIR: &'static str = "$CARGO_MANIFEST_DIR/tests/assets/standard/logs";

    setup_test();
    let sys = standard_starsys();
    let cfg = sonar::Cfg::from(&TEST_STANDARD_CONFIG); 
    let mut args = make_args(&cfg, sys);
    args.replay_path = Some(expand_path(REPLAY_DIR));
    let params = sonar::ParamsBuilder::new()
        .cfg(cfg)
        .args(args)
        .build()
        .expect("ok");
    
    let mut handle = sonar::start(params).await.unwrap();
    let expected_log = read_expected_data_standard();
    let mut events = Vec::with_capacity(expected_log.events.len());

    loop {
        tokio::select! {
            Ok(event) = handle.recv() => events.push(event),
            else => break,
        }
    }
    
    assert_eq!(expected_log.events, events);
}
