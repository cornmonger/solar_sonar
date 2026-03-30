pub(crate) mod analysis {
    pub(crate) mod analyzer;
    pub(crate) mod intel;
    pub(crate) mod callout;
    pub(crate) mod combat;
    pub(crate) mod message;
}
pub(crate) mod args;
pub(crate) mod assets;
pub(crate) mod audio;
pub(crate) mod error;
pub(crate) mod cli;
pub(crate) mod cfg {
    pub(crate) mod characters;
    pub(crate) mod client;
    pub(crate) mod config;
    pub(crate) mod logs;
    pub(crate) mod mode {
        pub(crate) mod analysis {
            pub(crate) mod combat;
        }
        pub(crate) mod mode;
    }
    pub(crate) mod server;
    pub(crate) mod settings;
    pub(crate) mod tls;
}
pub(crate) mod fortune;
pub(crate) mod generated {
    pub(crate) mod starmap;
}
pub(crate) mod indices {
    pub(crate) mod channel_names;
    pub(crate) mod characters;
    pub(crate) mod index;
    pub(crate) mod modes;
}
pub(crate) mod logs;
pub(crate) mod model {
    pub(crate) mod msg {
        pub(crate) mod event;
        pub(crate) mod log;
        pub(crate) mod analysis;
    }
    pub(crate) mod eve {
        pub(crate) mod star_map;
        pub(crate) mod types;
    }
}
pub(crate) mod paths;
pub(crate) mod map {
    pub(crate) mod nav;
}
pub(crate) mod run;
pub(crate) mod sonar_io {
    pub(crate) mod sonar;
    pub(crate) mod stdio {
        pub(crate) mod stdio;
        pub(crate) mod ansi;
    }
    pub(crate) mod audio;
}
pub(crate) mod tls {
    pub(crate) mod certificate;
    pub(crate) mod client;
    pub(crate) mod frame;
    pub(crate) mod server;
}

pub use self::{
    args::{Args, ArgParam},
    cfg::{
        characters::*,
        client::*,
        config::*,
        logs::*,
        mode::{
            analysis::{
                combat::*,
            },
            mode::*,
        },
        server::*,
        settings::*,
        tls::*,
    },
    generated::starmap::STAR_MAP,
    indices::{
        channel_names::*,
        characters::*,
        index::*,
        modes::*,
    },
    map::{
        nav::*,
    },
    model::{
        eve::{
            star_map::*,
            types::*,
        },
        msg::{
            event::*,
            log::*,
            analysis::*,
        },
    },
    run::{run_cli,start,SolarSonar, ParamsBuilder, Params},
};

pub(crate) use self::{
    analysis::{
        analyzer::*,
        callout::*,
        combat::*,
        intel::*,
        message::*,
    },
    assets::*,
    error::*,
    cli::*,
    fortune::*,
    logs::*,
    paths::*,
    audio::*,
    run::*,
    sonar_io::{
        audio::*,
        sonar::*,
        stdio::{
            ansi::*,
            stdio::*,
        },
    },
    tls::{
        certificate::*,
        client::*,
        frame::*,
        server::*,
    },
};

pub(crate) use std::{
    collections::HashMap,
    borrow::Cow,
    env,
    error::Error,
    fmt::Display,
    fs::{
        self,
        File,
    },
    hash::{Hash, Hasher},
    io::{
        self,
        BufRead,
        BufReader,
        Seek,
        SeekFrom,
        Write,
    },
    marker::PhantomData,
    mem,
    net::{self, IpAddr, Ipv4Addr},
    path::{Path, PathBuf},
    process::{Command, ExitCode},
    slice,
    sync::{Arc, OnceLock},
    time::{Duration},
};

pub(crate) use bzip2::read::BzDecoder;
pub(crate) use clap::Parser;
pub(crate) use chrono::{NaiveDateTime, DateTime, Utc};
pub(crate) use const_format::formatcp;
pub(crate) use encoding_rs_io::DecodeReaderBytesBuilder;
pub(crate) use encoding_rs::UTF_16LE;
pub(crate) use futures::{StreamExt, SinkExt, future::{join_all, BoxFuture}};
pub(crate) use heck::ToSnakeCase;
pub(crate) use itertools::Itertools;
pub(crate) use xxhash_rust::xxh3::xxh3_64;

/// re-exports
pub(crate) mod r {
    pub(crate) mod tls {
        pub(crate) use tokio_rustls::{
            TlsAcceptor,
            TlsConnector,
            client,
            rustls::{
                pki_types::{
                    CertificateDer,
                    PrivateKeyDer,
                    ServerName,
                },
                RootCertStore,
            },
            server,
        };
    }
    pub(crate) mod tokio {
        pub(crate) use tokio::{
            io::{ReadHalf, WriteHalf},
            task::{
                JoinHandle,
            },
            net::{
                TcpListener,
                TcpStream,
            },
            sync::{
                broadcast,
                mpsc,
            },
            time::timeout,
        };
        pub(crate) use tokio_util::{
            bytes::BytesMut,
            codec::{
                Decoder,
                Encoder,
                LengthDelimitedCodec,
                FramedRead,
                FramedWrite
            },
            sync::CancellationToken,
        };
    }
}

pub(crate) use tokio_rustls::{
    rustls::{
        self, pki_types::pem::PemObject,
    },
};

