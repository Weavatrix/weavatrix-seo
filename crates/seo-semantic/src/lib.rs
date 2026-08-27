//! Semantic inference over crawled pages via weavatrix-semantic.

#![forbid(unsafe_code)]

mod analyze;
mod embed;
mod inputs;

pub use analyze::{SemanticPass, analyze};
pub use embed::{DIM, MODEL, embed};
pub use inputs::{LinkInputs, PageRow, VectorRow, link_inputs};
