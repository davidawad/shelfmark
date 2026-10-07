//! R19 over the real wiring: the gRPC transport (U11) serving the same
//! service the binary runs returns the binary's envelope byte for byte.

use std::sync::Arc;

use serde_json::json;
use shelfmark_core::{Request, Verb};
use shelfmark_grpc::{serve_with_incoming, Client};
use tokio::sync::oneshot;

use crate::common::Bench;

#[tokio::test(flavor = "multi_thread")]
async fn grpc_and_cli_return_the_same_bytes() {
    let b = Bench::new("grpc");
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = format!("http://{}", listener.local_addr().unwrap());
    let service = Arc::new(shelfmark_cli::wiring::open(&b.store));
    let (stop, rx) = oneshot::channel::<()>();
    let server = tokio::spawn(async move {
        serve_with_incoming(listener, service, async {
            let _ = rx.await;
        })
        .await
        .unwrap();
    });
    let mut client = Client::connect(addr).await.unwrap();

    // Place the Work first, so both classify calls below are memo hits
    // over the same store.
    let isbn = ["classify", "--identifier", "isbn:9780262033848"];
    b.ok(&isbn);
    let cases: Vec<(Vec<&str>, Request)> = vec![
        (
            isbn.to_vec(),
            Request::new(
                Verb::Classify,
                json!({"identifiers": ["isbn:9780262033848"]}),
            ),
        ),
        (
            vec!["classify", "--text", "x", "--scheme", "scheme:lcc@2024"],
            Request::new(
                Verb::Classify,
                json!({"text": "x", "scheme": "scheme:lcc@2024"}),
            ),
        ),
        (
            vec!["scheme", "browse", "QA"],
            Request::new(Verb::SchemeBrowse, json!({"code": "QA"})),
        ),
        (
            vec!["list-flagged"],
            Request::new(Verb::ListFlagged, json!({})),
        ),
    ];
    for (argv, request) in cases {
        let (cli, _) = b.raw(&argv);
        let over_grpc = client.call(&request).await.unwrap();
        assert_eq!(
            format!("{}\n", serde_json::to_string(&over_grpc).unwrap()),
            cli,
            "{argv:?}"
        );
    }
    stop.send(()).unwrap();
    server.await.unwrap();
}
