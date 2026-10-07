//! A typed client: core shapes in, core shapes out.
//!
//! Transport errors are `tonic::Status`, as every tonic caller expects;
//! boxing it only to quiet `result_large_err` would cost callers more.
#![allow(clippy::result_large_err)]

use shelfmark_core::envelope::{BatchLine, Request};
use shelfmark_core::Envelope;
use tokio_stream::{Stream, StreamExt};
use tonic::transport::{Channel, Endpoint};
use tonic::Status;

use crate::convert::{envelope_from_pb, request_to_pb};
use crate::pb;
use crate::pb::shelfmark_client::ShelfmarkClient;

/// One batch input: its content hash (an `item:` ID for files) and the
/// request to run for it.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchItem {
    pub input: String,
    pub request: Request,
}

#[derive(Debug, Clone)]
pub struct Client {
    inner: ShelfmarkClient<Channel>,
}

impl Client {
    /// Connects to a server, e.g. `http://127.0.0.1:50051`.
    pub async fn connect(dst: impl Into<String>) -> Result<Self, tonic::transport::Error> {
        let channel = Endpoint::from_shared(dst.into())?.connect().await?;
        Ok(Client::from_channel(channel))
    }

    pub fn from_channel(channel: Channel) -> Self {
        Client {
            inner: ShelfmarkClient::new(channel),
        }
    }

    /// `Err` only for transport failures; service errors arrive as an
    /// error [`Envelope`], as from the CLI.
    pub async fn call(&mut self, request: &Request) -> Result<Envelope, Status> {
        let env = self.inner.call(request_to_pb(request)).await?.into_inner();
        Ok(envelope_from_pb(env))
    }

    /// Streams one [`BatchLine`] per item, in the order sent.
    pub async fn batch(
        &mut self,
        items: impl IntoIterator<Item = BatchItem, IntoIter: Send + 'static>,
    ) -> Result<impl Stream<Item = Result<BatchLine, Status>>, Status> {
        let outbound = tokio_stream::iter(items.into_iter().map(|i| pb::BatchItem {
            input: i.input,
            request: Some(request_to_pb(&i.request)),
        }));
        let lines = self.inner.batch(outbound).await?.into_inner();
        Ok(lines.map(|line| {
            line.map(|l| BatchLine {
                input: l.input,
                envelope: envelope_from_pb(l.envelope.unwrap_or_default()),
            })
        }))
    }

    /// [`Client::batch`], collected.
    pub async fn batch_collect(
        &mut self,
        items: impl IntoIterator<Item = BatchItem, IntoIter: Send + 'static>,
    ) -> Result<Vec<BatchLine>, Status> {
        self.batch(items).await?.collect().await
    }
}
