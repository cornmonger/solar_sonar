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

#[derive(Debug)]
pub(crate) struct SonarRelog {
    output: PathBuf,
    format: RelogFormat,
    inner: Mutex<InnerMut>,
}

#[derive(Debug)]
struct InnerMut {
    session_date: NaiveDate,
    filepath: PathBuf,
    writer: BufWriter<File>,
}

impl SonarRelog {
    /// Output parameter will be treated as a directory unless it
    /// ends with ".binlog".
    pub(crate) fn init(mut output: PathBuf) -> SolarResult<Self> {
        let session_date = Utc::now().date_naive(); 
        let filename = output.file_name().and_then(|s| s.to_str());
        let format = RelogFormat::from_filename(filename);
        if filename.is_some_and(|s| s.starts_with('*')) {
            output = output.parent().expect("exists").to_path_buf();
        }
        
        let inner = Self::new_inner(&output, format, session_date)?;
        let inner = Mutex::new(inner);
        
        Ok(Self { output, format, inner })
    }
    
    fn new_inner(output: &Path, format: RelogFormat, session_date: NaiveDate) -> SolarResult<InnerMut> {
        let filepath = if output.ends_with(".log") {
            output.to_path_buf()
        } else {
            let ext = format.ext();
            let filename = format!("solar_sonar_{}.{ext}", session_date.format("%Y-%m-%d"));
            output.join(filename)
        };
        
        let mut fsopts = fs::OpenOptions::new();
        fsopts.append(true);
        let file = fsopts.open(&filepath)
            .map_err(|e| SolarError::write(e, &filepath))?;
        let writer = BufWriter::new(file);
        
        let inner = InnerMut {
            session_date,
            filepath,
            writer,
        };
        
        Ok(inner)
    }
}

impl SonarListener for SonarRelog {
    async fn on_cancel(&self, _run: &Running) { /* file closed on drop */ }

    async fn on_event(&self, _run: &Running, event: &DataEvent) -> SolarResult<()> {
        let today = Utc::now().date_naive();
        if today != self.inner.lock().expect("read").session_date {
            *self.inner.lock().expect("mut") = Self::new_inner(&self.output, self.format, today)?;
        }
        
        self.inner.lock().expect("mut").writer.write_all(bitcode::encode(event).as_slice())
            .map_err(|e| SolarError::write(e, &self.inner.lock().expect("lock").filepath))?; 
        
        Ok(())
    }
}
