//! The tonic server: every RPC delegates to an `Arc<dyn Service>`.

use std::net::SocketAddr;
use std::sync::Arc;

use shelfmark_core::{Envelope, Error, Service};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::{Stream, StreamExt};
use tonic::{Request as TonicRequest, Response, Status, Streaming};

use crate::convert::{envelope_to_pb, request_from_pb};
use crate::pb;
use crate::pb::shelfmark_server::{Shelfmark, ShelfmarkServer};

/// Batch lines buffered ahead of a slow reader.
const BATCH_BUFFER: usize = 16;

/// The `shelfmark.v1.Shelfmark` service over any core [`Service`].
#[derive(Clone)]
pub struct GrpcService {
    service: Arc<dyn Service>,
}

impl GrpcService {
    pub fn new(service: Arc<dyn Service>) -> Self {
        GrpcService { service }
    }

    /// The tonic service, ready for `Server::builder().add_service(..)`.
    pub fn into_server(self) -> ShelfmarkServer<Self> {
        ShelfmarkServer::new(self)
    }
}

/// Decodes the request and runs it on a blocking thread: `Service::call`
/// is synchronous and may touch the disk. Request errors become error
/// envelopes, exactly as the CLI reports them.
async fn dispatch(service: &Arc<dyn Service>, request: Option<pb::Request>) -> Envelope {
    let request = match request.map(request_from_pb) {
        Some(Ok(r)) => r,
        Some(Err(e)) => return Envelope::err(e),
        None => {
            return Envelope::err(Error::new(
                shelfmark_core::ErrorCode::InvalidRequest,
                "batch item has no request",
            ))
        }
    };
    let service = Arc::clone(service);
    tokio::task::spawn_blocking(move || service.call(request))
        .await
        .unwrap_or_else(|e| Envelope::err(Error::internal(format!("service call failed: {e}"))))
}

type BatchStream = std::pin::Pin<Box<dyn Stream<Item = Result<pb::BatchLine, Status>> + Send>>;

#[tonic::async_trait]
impl Shelfmark for GrpcService {
    async fn call(
        &self,
        request: TonicRequest<pb::Request>,
    ) -> Result<Response<pb::Envelope>, Status> {
        let env = dispatch(&self.service, Some(request.into_inner())).await;
        Ok(Response::new(envelope_to_pb(&env)))
    }

    type BatchStream = BatchStream;

    /// One line per item, in arrival order. A bad item yields an error
    /// line and the batch carries on; only a broken stream ends it.
    async fn batch(
        &self,
        request: TonicRequest<Streaming<pb::BatchItem>>,
    ) -> Result<Response<Self::BatchStream>, Status> {
        let mut items = request.into_inner();
        let service = Arc::clone(&self.service);
        let (tx, rx) = mpsc::channel(BATCH_BUFFER);
        tokio::spawn(async move {
            while let Some(item) = items.next().await {
                let line = match item {
                    Ok(item) => {
                        let env = dispatch(&service, item.request).await;
                        Ok(pb::BatchLine {
                            input: item.input,
                            envelope: Some(envelope_to_pb(&env)),
                        })
                    }
                    Err(status) => Err(status),
                };
                let stop = line.is_err();
                if tx.send(line).await.is_err() || stop {
                    break;
                }
            }
        });
        Ok(Response::new(Box::pin(ReceiverStream::new(rx))))
    }
}

/// Serves `service` on `addr` until the process stops.
pub async fn serve(
    addr: SocketAddr,
    service: Arc<dyn Service>,
) -> Result<(), tonic::transport::Error> {
    tonic::transport::Server::builder()
        .add_service(GrpcService::new(service).into_server())
        .serve(addr)
        .await
}

/// Serves on an already-bound listener (tests bind port 0) until
/// `shutdown` resolves.
pub async fn serve_with_incoming(
    listener: tokio::net::TcpListener,
    service: Arc<dyn Service>,
    shutdown: impl std::future::Future<Output = ()> + Send,
) -> Result<(), tonic::transport::Error> {
    tonic::transport::Server::builder()
        .add_service(GrpcService::new(service).into_server())
        .serve_with_incoming_shutdown(
            tokio_stream::wrappers::TcpListenerStream::new(listener),
            shutdown,
        )
        .await
}
