use crate::*;

#[derive(Debug)]
pub(crate) struct SonarBinLog {
    output: PathBuf,
    inner: Mutex<InnerMut>,
}

#[derive(Debug)]
struct InnerMut {
    session_date: NaiveDate,
    filepath: PathBuf,
    writer: BufWriter<File>,
}

impl SonarBinLog {
    /// Output parameter will be treated as a directory unless it
    /// ends with ".binlog".
    pub(crate) fn init(output: PathBuf) -> SolarResult<Self> {
        let session_date = Utc::now().date_naive(); 
        let inner = Self::new_inner(&output, session_date)?;
        let inner = Mutex::new(inner);
        
        Ok(Self { output, inner })
    }
    
    fn new_inner(dir: &Path, session_date: NaiveDate) -> SolarResult<InnerMut> {
        let filepath = if dir.ends_with(".binlog") {
            dir.to_path_buf()
        } else {
            let filename = format!("solar_sonar_{}.binlog", session_date.format("%Y-%m-%d"));
            dir.join(filename)
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

impl SonarListener for SonarBinLog {
    async fn on_cancel(&self, _run: &Running) { /* file closed on drop */ }

    async fn on_event(&self, _run: &Running, event: &DataEvent) -> SolarResult<()> {
        let today = Utc::now().date_naive();
        if today != self.inner.lock().expect("read").session_date {
            *self.inner.lock().expect("mut") = Self::new_inner(&self.output, today)?;
        }
        
        self.inner.lock().expect("mut").writer.write_all(bitcode::encode(event).as_slice())
            .map_err(|e| SolarError::write(e, &self.output))?; 
        
        Ok(())
    }
}
