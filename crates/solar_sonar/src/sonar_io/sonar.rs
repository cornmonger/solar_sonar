use crate::*;

pub type SonarBroadcastTx = r::tokio::broadcast::Sender<DataEvent>;
pub type SonarBroadcastRx = r::tokio::broadcast::Receiver<DataEvent>;

const EVENT_IO_BUFFER_MAX: usize = 64;

#[derive(Debug)]
pub struct SonarIO {
    pub(crate) cancel: r::tokio::CancellationToken,
    pub(crate) tx: SonarBroadcastTx,
    pub(crate) rx: SonarBroadcastRx,
    pub(crate) audio: Option<SonarAudio>,
    pub(crate) stdio: Option<SonarStdio>,
}

#[derive(Debug, Clone)]
pub struct SonarOptions {
    pub audio: bool,
    pub stdio: bool,
}

impl SonarIO {
    pub(crate) fn init(options: SonarOptions) -> SolarResult<Self> {
        let (tx, rx) = r::tokio::broadcast::channel(EVENT_IO_BUFFER_MAX);
        let cancel = r::tokio::CancellationToken::new();
        
        let stdio = match options.stdio {
            true => Some(SonarStdio::new()),
            false => None,
        };
        
        let audio = match options.audio {
            true => Some(SonarAudio::init()?),
            false => None,
        };
        
        Ok(Self { cancel, tx, rx, stdio, audio })
    }
    
    pub(crate) fn subscribe(&self) -> SonarBroadcastRx {
        self.tx.subscribe()
    }
    
    pub(crate) fn new_transmitter(&self) -> SonarBroadcastTx {
        self.tx.clone()
    }

    pub(crate) fn broadcast(&self, event: DataEvent) -> SolarResult<()> {
        self.tx.send(event)
            .map(|_| ())
            .map_err(|e| SolarError::broadcast(e))
    }
    
    pub(crate) async fn select(&mut self, run: &Running) -> SolarResult<()> {
        tokio::select! {
            _ = self.cancel.cancelled() => {
                self.on_cancel(run).await;
                Ok(())
            },
            event = self.rx.recv() => match event {
                Ok(event) => self.on_event(run, event).await, 
                Err(_) => SolarError::err_msg("Sonar IO event queue error"),
            }
        }
    }

    async fn on_cancel(&self, run: &Running) {
        if let Some(stdio) = self.stdio.as_ref() {
            stdio.on_cancel(run).await;
        }
        if let Some(audio) = self.audio.as_ref() {
            audio.on_cancel(run).await;
        }
    }
    
    async fn on_event(&self, run: &Running, event: DataEvent) -> SolarResult<()> {
        if let Some(stdio) = self.stdio.as_ref() {
            stdio.on_event(run, &event).await?;
        }
        if let Some(audio) = self.audio.as_ref() {
            audio.on_event(run, &event).await?;
        }
        
        Ok(())
    }
    
    pub(crate) async fn close(self) {
        self.cancel.cancel();
    }
}

pub trait SonarListener {
    async fn on_cancel(&self, run: &Running);
    async fn on_event(&self, run: &Running, event: &DataEvent) -> SolarResult<()>;
}
