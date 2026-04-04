use crate::*;

pub(crate) struct TlsClient {
    cancel: r::tokio::CancellationToken,
    event_tx: r::tokio::mpsc::UnboundedSender<DataEvent>,
    stream: r::tls::client::TlsStream<r::tokio::TcpStream>,
}

pub(crate) struct TlsClientOptions {
    pub(crate) ip: IpAddr,
    pub(crate) port: u16,
    pub(crate) cert_files: CertFiles,
}

pub(crate) struct TlsClientHandle {
    cancel: r::tokio::CancellationToken,
    event_rx: r::tokio::mpsc::UnboundedReceiver<DataEvent>,
    handle: Option<r::tokio::JoinHandle<SolarResult<()>>>,
}

impl TlsClientHandle {
    pub(crate) async fn close(&mut self, timeout: Duration) {
        if let Some(handle) = self.handle.take() && !handle.is_finished() {
            if !self.cancel.is_cancelled() {
                self.cancel.cancel();
            }
            
            let abort_handle = handle.abort_handle();
            let complete = r::tokio::timeout(timeout, async {
                tokio::join!(handle)
            }).await;
            
            if complete.is_err() {
                abort_handle.abort();
            }
        }
    }
    
    pub(crate) async fn recv(&mut self) -> Option<Vec<DataEvent>> {
        let mut vec = Vec::new();
        match self.event_rx.recv_many(&mut vec, 8).await {
            0 => None,
            _ => Some(vec)
        }
    }
}

impl TlsClient {
    pub(crate) async fn start(_run: &Running, io: &SonarIO, opts: TlsClientOptions) -> SolarResult<TlsClientHandle> {
        let root_cert = r::tls::CertificateDer::from_pem_file(opts.cert_files.authority_public)
            .map_err(|e| SolarError::msg(format!("Failed to load TLS authority public key: {e}")))?;
        let mut root_store = r::tls::RootCertStore::empty();
        root_store.add(root_cert)
            .map_err(|e| SolarError::msg(format!("Failed to load TLS authority public key: {e}")))?;
        
        let config = rustls::ClientConfig::builder()
            .with_root_certificates(root_store)
            .with_no_client_auth();
        
        let connector = r::tls::TlsConnector::from(Arc::new(config));
        let addr = net::SocketAddr::new(opts.ip, opts.port);
        
        let domain = r::tls::ServerName::try_from("127.0.0.1".to_string())
            .map_err(|e| SolarError::msg(format!("Failed to parse TLS client name: {e}")))?;
        
        let (event_tx, event_rx) = r::tokio::mpsc::unbounded_channel();
        let cancel = r::tokio::CancellationToken::new();
        let cancel_stream = cancel.clone();
        
        let to = addr.to_string();
        let iotx = io.new_transmitter();
        
        let handle = Some(tokio::spawn(async move {
            iotx.send(DataEvent::Connecting { to: to.clone() }).unwrap();
            
            let stream = r::tokio::TcpStream::connect(&addr).await
                .map_err(|e| {
                    let _ = iotx.send(DataEvent::Connect { to: to.clone(), success: false });
                    SolarError::msg(format!("Failed to connect to TLS client: {e}"))
                })?;
            let stream = connector.connect(domain, stream).await
                .map_err(|e| {
                    let _ = iotx.send(DataEvent::Connect { to: to.clone(), success: false });
                    SolarError::msg(format!("Failed to connect to TLS client: {e}"))
                })?;
            
            iotx.send(DataEvent::Connect { to, success: true }).unwrap();
            
            let client = Self {
                cancel: cancel_stream,
                event_tx,
                stream
            };
                
            client.run().await
        }));
        
        let tls_handle = TlsClientHandle {
            cancel,
            event_rx,
            handle,
        };
        
        Ok(tls_handle)
    }
    
    async fn close_frame(
        framed_read: &mut r::tokio::FramedRead<r::tokio::ReadHalf<r::tls::client::TlsStream<r::tokio::TcpStream>>, BitcodeCodec<ServerToClient>>,
        framed_write: &mut r::tokio::FramedWrite<r::tokio::WriteHalf<r::tls::client::TlsStream<r::tokio::TcpStream>>, BitcodeCodec<ClientToServer>>
    ) {
        if framed_write.send(&ClientToServer::Close).await.is_err() {
            return;
        }
        
        let _ = r::tokio::timeout(Duration::from_secs(2), async {
            loop {
                tokio::select! {
                    server_msg = framed_read.next() => match server_msg {
                        Some(Ok(ServerToClient::Close)) => break,
                        Some(Ok(_)) => continue,
                        Some(Err(_)) => break,
                        None => break,
                    },
                }
            }
        }).await;
    }
    
    async fn run(self) -> SolarResult<()> {
        let (read_stream, write_stream) = tokio::io::split(self.stream);
        let mut framed_read = r::tokio::FramedRead::new(read_stream, BitcodeCodec::<ServerToClient>::new());
        let mut framed_write = r::tokio::FramedWrite::new(write_stream, BitcodeCodec::<ClientToServer>::new());
        
        loop {
            let result = tokio::select! {
                _ = self.cancel.cancelled() => {
                    Self::close_frame(&mut framed_read, &mut framed_write).await;
                    break;
                },
                event = framed_read.next() => match event {
                    Some(Ok(event)) => {
                        match event {
                            ServerToClient::Event(event) => {
                                match self.event_tx.send(event) {
                                    Ok(_) => Ok(()),
                                    Err(_) => break,
                                }
                            },
                            ServerToClient::Close => {
                                let _ = framed_write.send(&ClientToServer::Close).await;
                                break;
                            },
                        }
                    },
                    Some(Err(e)) => Err(e),
                    None => break,
                },
            };
            
            if result.is_err() { break; }
        }
        
        let _ = framed_write.close().await;
        Ok(())
    }
}

