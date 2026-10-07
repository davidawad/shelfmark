//! In-process server over a fake Service: the gRPC envelope is the CLI
//! envelope byte for byte, and a streaming batch keeps order and
//! per-line errors.

use std::sync::Arc;
use std::time::Duration;

use serde_json::json;
use shelfmark_core::envelope::{BatchLine, Mode, Request, Verb};
use shelfmark_core::{Envelope, Error, ErrorCode, Service};
use shelfmark_grpc::pb::shelfmark_client::ShelfmarkClient;
use shelfmark_grpc::{pb, serve_with_incoming, BatchItem, Client};
use tokio::sync::oneshot;
use tokio_stream::StreamExt;

/// Echoes the request back as data; `{"fail": true}` fails with every
/// optional error field set; `{"sleep_ms": n}` waits first, so a reordered
/// batch would show.
struct Fake;

impl Service for Fake {
    fn call(&self, r: Request) -> Envelope {
        if let Some(ms) = r.args.get("sleep_ms").and_then(|v| v.as_u64()) {
            std::thread::sleep(Duration::from_millis(ms));
        }
        if r.args.get("fail") == Some(&json!(true)) {
            return Envelope::err(
                Error::new(ErrorCode::SchemeNotCompiled, "scheme:lcc is not compiled")
                    .next("shelfmark scheme compile data/schemes/lcc/lcc.json")
                    .details(json!({"scheme": "scheme:lcc", "score": 0.1 + 0.2})),
            );
        }
        Envelope::ok(json!({
            "verb": r.verb,
            "args": r.args,
            "mode": r.mode,
            "expected_revision": r.expected_revision,
            "floats": [0.1, 1e-300, 1.7976931348623157e308, -0.0, 1.2345678901234567],
            "text": "Gödel, Escher, Bach \u{1F4DA} \"quoted\"\n",
        }))
    }
}

struct Server {
    addr: String,
    stop: Option<oneshot::Sender<()>>,
    task: tokio::task::JoinHandle<()>,
}

impl Server {
    async fn start() -> Self {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = format!("http://{}", listener.local_addr().unwrap());
        let (stop, rx) = oneshot::channel();
        let task = tokio::spawn(async move {
            serve_with_incoming(listener, Arc::new(Fake), async {
                let _ = rx.await;
            })
            .await
            .unwrap();
        });
        Server {
            addr,
            stop: Some(stop),
            task,
        }
    }

    async fn shutdown(mut self) {
        self.stop.take().unwrap().send(()).unwrap();
        self.task.await.unwrap();
    }
}

fn requests() -> Vec<Request> {
    let mut out: Vec<Request> = Verb::ALL
        .iter()
        .map(|&v| Request::new(v, json!({"work": "work:abc", "k": 5})))
        .collect();
    out.push(Request::new(Verb::Status, json!({})));
    out.push(Request::new(Verb::Classify, json!({"fail": true})));
    out.push(Request::new(
        Verb::Explain,
        json!({"nested": {"z": [1, {"b": null, "a": 2.5}], "a": "x"}, "n": -12345678901234i64}),
    ));
    out.push(Request {
        verb: Verb::Confirm,
        args: json!({"placement": "placement:0f0f", "class": "class:lcc/QA76"}),
        mode: Mode::Commit,
        expected_revision: Some("rev-41".into()),
    });
    out
}

fn bytes(env: &Envelope) -> String {
    serde_json::to_string(env).unwrap()
}

#[tokio::test(flavor = "multi_thread")]
async fn call_envelope_is_byte_equal_to_service_call() {
    let server = Server::start().await;
    let mut client = Client::connect(server.addr.clone()).await.unwrap();
    for req in requests() {
        let direct = bytes(&Fake.call(req.clone()));
        let over_grpc = bytes(&client.call(&req).await.unwrap());
        assert_eq!(over_grpc, direct, "verb {}", req.verb.as_str());
    }
    server.shutdown().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn error_envelope_keeps_code_next_command_and_details() {
    let server = Server::start().await;
    let mut client = Client::connect(server.addr.clone()).await.unwrap();
    let env = client
        .call(&Request::new(Verb::Classify, json!({"fail": true})))
        .await
        .unwrap();
    match env {
        Envelope::Err { error, .. } => {
            assert_eq!(error.code, ErrorCode::SchemeNotCompiled);
            assert_eq!(
                error.next_command.as_deref(),
                Some("shelfmark scheme compile data/schemes/lcc/lcc.json")
            );
            assert_eq!(error.details.unwrap()["score"], json!(0.1 + 0.2));
        }
        Envelope::Ok { .. } => panic!("expected an error envelope"),
    }
    server.shutdown().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn batch_preserves_order_and_per_line_errors() {
    let server = Server::start().await;
    let mut client = Client::connect(server.addr.clone()).await.unwrap();
    // Earlier items sleep longer: any concurrency that reorders would show.
    let items: Vec<BatchItem> = (0..40u64)
        .map(|i| BatchItem {
            input: format!("item:{i:04}"),
            request: Request::new(
                Verb::Classify,
                json!({"i": i, "fail": i % 7 == 3, "sleep_ms": (40 - i) % 5}),
            ),
        })
        .collect();
    let lines = client.batch_collect(items.clone()).await.unwrap();
    assert_eq!(lines.len(), items.len());
    for (item, line) in items.iter().zip(&lines) {
        let expected = BatchLine {
            input: item.input.clone(),
            envelope: Fake.call(item.request.clone()),
        };
        // The NDJSON line the CLI would print, byte for byte.
        assert_eq!(
            serde_json::to_string(line).unwrap(),
            serde_json::to_string(&expected).unwrap()
        );
    }
    let failed: Vec<_> = lines
        .iter()
        .filter(|l| !l.envelope.is_ok())
        .map(|l| l.input.as_str())
        .collect();
    assert_eq!(
        failed,
        [
            "item:0003",
            "item:0010",
            "item:0017",
            "item:0024",
            "item:0031",
            "item:0038"
        ]
    );
    server.shutdown().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn bad_batch_items_become_error_lines_and_the_batch_continues() {
    let server = Server::start().await;
    let mut raw = ShelfmarkClient::connect(server.addr.clone()).await.unwrap();
    let ok = |i: &str| pb::BatchItem {
        input: i.into(),
        request: Some(pb::Request {
            verb: pb::Verb::Status as i32,
            ..Default::default()
        }),
    };
    let items = vec![
        ok("a"),
        pb::BatchItem {
            input: "no-request".into(),
            request: None,
        },
        pb::BatchItem {
            input: "bad-json".into(),
            request: Some(pb::Request {
                verb: pb::Verb::Classify as i32,
                args_json: "{not json".into(),
                ..Default::default()
            }),
        },
        pb::BatchItem {
            input: "no-verb".into(),
            request: Some(pb::Request::default()),
        },
        ok("z"),
    ];
    let lines: Vec<pb::BatchLine> = raw
        .batch(tokio_stream::iter(items))
        .await
        .unwrap()
        .into_inner()
        .map(|l| l.unwrap())
        .collect()
        .await;
    let summary: Vec<(String, bool, i32)> = lines
        .into_iter()
        .map(|l| {
            let env = l.envelope.unwrap();
            (l.input, env.ok, env.error.map_or(0, |e| e.code))
        })
        .collect();
    let invalid = pb::ErrorCode::InvalidRequest as i32;
    assert_eq!(
        summary,
        [
            ("a".to_string(), true, 0),
            ("no-request".to_string(), false, invalid),
            ("bad-json".to_string(), false, invalid),
            ("no-verb".to_string(), false, invalid),
            ("z".to_string(), true, 0),
        ]
    );
    server.shutdown().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn unspecified_verb_call_is_an_invalid_request_envelope() {
    let server = Server::start().await;
    let mut raw = ShelfmarkClient::connect(server.addr.clone()).await.unwrap();
    let env = raw.call(pb::Request::default()).await.unwrap().into_inner();
    assert!(!env.ok);
    let err = env.error.unwrap();
    assert_eq!(err.code, pb::ErrorCode::InvalidRequest as i32);
    assert_eq!(err.next_command.as_deref(), Some("shelfmark describe"));
    server.shutdown().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn empty_batch_yields_no_lines() {
    let server = Server::start().await;
    let mut client = Client::connect(server.addr.clone()).await.unwrap();
    let lines = client.batch_collect(Vec::new()).await.unwrap();
    assert!(lines.is_empty());
    server.shutdown().await;
}
