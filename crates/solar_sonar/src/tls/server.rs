use crate::*;

pub(crate) struct TlsServer {
    cancel: r::tokio::CancellationToken,
    event_rx: SonarBroadcastRx,
    listener: r::tokio::TcpListener,
    acceptor: r::tls::TlsAcceptor,
}

pub(crate) struct TlsServerOptions {
    pub(crate) ip: IpAddr,
    pub(crate) port: u16,
    pub(crate) cert_files: CertFiles,
    pub(crate) event_rx: SonarBroadcastRx,
}

pub(crate) struct TlsServerHandle {
    cancel: r::tokio::CancellationToken,
    handle: Option<r::tokio::JoinHandle<SolarResult<()>>>,
}

impl TlsServerHandle {
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
}

impl TlsServer {
    pub(crate) async fn start(_run: &Running, opts: TlsServerOptions) -> SolarResult<TlsServerHandle> {
        let addr = net::SocketAddr::new(opts.ip, opts.port);
        let cert = r::tls::CertificateDer::from_pem_file(opts.cert_files.entity_public)
            .map_err(|e| SolarError::msg(format!("Failed to load TLS server public key: {e}")))?;
        let key = r::tls::PrivateKeyDer::from_pem_file(opts.cert_files.entity_private)
            .map_err(|e| SolarError::msg(format!("Failed to load TLS server private key: {e}")))?;
        
        let config = rustls::ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(vec![cert], key)
            .map_err(|e| SolarError::msg(format!("Failed to configure TLS server: {e}")))?;
        
        let cancel = r::tokio::CancellationToken::new();
        let acceptor = r::tls::TlsAcceptor::from(Arc::new(config));
        let listener = r::tokio::TcpListener::bind(&addr).await
            .map_err(|e| SolarError::msg(format!("Failed to bind to address: {e}")))?;
        
        let server = Self {
            event_rx: opts.event_rx,
            listener,
            acceptor,
            cancel: cancel.clone(),
        };
            
        let handle = Some(tokio::spawn(server.run()));
        let tls_handle = TlsServerHandle { handle, cancel };
        Ok(tls_handle)
    }
    
    async fn run(self) -> SolarResult<()> {
        let mut client_handles = Vec::new();
        let cancel_token = r::tokio::CancellationToken::new();
        
        loop {
            tokio::select! {
                _ = self.cancel.cancelled() => break,
                conn = self.listener.accept() => {
                    match conn {
                        Ok((stream, _addr)) => {
                            let stream = self.acceptor.accept(stream).await.unwrap();
                            let handle = tokio::spawn(Self::loop_client(stream, self.event_rx.resubscribe(), cancel_token.clone()));
                            client_handles.push(handle);
                        },
                        Err(_e) => {},
                    }
                },
            }
        }
        
        join_all(client_handles).await;
        Ok(())
    }
    
    async fn close_frame(
        framed_read: &mut r::tokio::FramedRead<r::tokio::ReadHalf<r::tls::server::TlsStream<r::tokio::TcpStream>>, BitcodeCodec<ClientToServer>>,
        framed_write: &mut r::tokio::FramedWrite<r::tokio::WriteHalf<r::tls::server::TlsStream<r::tokio::TcpStream>>, BitcodeCodec<ServerToClient>>
    ) {
        if framed_write.send(&ServerToClient::Close).await.is_err() {
            return;
        }
        
        let _ = r::tokio::timeout(Duration::from_secs(2), async {
            loop {
                tokio::select! {
                    server_msg = framed_read.next() => match server_msg {
                        Some(Ok(ClientToServer::Close)) => break,
                        Some(Err(_)) => break,
                        None => break,
                    },
                }
            }
        }).await;
    }
    
    async fn loop_client(
        stream: r::tls::server::TlsStream<r::tokio::TcpStream>,
        mut event_rx: SonarBroadcastRx,
        cancel_token: r::tokio::CancellationToken
    ) {
        let (read_stream, write_stream) = tokio::io::split(stream);
        let mut framed_read = r::tokio::FramedRead::new(read_stream, BitcodeCodec::<ClientToServer>::new());
        let mut framed_write = r::tokio::FramedWrite::new(write_stream, BitcodeCodec::<ServerToClient>::new());
        
        loop {
            let result = tokio::select! {
                _ = cancel_token.cancelled() => {
                    Self::close_frame(&mut framed_read, &mut framed_write).await;
                    break;
                },
                client_msg = framed_read.next() => match client_msg {
                    Some(Ok(ClientToServer::Close)) => {
                        let _ = framed_write.send(&ServerToClient::Close).await;
                        break;
                    },
                    Some(Err(_)) => break,
                    None => break,
                },
                event = event_rx.recv() => match event {
                    Ok(event) => {
                        framed_write.send(&ServerToClient::Event(event)).await
                    },
                    Err(_) => {
                        Self::close_frame(&mut framed_read, &mut framed_write).await;
                        break;
                    },
                },
            };
            
            if result.is_err() { break; }
        }
        
        let _ = framed_write.close().await;
    }
}

