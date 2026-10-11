//! L0-L3 typed IDs (R1, R2, R4, R20).
//!
//! Every ID is `<kind>:<body>`, so a string names its own kind and one
//! `resolve` verb can route it. Hash IDs carry their algorithm tag
//! (`work:nt1s256:<hex>`) so the derivation can change without breaking
//! old IDs; scheme-bound IDs carry the scheme version
//! (`class:lcc@2024:QA76.76.C65`).
//!
//! Vocabulary: the `work:` kind names a *specimen* (shelfmark's public noun
//! since 0.2; it was "Work"). The rendered prefix stays `work:` because an
//! ID's string is hashed into every placement ID and memo key, so it must
//! never change for an existing specimen. `specimen:` is accepted as an
//! input alias for `work:` and parses to the same ID.

use std::fmt;
use std::str::FromStr;

use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use sha2::{Digest as _, Sha256};
use unicode_normalization::UnicodeNormalization;

/// Input alias for the `work:` prefix (a specimen ID). Parses to the same
/// ID; IDs always render with `work:`.
pub const SPECIMEN_ALIAS: &str = "specimen";

/// The noun an ID names.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    Work,
    Item,
    Scheme,
    Class,
    Profile,
    Placement,
}

impl Kind {
    pub const ALL: [Kind; 6] = [
        Kind::Work,
        Kind::Item,
        Kind::Scheme,
        Kind::Class,
        Kind::Profile,
        Kind::Placement,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Kind::Work => "work",
            Kind::Item => "item",
            Kind::Scheme => "scheme",
            Kind::Class => "class",
            Kind::Profile => "profile",
            Kind::Placement => "placement",
        }
    }

    /// The public noun: `specimen` for [`Kind::Work`], else the prefix.
    pub fn noun(self) -> &'static str {
        match self {
            Kind::Work => SPECIMEN_ALIAS,
            other => other.as_str(),
        }
    }

    fn parse(s: &str) -> Option<Kind> {
        if s == SPECIMEN_ALIAS {
            return Some(Kind::Work);
        }
        Kind::ALL.into_iter().find(|k| k.as_str() == s)
    }
}

impl fmt::Display for Kind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Why a string is not a valid ID.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum IdError {
    #[error("`{0}` has no `<kind>:` prefix")]
    NoKind(String),
    #[error(
        "`{0}` is not a known kind (work or specimen, item, scheme, class, profile, placement)"
    )]
    UnknownKind(String),
    #[error("`{id}` is not a {kind} ID: {reason}")]
    Malformed {
        kind: Kind,
        id: String,
        reason: &'static str,
    },
}

/// Normalization v1 for Work IDs (plan PD2): Unicode NFKC (so NFD
/// accents and PDF ligatures match their composed forms), lowercase, every
/// run of non-alphanumeric characters becomes one space, trimmed.
pub fn normalize_text_v1(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut gap = false;
    for ch in text.nfkc() {
        if ch.is_alphanumeric() {
            if gap && !out.is_empty() {
                out.push(' ');
            }
            gap = false;
            out.extend(ch.to_lowercase());
        } else {
            gap = true;
        }
    }
    out
}

/// Lowercase hex SHA-256 of `bytes`.
pub fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut s = String::with_capacity(64);
    for b in digest {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

/// A self-describing content hash: algorithm tag plus 64 lowercase hex.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct HashBody {
    pub alg: String,
    pub hex: String,
}

impl HashBody {
    fn parse(kind: Kind, id: &str, body: &str, algs: &[&str]) -> Result<Self, IdError> {
        let bad = |reason| IdError::Malformed {
            kind,
            id: id.to_string(),
            reason,
        };
        let (alg, hex) = body.split_once(':').ok_or(bad("expected <alg>:<hex>"))?;
        if !algs.contains(&alg) {
            return Err(bad("unknown hash algorithm"));
        }
        if hex.len() != 64
            || !hex
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        {
            return Err(bad("digest must be 64 lowercase hex characters"));
        }
        Ok(HashBody {
            alg: alg.to_string(),
            hex: hex.to_string(),
        })
    }
}

/// `<name>@<version>`: names and versions are `[A-Za-z0-9._-]+`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Versioned {
    pub name: String,
    pub version: String,
}

fn is_token(s: &str) -> bool {
    !s.is_empty()
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'-'))
}

impl Versioned {
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
        Versioned {
            name: name.into(),
            version: version.into(),
        }
    }

    fn parse(kind: Kind, id: &str, body: &str) -> Result<Self, IdError> {
        let bad = |reason| IdError::Malformed {
            kind,
            id: id.to_string(),
            reason,
        };
        let (name, version) = body
            .split_once('@')
            .ok_or(bad("expected <name>@<version>"))?;
        if !is_token(name) || !is_token(version) {
            return Err(bad("name and version must be [A-Za-z0-9._-]+"));
        }
        Ok(Versioned::new(name, version))
    }
}

impl fmt::Display for Versioned {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}@{}", self.name, self.version)
    }
}

/// Defines a newtype ID with Display/FromStr/serde as its string form.
macro_rules! string_id {
    ($ty:ident) => {
        impl fmt::Display for $ty {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(&self.render())
            }
        }
        impl Serialize for $ty {
            fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
                s.serialize_str(&self.render())
            }
        }
        impl<'de> Deserialize<'de> for $ty {
            fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
                let s = String::deserialize(d)?;
                s.parse().map_err(de::Error::custom)
            }
        }
    };
}

fn split_kind(id: &str, want: Kind) -> Result<&str, IdError> {
    let (kind, body) = id
        .split_once(':')
        .ok_or_else(|| IdError::NoKind(id.to_string()))?;
    match Kind::parse(kind) {
        Some(k) if k == want => Ok(body),
        Some(_) => Err(IdError::Malformed {
            kind: want,
            id: id.to_string(),
            reason: "wrong kind prefix",
        }),
        None => Err(IdError::UnknownKind(kind.to_string())),
    }
}

/// Defines a content-hash ID kind with its accepted algorithm tags.
macro_rules! hash_id {
    ($ty:ident, $kind:expr, [$($alg:literal),+]) => {
        #[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub struct $ty(pub HashBody);

        impl $ty {
            pub const ALGS: &'static [&'static str] = &[$($alg),+];
            fn render(&self) -> String {
                format!("{}:{}:{}", $kind.as_str(), self.0.alg, self.0.hex)
            }
        }

        impl FromStr for $ty {
            type Err = IdError;
            fn from_str(id: &str) -> Result<Self, IdError> {
                let body = split_kind(id, $kind)?;
                HashBody::parse($kind, id, body, Self::ALGS).map($ty)
            }
        }

        string_id!($ty);
    };
}

hash_id!(WorkId, Kind::Work, ["nt1s256"]);
hash_id!(ItemId, Kind::Item, ["s256"]);
hash_id!(PlacementId, Kind::Placement, ["s256"]);

impl WorkId {
    /// The Work ID of extracted text under normalization v1 (plan PD2).
    pub fn from_text(text: &str) -> Self {
        WorkId(HashBody {
            alg: "nt1s256".into(),
            hex: sha256_hex(normalize_text_v1(text).as_bytes()),
        })
    }
}

impl ItemId {
    /// The Item ID of a file's bytes.
    pub fn from_bytes(bytes: &[u8]) -> Self {
        ItemId(HashBody {
            alg: "s256".into(),
            hex: sha256_hex(bytes),
        })
    }
}

impl PlacementId {
    /// The Placement ID of a claim's canonical JSON (without its own `id`).
    pub fn from_canonical(bytes: &[u8]) -> Self {
        PlacementId(HashBody {
            alg: "s256".into(),
            hex: sha256_hex(bytes),
        })
    }
}

/// `scheme:<name>@<version>`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SchemeId(pub Versioned);

/// `profile:<name>@<version>`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ProfileId(pub Versioned);

macro_rules! versioned_id {
    ($ty:ident, $kind:expr) => {
        impl $ty {
            pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
                $ty(Versioned::new(name, version))
            }
            fn render(&self) -> String {
                format!("{}:{}", $kind.as_str(), self.0)
            }
        }
        impl FromStr for $ty {
            type Err = IdError;
            fn from_str(id: &str) -> Result<Self, IdError> {
                let body = split_kind(id, $kind)?;
                Versioned::parse($kind, id, body).map($ty)
            }
        }
        string_id!($ty);
    };
}

versioned_id!(SchemeId, Kind::Scheme);
versioned_id!(ProfileId, Kind::Profile);

/// `class:<scheme>@<version>:<code>`. The code is everything after the
/// second colon: codes are scheme-defined and may themselves contain `:`,
/// so only the first two separators are structural. No whitespace.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ClassId {
    pub scheme: SchemeId,
    pub code: String,
}

impl ClassId {
    pub fn new(scheme: SchemeId, code: impl Into<String>) -> Self {
        ClassId {
            scheme,
            code: code.into(),
        }
    }

    fn render(&self) -> String {
        format!("class:{}:{}", self.scheme.0, self.code)
    }
}

impl FromStr for ClassId {
    type Err = IdError;
    fn from_str(id: &str) -> Result<Self, IdError> {
        let body = split_kind(id, Kind::Class)?;
        let bad = |reason| IdError::Malformed {
            kind: Kind::Class,
            id: id.to_string(),
            reason,
        };
        let (scheme, code) = body
            .split_once(':')
            .ok_or(bad("expected <scheme>@<version>:<code>"))?;
        if code.is_empty() || code.chars().any(char::is_whitespace) {
            return Err(bad("code must be non-empty with no whitespace"));
        }
        let scheme = SchemeId(Versioned::parse(Kind::Class, id, scheme)?);
        Ok(ClassId::new(scheme, code))
    }
}

string_id!(ClassId);

/// Any typed ID: what `resolve` accepts.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AnyId {
    Work(WorkId),
    Item(ItemId),
    Scheme(SchemeId),
    Class(ClassId),
    Profile(ProfileId),
    Placement(PlacementId),
}

impl AnyId {
    pub fn kind(&self) -> Kind {
        match self {
            AnyId::Work(_) => Kind::Work,
            AnyId::Item(_) => Kind::Item,
            AnyId::Scheme(_) => Kind::Scheme,
            AnyId::Class(_) => Kind::Class,
            AnyId::Profile(_) => Kind::Profile,
            AnyId::Placement(_) => Kind::Placement,
        }
    }
}

impl FromStr for AnyId {
    type Err = IdError;
    fn from_str(id: &str) -> Result<Self, IdError> {
        let (kind, _) = id
            .split_once(':')
            .ok_or_else(|| IdError::NoKind(id.to_string()))?;
        match Kind::parse(kind).ok_or_else(|| IdError::UnknownKind(kind.to_string()))? {
            Kind::Work => id.parse().map(AnyId::Work),
            Kind::Item => id.parse().map(AnyId::Item),
            Kind::Scheme => id.parse().map(AnyId::Scheme),
            Kind::Class => id.parse().map(AnyId::Class),
            Kind::Profile => id.parse().map(AnyId::Profile),
            Kind::Placement => id.parse().map(AnyId::Placement),
        }
    }
}

impl fmt::Display for AnyId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AnyId::Work(x) => x.fmt(f),
            AnyId::Item(x) => x.fmt(f),
            AnyId::Scheme(x) => x.fmt(f),
            AnyId::Class(x) => x.fmt(f),
            AnyId::Profile(x) => x.fmt(f),
            AnyId::Placement(x) => x.fmt(f),
        }
    }
}

impl Serialize for AnyId {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for AnyId {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        String::deserialize(d)?.parse().map_err(de::Error::custom)
    }
}
