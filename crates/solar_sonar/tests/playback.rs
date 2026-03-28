use solar_sonar as sonar;
use pretty_assertions::assert_eq;
use std::{ffi::OsStr, path::{Path, PathBuf}, sync::OnceLock};

#[derive(Clone, Copy)]
enum TestKind {
    Standard,
    TlsClient,
}

impl TestKind {
    const STANDARD_CONFIG_DIR: &'static str = "$CARGO_MANIFEST_DIR/assets/tests/config/standard";
    const TLS_CLIENT_CONFIG_DIR: &'static str = "$CARGO_MANIFEST_DIR/assets/tests/config/tls_client";
    
    fn config_dir(&self) -> &Path {
        match self {
            Self::Standard => Path::new(Self::STANDARD_CONFIG_DIR),
            Self::TlsClient => Path::new(Self::TLS_CLIENT_CONFIG_DIR),
        }
    }
    
    fn cfg(&self) -> TestCfg {
        match self {
            Self::Standard => TEST_STANDARD_CONFIG,
            Self::TlsClient => TEST_TLS_CLIENT_CONFIG,
        }
    }
}

impl IntoIterator for TestKind {
    type Item = Self;
    type IntoIter = std::array::IntoIter<Self, 2>;

    fn into_iter(self) -> Self::IntoIter {
        [Self::Standard, Self::TlsClient].into_iter()
    }
}

struct TestCfg {
    characters: &'static [TestCharacterCfg],
    logs_dir: &'static str,
    modes: &'static [&'static str],
    server: &'static TestServerCfg,
    client: &'static TestClientCfg,
    logs: &'static [TestLogCfg],
}

fn expand_path<P: AsRef<OsStr>>(path: P) -> PathBuf {
    shellexpand::path::full(path.as_ref()).unwrap().into()
}

impl Into<sonar::Cfg> for TestCfg {
    fn into(self) -> sonar::Cfg {
        sonar::Cfg {
            characters: self.characters.iter()
                .map(|c| c.into())
                .collect::<Vec<_>>(),
            settings: sonar::SettingsCfg {
                logs_dir: PathBuf::from(self.logs_dir),
                modes: self.modes.iter()
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>()
            },
            logs: self.logs.iter()
                .map(|c| c.into())
                .collect::<Vec<_>>(),
            server: self.server.into(),
            client: self.client.into(),
        }
    }
}

struct TestCharacterCfg {
    alias: &'static str,
    id: u32,
    name: &'static str,
}

impl Into<sonar::CharacterCfg> for &TestCharacterCfg {
    fn into(self) -> sonar::CharacterCfg {
        sonar::CharacterCfg {
            alias: self.alias.to_string(),
            id: self.id,
            name: self.name.to_string(),
       }
    }
}

struct TestServerCfg {
    default: Option<&'static str>,
    serve: &'static [&'static TestServeCfg],
}

struct TestClientCfg {
    default: Option<&'static str>,
    connect: &'static [&'static TestConnectCfg],
}

struct TestServeCfg {
    name: &'static str,
    tls: &'static TestTlsCfg,
}

impl Into<sonar::ServeCfg> for &TestServeCfg {
    fn into(self) -> sonar::ServeCfg {
        sonar::ServeCfg {
            name: self.name.to_string(),
            tls: self.tls.into(),
        }
    }
}

impl Into<sonar::ServerCfg> for &TestServerCfg {
    fn into(self) -> sonar::ServerCfg {
        sonar::ServerCfg {
            default: self.default.map(|s| s.to_string()),
            serve: self.serve.iter()
                .map(|c| (*c).into())
                .collect(),
        }
    }
}

struct TestConnectCfg {
    name: &'static str,
    tls: &'static TestTlsCfg,
}

impl Into<sonar::ConnectCfg> for &TestConnectCfg {
    fn into(self) -> sonar::ConnectCfg {
        sonar::ConnectCfg {
            name: self.name.to_string(),
            tls: self.tls.into(),
        }
    }
}

impl Into<sonar::ClientCfg> for &TestClientCfg {
    fn into(self) -> sonar::ClientCfg {
        sonar::ClientCfg {
            default: self.default.map(|s| s.to_string()),
            connect: self.connect.iter()
                .map(|c| (*c).into())
                .collect(),
        }
    }
}

struct TestTlsCfg {
    ip: &'static str,
    port: u16,
}

impl Into<sonar::TlsCfg> for &TestTlsCfg {
    fn into(self) -> sonar::TlsCfg {
        sonar::TlsCfg {
            ip: self.ip.to_string(),
            port: self.port,
        }
    }
}

pub struct TestLogCfg {
    kind: sonar::LogKind,
    name: Option<&'static str>,
    characters: &'static [&'static str],
    modes: &'static [&'static str],
    analysis: &'static [&'static str],
}

impl Into<sonar::LogCfg> for &TestLogCfg {
    fn into(self) -> sonar::LogCfg {
        sonar::LogCfg {
            kind: self.kind.as_input_str().to_string(),
            name: self.name.map(|s| s.to_string()),
            modes: self.modes.iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>(),
            characters: self.characters.iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>(),
            analysis: self.analysis.iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>(),
        }
    }
}

const TEST_STANDARD_CONFIG: TestCfg = TestCfg {
    characters: &[
        TestCharacterCfg {
            alias: "test",
            id: 12345,
            name: "Test",
        },
    ],
    logs_dir: "$CARGO_MANIFEST_DIR/assets/tests/logs",
    modes: &["stand", "crab", "fleet"],
    logs: &[
        TestLogCfg {
            kind: sonar::LogKind::Group,
            name: Some("test.intel"),
            characters: &["test"],
            modes: &["stand", "crab", "fleet"],
            analysis: &["intel"],
        },
    ],
    server: &TestServerCfg {
        default: Some("test"),
        serve: &[
            &TestServeCfg {
                name: "test",
                tls: &TestTlsCfg {
                    ip: "127.0.0.1",
                    port: 3232,
                },
            } 
        ],
    },
    client: &TestClientCfg {
        default: None,
        connect: &[],
    },
};

const TEST_TLS_CLIENT_CONFIG: TestCfg = TestCfg {
    characters: &[
        TestCharacterCfg {
            alias: "test",
            id: 12345,
            name: "Test",
        },
    ],
    logs_dir: "$CARGO_MANIFEST_DIR/assets/tests/logs",
    modes: &["stand", "crab", "fleet"],
    logs: &[
        TestLogCfg {
            kind: sonar::LogKind::Group,
            name: Some("test.intel"),
            characters: &["test"],
            modes: &["stand", "crab", "fleet"],
            analysis: &["intel"],
        },
    ],
    server: &TestServerCfg {
        default: Some("test"),
        serve: &[
            &TestServeCfg {
                name: "test",
                tls: &TestTlsCfg {
                    ip: "127.0.0.1",
                    port: 3232,
                },
            } 
        ],
    },
    client: &TestClientCfg {
        default: Some("test_tls"),
        connect: &[
            &TestConnectCfg {
                name: "test_tls",
                tls: &TestTlsCfg {
                    ip: "127.0.0.1",
                    port: 3232,
                }
            },
        ],
    },
};

fn test_config(kind: TestKind) -> sonar::Cfg {
    let cfg: sonar::Cfg = kind.cfg().into();
    cfg
}

fn home_config() -> sonar::Cfg {
    sonar::Cfg::read().expect("home")
}

fn setup() {
    static ONCE: OnceLock<()> = OnceLock::new();
    ONCE.get_or_init(|| { sonar::SolarSonar::init_once().unwrap(); });
}

#[test]
fn test_configs() {
    setup();
    
    for kind in TestKind::Standard.into_iter() {
        let actual = sonar::Cfg::read_dir(kind.config_dir()).unwrap();
        assert_eq!(test_config(kind), actual);
    }
}

const TEST_SYSTEM: &'static str = "UALX-3";
fn test_system() -> &'static sonar::SolarSystem {
    sonar::StarMap::get().system_named(TEST_SYSTEM)
}

fn test_args(cfg: &sonar::Cfg, sys: &sonar::SolarSystem) -> sonar::Args {
    sonar::Args {
        watch_character_ids: vec![cfg.characters[0].id],
        watch_system_ids: vec![sys.id],
        jumps: sonar::Args::DEFAULT_JUMPS,
        stdio: false,
        audio: false,
        replay_file: None,
        server_profile: None,
        client_profile: None,
    }
}


#[tokio::test]
async fn test_play_fixture() {
    const REPLAY: &'static str = "$CARGO_MANIFEST_DIR/assets/tests/logs/Chatlogs/test.intel_20260217_130052_12345.txt";

    setup();
    let sys = test_system();
    let cfg = test_config(TestKind::Standard); 
    let mut args = test_args(&cfg, sys);
    args.replay_file = Some(expand_path(REPLAY));
    let params = sonar::ParamsBuilder::new()
        .cfg(cfg)
        .args(args)
        .build()
        .expect("ok");
    
    let mut handle = sonar::start(params).unwrap();

    loop {
        tokio::select! {
            Ok(event) = handle.recv() => {
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
    let cfg = home_config(); 
    let mut args = test_args(&cfg, sys);
    args.audio = true;
    args.stdio = true;
    args.jumps = 10;
    let params = sonar::ParamsBuilder::new()
        .cfg(cfg)
        .args(args)
        .build()
        .expect("ok");
    
    let mut handle = sonar::start(params).unwrap();

    loop {
        tokio::select! {
            Ok(event) = handle.recv() => {
                dbg!(event);
            },
            else => break,
        }
    }
}
