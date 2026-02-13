use serde::{Deserialize, Serialize};
use std::fmt;

use super::paper::Paper;

/// An author in the Semantic Scholar corpus.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Author {
    #[serde(default)]
    pub author_id: String,
    #[serde(default)]
    pub external_ids: Option<AuthorExternalIds>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub affiliations: Option<Vec<String>>,
    #[serde(default)]
    pub homepage: Option<String>,
    #[serde(default)]
    pub paper_count: Option<u32>,
    #[serde(default)]
    pub citation_count: Option<u32>,
    #[serde(default)]
    pub h_index: Option<u32>,
    #[serde(default)]
    pub papers: Option<Vec<Paper>>,
}

impl fmt::Display for Author {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = self.name.as_deref().unwrap_or("(unknown)");
        write!(f, "{name}")?;
        if let Some(h) = self.h_index {
            write!(f, " (h-index: {h})")?;
        }
        if let Some(count) = self.paper_count {
            write!(f, " — {count} papers")?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AuthorExternalIds {
    #[serde(default, rename = "DBLP")]
    pub dblp: Option<Vec<String>>,
}

/// Response from `GET /author/search`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorSearchResponse {
    #[serde(default)]
    pub total: u64,
    #[serde(default)]
    pub offset: Option<u32>,
    #[serde(default)]
    pub next: Option<u32>,
    #[serde(default)]
    pub data: Vec<Author>,
}
