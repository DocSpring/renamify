use clap::ValueEnum;
use renamify_core::{Preview, Style};

#[derive(Debug, Clone, Copy, PartialEq, ValueEnum)]
pub enum StyleArg {
    Snake,
    Kebab,
    Camel,
    Pascal,
    ScreamingSnake,
    Title,
    Train,
    ScreamingTrain,
    Dot,
    Lower,
    Upper,
}

impl From<StyleArg> for Style {
    fn from(arg: StyleArg) -> Self {
        match arg {
            StyleArg::Snake => Self::Snake,
            StyleArg::Kebab => Self::Kebab,
            StyleArg::Camel => Self::Camel,
            StyleArg::Pascal => Self::Pascal,
            StyleArg::ScreamingSnake => Self::ScreamingSnake,
            StyleArg::Title => Self::Title,
            StyleArg::Train => Self::Train,
            StyleArg::ScreamingTrain => Self::ScreamingTrain,
            StyleArg::Dot => Self::Dot,
            StyleArg::Lower => Self::Lower,
            StyleArg::Upper => Self::Upper,
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum, PartialEq)]
pub enum PreviewArg {
    Table,
    Diff,
    Matches,
    Summary,
    None,
}

impl PreviewArg {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "table" => Some(Self::Table),
            "diff" => Some(Self::Diff),
            "matches" => Some(Self::Matches),
            "summary" => Some(Self::Summary),
            "none" => Some(Self::None),
            _ => None,
        }
    }
}

impl From<PreviewArg> for Preview {
    fn from(arg: PreviewArg) -> Self {
        match arg {
            PreviewArg::Table => Self::Table,
            PreviewArg::Diff => Self::Diff,
            PreviewArg::Matches => Self::Matches,
            PreviewArg::Summary => Self::Summary,
            PreviewArg::None => Self::Table, // Default to table if None is somehow converted
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum, PartialEq)]
pub enum OutputFormat {
    Summary,
    Json,
}

#[derive(Debug, Clone, Copy, ValueEnum, PartialEq)]
pub enum SearchPreviewArg {
    Table,
    Matches,
    Summary,
    None,
}

impl From<SearchPreviewArg> for Preview {
    fn from(arg: SearchPreviewArg) -> Self {
        match arg {
            SearchPreviewArg::Table => Self::Table,
            SearchPreviewArg::Matches => Self::Matches,
            SearchPreviewArg::Summary => Self::Summary,
            SearchPreviewArg::None => Self::Matches, // Default to matches for search
        }
    }
}
