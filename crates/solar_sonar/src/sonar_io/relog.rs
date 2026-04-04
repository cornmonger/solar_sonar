use tokio::io::AsyncWriteExt;
use tokio_util::codec::FramedWrite;

use crate::*;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum RelogFormat {
    Binary,
    Ron,
}

impl RelogFormat {
    pub fn from_filename(filename: Option<&str>) -> Self {
        match filename {
            Some("*.ron.log") => Self::Ron,
            Some("*.bin.log") | _ => Self::Binary,
        }
    }
    
    pub fn ext(&self) -> &'static str {
        match self {
            Self::Binary => ".bin.log",
            Self::Ron => ".ron.log",
        }
    }
}

pub(crate) struct SonarRelog {
    output: PathBuf,
    format: RelogFormat,
    inner: Mutex<InnerMut>,
}

struct InnerMut {
    session_date: NaiveDate,
    filepath: PathBuf,
    writer: Writer, 
}

enum Writer {
    None,
    Binary(r::tokio::FramedWrite<tokio::io::BufWriter<tokio::fs::File>, BitcodeCodec<DataEvent>>),
    Ron(tokio::io::BufWriter<tokio::fs::File>),
}

impl Writer {
    async fn new(filepath: &Path, format: RelogFormat) -> SolarResult<Self> {
        let mut fsopts = tokio::fs::OpenOptions::new();
        fsopts.append(true);
        let file = fsopts.open(&filepath).await
            .map_err(|e| SolarError::write(e, &filepath))?;
        let file_writer = tokio::io::BufWriter::new(file);
        
        match format {
            RelogFormat::Binary => {
                let frame_writer = r::tokio::FramedWrite::new(file_writer, BitcodeCodec::new());
                Ok(Writer::Binary(frame_writer))
            },
            RelogFormat::Ron => {
                Ok(Writer::Ron(file_writer))
            }
        }
    }
    
    async fn write(&mut self, item: &DataEvent) -> SolarResult<()> {
        match self {
            Self::Binary(writer) => {
                writer.send(item).await.unwrap();
            },
            Self::Ron(writer) => {
                let ron_options = make_ron_options();
                let ron_config = make_ron_config(&ron_options);
                let mut ron = ron_options.to_string_pretty(item, ron_config)
                    .map_err(|e| SolarError::msg(e.to_string()))?;
                ron.push_str(",\n");
                writer.write_all(ron.as_bytes()).await
                    .map_err(|e| SolarError::msg(e.to_string()))?;
            },
            Self::None => unreachable!("No writer exists"),
        }
        
        Ok(())
    }
    
    async fn close(&mut self) {
        match self {
            Self::Binary(writer) => {
                let _ = writer.flush().await;
                *self = Writer::None;
            },
            Self::Ron(writer) => {
                let _ = writer.flush().await;
                *self = Writer::None;
            },
            Self::None => {},
        }
    }
}

impl SonarRelog {
    /// Output parameter will be treated as a directory unless it
    /// ends with ".bin.log" or ".ron.log"
    pub(crate) async fn init(mut output: PathBuf) -> SolarResult<Self> {
        let session_date = Utc::now().date_naive(); 
        let filename = output.file_name().and_then(|s| s.to_str());
        let format = RelogFormat::from_filename(filename);
        if filename.is_some_and(|s| s.starts_with('*')) {
            output = output.parent().expect("exists").to_path_buf();
        }
        
        let inner = Self::new_inner(&output, format, session_date).await?;
        let inner = Mutex::new(inner);
        
        Ok(Self { output, format, inner })
    }
    
    async fn new_inner(output: &Path, format: RelogFormat, session_date: NaiveDate) -> SolarResult<InnerMut> {
        let filepath = if output.ends_with(".log") {
            output.to_path_buf()
        } else {
            let ext = format.ext();
            let filename = format!("solar_sonar_{}.{ext}", session_date.format("%Y-%m-%d"));
            output.join(filename)
        };
        
        let writer = Writer::new(&filepath, format).await?;
        
        let inner = InnerMut {
            session_date,
            filepath,
            writer,
        };
        
        Ok(inner)
    }
}

impl SonarListener for SonarRelog {
    async fn on_cancel(&self, _run: &Running) {
        self.inner.lock().expect("lock").writer.close().await;
    }

    async fn on_event(&self, _run: &Running, event: &DataEvent) -> SolarResult<()> {
        let today = Utc::now().date_naive();
        if today != self.inner.lock().expect("read").session_date {
            *self.inner.lock().expect("mut") = Self::new_inner(&self.output, self.format, today).await?;
        }
        
        self.inner.lock().expect("mut").writer.write(event).await
    }
}
