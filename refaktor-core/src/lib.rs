#![allow(unused)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]

pub mod case_model;
pub mod pattern;

pub use case_model::{
    detect_style, generate_variant_map, parse_to_tokens, to_style, Style, Token, TokenModel,
};
pub use pattern::{build_pattern, find_matches, is_boundary, Match, MatchPattern};

