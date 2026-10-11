//! A learned, order-preserving general classification: a class tree fitted
//! to the benchmark's similarity graph by minimizing normalized Dasgupta
//! cost plus a weighted expected-navigation cost, under bounded branching
//! and depth, with stable codes when new works are inserted. See README.md
//! for the objective, the algorithm and the guarantees.

pub mod binary;
pub mod classtree;
pub mod collapse;
pub mod fit;
pub mod graph;
pub mod insert;
pub mod rng;
pub mod scheme;
pub mod search;
pub mod stopwords;
pub mod terms;
pub mod vectors;

pub use classtree::{ClassTree, Objective, Params};
pub use fit::{fit, Fit, FitConfig};
pub use graph::Graph;
pub use insert::{CentroidFile, InsertParams, Inserter};
