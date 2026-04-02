use crate::*;

pub(crate) struct SonarBinaryLog {
    output_dir: PathBuf,
    inner: RefCell<InnerMut>,
}

struct InnerMut {
    session_date: NaiveDate,
    filepath: PathBuf,
    writer: BufWriter<File>,
}

impl SonarBinaryLog {
    pub(crate) fn new(output_dir: PathBuf) -> SolarResult<Self> {
        let session_date = Utc::now().date_naive(); 
        let inner = Self::new_inner(&output_dir, session_date)?;
        let inner = RefCell::new(inner);
        
        Ok(Self { output_dir, inner })
    }
    
    fn new_inner(dir: &Path, session_date: NaiveDate) -> SolarResult<InnerMut> {
        let filename = format!("solar_sonar_{}.binlog", session_date.format("%Y-%m-%d"));
        let filepath = dir.join(filename);
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

impl SonarListener for SonarBinaryLog {
    async fn on_cancel(&self, _run: &Running) { /* file closed on drop */ }

    async fn on_event(&self, _run: &Running, event: &DataEvent) -> SolarResult<()> {
        let today = Utc::now().date_naive();
        if today != self.inner.borrow().session_date {
            self.inner.replace(Self::new_inner(&self.output_dir, today)?);
        }
        
        self.inner.borrow_mut().writer.write_all(bitcode::encode(event).as_slice())
            .map_err(|e| SolarError::write(e, &self.output_dir))?; 
        
        Ok(())
    }
}
