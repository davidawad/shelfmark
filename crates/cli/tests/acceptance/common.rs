//! Shared helpers: a fresh store, the `shelfmark` binary with JSON output,
//! and fixture files (including a minimal PDF) written into the store's
//! working directory.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

use serde_json::Value;

/// The fixtures directory of the workspace.
pub fn fixture(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures")
        .join(rel)
}

/// A scratch directory holding a store (`store/`) and input files.
pub struct Bench {
    pub dir: PathBuf,
    pub store: PathBuf,
}

impl Bench {
    /// A new, empty bench with the fixture identifier table installed in
    /// the store (plan PD5: `<store>/tables/`).
    pub fn new(tag: &str) -> Bench {
        static N: AtomicUsize = AtomicUsize::new(0);
        let dir = std::env::temp_dir().join(format!(
            "shelfmark-acceptance-{tag}-{}-{}",
            std::process::id(),
            N.fetch_add(1, Ordering::SeqCst)
        ));
        let _ = std::fs::remove_dir_all(&dir);
        let store = dir.join("store");
        let tables = store.join("tables");
        std::fs::create_dir_all(&tables).unwrap();
        std::fs::copy(
            fixture("tables/identifiers.jsonl"),
            tables.join("identifiers.jsonl"),
        )
        .unwrap();
        Bench { dir, store }
    }

    /// Run `shelfmark --store <store> <args>`; returns the envelope and
    /// the exit code. Output is JSON because stdout is not a terminal.
    pub fn run(&self, args: &[&str]) -> (Value, i32) {
        let (text, code) = self.raw(args);
        let envelope: Value = serde_json::from_str(&text)
            .unwrap_or_else(|e| panic!("{args:?}: not one JSON envelope ({e}): {text}"));
        (envelope, code)
    }

    /// The exact stdout of a run, and its exit code.
    pub fn raw(&self, args: &[&str]) -> (String, i32) {
        let out = Command::new(env!("CARGO_BIN_EXE_shelfmark"))
            .arg("--store")
            .arg(&self.store)
            .args(args)
            .output()
            .expect("shelfmark runs");
        let text = String::from_utf8(out.stdout).expect("UTF-8 output");
        (text, out.status.code().expect("exit code"))
    }

    /// The `data` of a successful call; panics with the error otherwise.
    pub fn ok(&self, args: &[&str]) -> Value {
        let (env, code) = self.run(args);
        assert_eq!(env["ok"], true, "{args:?} failed: {env}");
        assert_eq!(code, 0, "{args:?}");
        env["data"].clone()
    }

    /// Preview a mutation, then commit it at the previewed revision.
    pub fn commit(&self, args: &[&str]) -> Value {
        let preview = self.ok(args);
        let rev = preview["revision"]
            .as_str()
            .expect("preview has a revision");
        let mut full = args.to_vec();
        full.extend(["--commit", "--expected-revision", rev]);
        self.ok(&full)
    }

    /// Write `bytes` to `name` in the bench; returns its path as a string.
    pub fn file(&self, name: &str, bytes: impl AsRef<[u8]>) -> String {
        let path = self.dir.join(name);
        std::fs::write(&path, bytes).unwrap();
        path.to_str().unwrap().to_string()
    }
}

/// Codes of a placement's trace candidates.
pub fn candidate_codes(placement: &Value) -> Vec<String> {
    placement["trace"]["candidates"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| {
            let id = c["class"].as_str().unwrap();
            id.rsplit(':').next().unwrap().to_string()
        })
        .collect()
}

/// A one-page PDF showing `lines` in Helvetica, with a correct xref table.
pub fn pdf(lines: &[&str]) -> Vec<u8> {
    let mut content = String::from("BT /F1 12 Tf 72 720 Td 14 TL\n");
    for line in lines {
        let escaped = line
            .replace('\\', "\\\\")
            .replace('(', "\\(")
            .replace(')', "\\)");
        content.push_str(&format!("({escaped}) Tj T*\n"));
    }
    content.push_str("ET");
    let objects = [
        "<< /Type /Catalog /Pages 2 0 R >>".to_string(),
        "<< /Type /Pages /Kids [3 0 R] /Count 1 >>".to_string(),
        "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] \
         /Resources << /Font << /F1 5 0 R >> >> /Contents 4 0 R >>"
            .to_string(),
        format!(
            "<< /Length {} >>\nstream\n{content}\nendstream",
            content.len()
        ),
        "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica \
         /Encoding /WinAnsiEncoding >>"
            .to_string(),
    ];
    let mut out = String::from("%PDF-1.4\n");
    let mut offsets = Vec::new();
    for (i, body) in objects.iter().enumerate() {
        offsets.push(out.len());
        out.push_str(&format!("{} 0 obj\n{body}\nendobj\n", i + 1));
    }
    let xref = out.len();
    out.push_str(&format!(
        "xref\n0 {}\n0000000000 65535 f \n",
        objects.len() + 1
    ));
    for off in offsets {
        out.push_str(&format!("{off:010} 00000 n \n"));
    }
    out.push_str(&format!(
        "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n",
        objects.len() + 1
    ));
    out.into_bytes()
}
