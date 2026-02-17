pub(crate) mod assets;
pub(crate) mod error;
pub(crate) mod cli;
pub(crate) mod config;
pub(crate) mod fortune;
pub(crate) mod generated {
    pub(crate) mod starmap;
}
pub(crate) mod logs;
pub(crate) mod model;
pub(crate) mod paths;
pub(crate) mod audio;
pub(crate) mod run;
pub(crate) mod stdio;

pub use self::{
    model::*,
    run::{run,run_with},
};

pub(crate) use self::{
    assets::*,
    error::*,
    cli::*,
    config::*,
    fortune::*,
    generated::starmap::*,
    logs::*,
    paths::*,
    audio::*,
    run::*,
    stdio::*,
};

pub(crate) use std::{
    collections::HashMap,
    borrow::Cow,
    env,
    error::Error,
    fs::{
        self,
        File,
    },
    io::{
        self,
        BufRead,
        BufReader,
        Seek,
        SeekFrom,
        Write,
    },
    path::{Path, PathBuf},
    process::{Command, ExitCode},
    sync::OnceLock,
    time::{Duration},
};

pub(crate) use clap::Parser;
pub(crate) use chrono::{NaiveDateTime, DateTime, Utc};
pub(crate) use itertools::Itertools;
pub(crate) use heck::ToSnakeCase;
pub(crate) use encoding_rs_io::DecodeReaderBytesBuilder;
pub(crate) use encoding_rs::UTF_16LE;
pub(crate) use const_format::formatcp;
