//! The workspace stays splittable (plan PD1): every path dependency in
//! every manifest resolves inside this repo.

use std::path::{Path, PathBuf};

fn manifests(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        let name = path.file_name().unwrap().to_string_lossy().to_string();
        if path.is_dir() && name != "target" && !name.starts_with('.') {
            manifests(&path, out);
        } else if name == "Cargo.toml" {
            out.push(path);
        }
    }
}

#[test]
fn no_path_dependency_leaves_the_workspace() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap();
    let mut found = Vec::new();
    manifests(&root, &mut found);
    assert!(found.len() >= 2, "workspace and core manifests");
    for manifest in found {
        let text = std::fs::read_to_string(&manifest).unwrap();
        for rest in text.split("path = \"").skip(1) {
            let rel = &rest[..rest.find('"').unwrap()];
            let target = manifest.parent().unwrap().join(rel).canonicalize().unwrap();
            assert!(
                target.starts_with(&root),
                "{} points outside the workspace: {rel}",
                manifest.display()
            );
        }
    }
}
