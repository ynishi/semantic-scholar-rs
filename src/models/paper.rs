use serde::{Deserialize, Serialize};
use std::fmt;

/// A paper in the Semantic Scholar corpus.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Paper {
    #[serde(default)]
    pub paper_id: String,
    #[serde(default)]
    pub corpus_id: Option<u64>,
    #[serde(default)]
    pub external_ids: Option<ExternalIds>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default, rename = "abstract")]
    pub abstract_text: Option<String>,
    #[serde(default)]
    pub venue: Option<String>,
    #[serde(default)]
    pub publication_venue: Option<PublicationVenue>,
    #[serde(default)]
    pub year: Option<u32>,
    #[serde(default)]
    pub reference_count: Option<u32>,
    #[serde(default)]
    pub citation_count: Option<u32>,
    #[serde(default)]
    pub influential_citation_count: Option<u32>,
    #[serde(default)]
    pub is_open_access: Option<bool>,
    #[serde(default)]
    pub open_access_pdf: Option<OpenAccessPdf>,
    #[serde(default)]
    pub fields_of_study: Option<Vec<String>>,
    #[serde(default)]
    pub s2_fields_of_study: Option<Vec<S2FieldOfStudy>>,
    #[serde(default)]
    pub publication_types: Option<Vec<String>>,
    #[serde(default)]
    pub publication_date: Option<String>,
    #[serde(default)]
    pub journal: Option<Journal>,
    #[serde(default)]
    pub citation_styles: Option<CitationStyles>,
    #[serde(default)]
    pub authors: Option<Vec<AuthorRef>>,
    #[serde(default)]
    pub citations: Option<Vec<Citation>>,
    #[serde(default)]
    pub references: Option<Vec<Reference>>,
    #[serde(default)]
    pub tldr: Option<Tldr>,
}

impl fmt::Display for Paper {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let title = self.title.as_deref().unwrap_or("(untitled)");
        write!(f, "{title}")?;
        if let Some(year) = self.year {
            write!(f, " ({year})")?;
        }
        if let Some(authors) = &self.authors {
            let names: Vec<&str> = authors
                .iter()
                .filter_map(|a| a.name.as_deref())
                .take(3)
                .collect();
            if !names.is_empty() {
                write!(f, " — {}", names.join(", "))?;
                if authors.len() > 3 {
                    write!(f, " et al.")?;
                }
            }
        }
        Ok(())
    }
}

/// External identifiers for a paper (DOI, ArXiv, etc.).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExternalIds {
    #[serde(default, rename = "DOI")]
    pub doi: Option<String>,
    #[serde(default, rename = "ArXiv")]
    pub arxiv: Option<String>,
    #[serde(default, rename = "MAG")]
    pub mag: Option<String>,
    #[serde(default, rename = "ACL")]
    pub acl: Option<String>,
    #[serde(default, rename = "PubMed")]
    pub pubmed: Option<String>,
    #[serde(default, rename = "PubMedCentral")]
    pub pubmed_central: Option<String>,
    #[serde(default, rename = "DBLP")]
    pub dblp: Option<String>,
    #[serde(default, rename = "CorpusId")]
    pub corpus_id: Option<u64>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PublicationVenue {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default, rename = "type")]
    pub venue_type: Option<String>,
    #[serde(default)]
    pub alternate_names: Option<Vec<String>>,
    #[serde(default)]
    pub url: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OpenAccessPdf {
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct S2FieldOfStudy {
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub source: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Journal {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub pages: Option<String>,
    #[serde(default)]
    pub volume: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CitationStyles {
    #[serde(default)]
    pub bibtex: Option<String>,
}

/// Minimal author reference embedded in paper responses.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthorRef {
    #[serde(default)]
    pub author_id: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
}

/// A citing paper with citation context.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Citation {
    #[serde(default)]
    pub citing_paper: Option<Paper>,
    #[serde(default)]
    pub contexts: Option<Vec<String>>,
    #[serde(default)]
    pub intents: Option<Vec<String>>,
    #[serde(default)]
    pub is_influential: Option<bool>,
}

/// A referenced paper with citation context.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Reference {
    #[serde(default)]
    pub cited_paper: Option<Paper>,
    #[serde(default)]
    pub contexts: Option<Vec<String>>,
    #[serde(default)]
    pub intents: Option<Vec<String>>,
    #[serde(default)]
    pub is_influential: Option<bool>,
}

/// TL;DR auto-generated summary.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Tldr {
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub text: Option<String>,
}

// ---------------------------------------------------------------------------
// Response types
// ---------------------------------------------------------------------------

/// Response from `GET /paper/search`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaperSearchResponse {
    #[serde(default)]
    pub total: u64,
    #[serde(default)]
    pub offset: Option<u32>,
    #[serde(default)]
    pub next: Option<u32>,
    #[serde(default)]
    pub data: Vec<Paper>,
}

/// Response from `GET /paper/{id}/citations`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CitationResponse {
    #[serde(default)]
    pub offset: Option<u32>,
    #[serde(default)]
    pub next: Option<u32>,
    #[serde(default)]
    pub data: Vec<Citation>,
}

/// Response from `GET /paper/{id}/references`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferenceResponse {
    #[serde(default)]
    pub offset: Option<u32>,
    #[serde(default)]
    pub next: Option<u32>,
    #[serde(default)]
    pub data: Vec<Reference>,
}

/// Response from `GET /recommendations/v1/papers/forpaper/{id}`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecommendationResponse {
    #[serde(default)]
    pub recommended_papers: Vec<Paper>,
}
