//! The arXiv taxonomy as arxiv-base defines it: groups, archives,
//! categories, the defunct archives later categories subsumed, category
//! aliases and the bare archive names old submissions used.

use crate::pydef::{assignment, Py};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Group {
    pub id: String,
    pub full_name: String,
    pub is_test: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Archive {
    pub id: String,
    pub full_name: String,
    pub in_group: String,
    pub is_active: bool,
    /// `YYYY-MM`.
    pub start: Option<String>,
    pub end: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Category {
    pub id: String,
    pub full_name: String,
    pub in_archive: String,
    pub is_active: bool,
    pub alt_name: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Taxonomy {
    pub groups: Vec<Group>,
    pub archives: Vec<Archive>,
    pub categories: Vec<Category>,
    /// Defunct archive -> the category that took it over.
    pub subsumed: Vec<(String, String)>,
    /// Alias category -> canonical category.
    pub aliases: Vec<(String, String)>,
    /// Bare archive name -> last month (`YYYY-MM`) it was allowed as a
    /// primary category.
    pub legacy_primary: Vec<(String, String)>,
    /// The same, as a secondary category.
    pub legacy_secondary: Vec<(String, String)>,
}

/// Keyword arguments of one constructor call, checked against the fields
/// the reader knows so a new field is noticed.
struct Fields<'a> {
    ctx: String,
    args: &'a [(Option<String>, Py)],
}

impl<'a> Fields<'a> {
    fn of(v: &'a Py, ctor: &str, known: &[&str], ctx: String) -> Result<Self, String> {
        let Py::Call { name, args } = v else {
            return Err(format!("{ctx}: not a {ctor}(...) call"));
        };
        if name != ctor {
            return Err(format!("{ctx}: {name}(...), expected {ctor}(...)"));
        }
        for (k, _) in args {
            match k {
                Some(k) if known.contains(&k.as_str()) => {}
                Some(k) => return Err(format!("{ctx}: unknown field `{k}`")),
                None => return Err(format!("{ctx}: positional argument")),
            }
        }
        Ok(Fields { ctx, args })
    }

    fn get(&self, key: &str) -> Option<&'a Py> {
        self.args
            .iter()
            .find(|(k, _)| k.as_deref() == Some(key))
            .map(|(_, v)| v)
    }

    fn str(&self, key: &str) -> Result<String, String> {
        self.opt_str(key)?
            .ok_or_else(|| format!("{}: missing `{key}`", self.ctx))
    }

    fn opt_str(&self, key: &str) -> Result<Option<String>, String> {
        match self.get(key) {
            None | Some(Py::None) => Ok(None),
            Some(Py::Str(s)) => Ok(Some(s.clone())),
            Some(v) => Err(format!("{}: `{key}` is {v:?}, not a string", self.ctx)),
        }
    }

    fn bool(&self, key: &str) -> Result<bool, String> {
        match self.get(key) {
            None => Ok(false),
            Some(Py::Bool(b)) => Ok(*b),
            Some(v) => Err(format!("{}: `{key}` is {v:?}, not a bool", self.ctx)),
        }
    }

    fn month(&self, key: &str) -> Result<Option<String>, String> {
        self.get(key)
            .map(|v| month(v).map_err(|e| format!("{}: `{key}`: {e}", self.ctx)))
            .transpose()
    }
}

/// `date(y, m, d)` as `YYYY-MM`.
fn month(v: &Py) -> Result<String, String> {
    match v {
        Py::Call { name, args } if name == "date" && args.len() == 3 => {
            match (&args[0], &args[1]) {
                ((None, Py::Int(y)), (None, Py::Int(m))) => Ok(format!("{y:04}-{m:02}")),
                _ => Err(format!("unexpected date arguments {args:?}")),
            }
        }
        _ => Err(format!("{v:?} is not date(y, m, d)")),
    }
}

/// A dict assignment's entries, keyed by string.
fn entries(src: &str, target: &str) -> Result<Vec<(String, Py)>, String> {
    let Py::Dict(items) = assignment(src, target)? else {
        return Err(format!("{target} is not a dict"));
    };
    items
        .into_iter()
        .map(|(k, v)| match k {
            Py::Str(k) => Ok((k, v)),
            k => Err(format!("{target}: key {k:?} is not a string")),
        })
        .collect()
}

fn string_map(src: &str, target: &str) -> Result<Vec<(String, String)>, String> {
    entries(src, target)?
        .into_iter()
        .map(|(k, v)| match v {
            Py::Str(v) => Ok((k, v)),
            v => Err(format!("{target}[{k}] is {v:?}, not a string")),
        })
        .collect()
}

fn month_map(src: &str, target: &str) -> Result<Vec<(String, String)>, String> {
    entries(src, target)?
        .into_iter()
        .map(|(k, v)| {
            Ok((
                k.clone(),
                month(&v).map_err(|e| format!("{target}[{k}]: {e}"))?,
            ))
        })
        .collect()
}

/// Check that a dict key equals the entry's own `id`.
fn keyed(key: &str, id: String, ctx: &str) -> Result<String, String> {
    if key == id {
        Ok(id)
    } else {
        Err(format!("{ctx}: key `{key}` but id `{id}`"))
    }
}

/// Read the taxonomy out of `definitions.py`.
pub fn parse(src: &str) -> Result<Taxonomy, String> {
    let mut t = Taxonomy::default();
    for (key, v) in entries(src, "GROUPS")? {
        let ctx = format!("GROUPS[{key}]");
        let known = [
            "id",
            "full_name",
            "is_active",
            "start_year",
            "default_archive",
            "is_test",
        ];
        let f = Fields::of(&v, "Group", &known, ctx.clone())?;
        t.groups.push(Group {
            id: keyed(&key, f.str("id")?, &ctx)?,
            full_name: f.str("full_name")?,
            is_test: f.bool("is_test")?,
        });
    }
    for (key, v) in entries(src, "ARCHIVES")? {
        let ctx = format!("ARCHIVES[{key}]");
        let known = [
            "id",
            "full_name",
            "is_active",
            "alt_name",
            "in_group",
            "start_date",
            "end_date",
        ];
        let f = Fields::of(&v, "Archive", &known, ctx.clone())?;
        t.archives.push(Archive {
            id: keyed(&key, f.str("id")?, &ctx)?,
            full_name: f.str("full_name")?,
            in_group: f.str("in_group")?,
            is_active: f.bool("is_active")?,
            start: f.month("start_date")?,
            end: f.month("end_date")?,
        });
    }
    for (key, v) in entries(src, "CATEGORIES")? {
        let ctx = format!("CATEGORIES[{key}]");
        let known = [
            "id",
            "full_name",
            "is_active",
            "alt_name",
            "in_archive",
            "is_general",
            "description",
        ];
        let f = Fields::of(&v, "Category", &known, ctx.clone())?;
        t.categories.push(Category {
            id: keyed(&key, f.str("id")?, &ctx)?,
            full_name: f.str("full_name")?,
            in_archive: f.str("in_archive")?,
            is_active: f.bool("is_active")?,
            alt_name: f.opt_str("alt_name")?,
            description: f.opt_str("description")?,
        });
    }
    t.subsumed = string_map(src, "ARCHIVES_SUBSUMED")?;
    t.aliases = string_map(src, "CATEGORY_ALIASES")?;
    t.legacy_primary = month_map(src, "LEGACY_ARCHIVE_AS_PRIMARY")?;
    t.legacy_secondary = month_map(src, "LEGACY_ARCHIVE_AS_SECONDARY")?;
    Ok(t)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SRC: &str = r#"
GROUPS = {"grp_math": Group(id="grp_math", full_name="Mathematics", is_active=True)}
ARCHIVES = {"math": Archive(id="math", full_name="Mathematics", is_active=True,
    in_group="grp_math", start_date=date(1992, 2, 1))}
ARCHIVES_SUBSUMED = {'alg-geom': 'math.AG'}
LEGACY_ARCHIVE_AS_PRIMARY = {'astro-ph': date(2008, 12, 1)}
LEGACY_ARCHIVE_AS_SECONDARY = {}
CATEGORIES = {"math.AG": Category(id="math.AG", full_name="Algebraic Geometry",
    is_active=True, alt_name="alg-geom", in_archive="math", is_general=False,
    description="Algebraic varieties")}
CATEGORY_ALIASES = {}
"#;

    #[test]
    fn reads_the_model() {
        let t = parse(SRC).unwrap();
        assert_eq!(t.archives[0].start.as_deref(), Some("1992-02"));
        assert_eq!(t.categories[0].alt_name.as_deref(), Some("alg-geom"));
        assert_eq!(
            t.legacy_primary,
            vec![("astro-ph".into(), "2008-12".into())]
        );
        assert!(!t.groups[0].is_test);
    }

    #[test]
    fn refuses_unknown_fields_and_mismatched_keys() {
        let extra = SRC.replace("is_general=False", "is_general=False, colour=\"red\"");
        assert!(parse(&extra)
            .unwrap_err()
            .contains("unknown field `colour`"));
        let key = SRC.replace("\"math.AG\": Category", "\"math.AX\": Category");
        assert!(parse(&key)
            .unwrap_err()
            .contains("key `math.AX` but id `math.AG`"));
    }
}
