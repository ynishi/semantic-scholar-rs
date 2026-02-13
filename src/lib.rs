mod client;
pub mod error;
pub mod models;

pub use error::{Error, Result};
pub use models::*;

use client::HttpClient;

/// Commonly requested field sets for the Semantic Scholar API.
pub mod fields {
    /// Fields returned by default for paper search results.
    pub const PAPER_SEARCH: &str = "paperId,title,year,citationCount,authors";

    /// Detailed fields for a single paper lookup.
    pub const PAPER_DETAIL: &str = "paperId,title,abstract,year,citationCount,\
        referenceCount,influentialCitationCount,isOpenAccess,openAccessPdf,\
        fieldsOfStudy,s2FieldsOfStudy,publicationTypes,publicationDate,\
        journal,authors,tldr,venue";

    /// Fields returned for citation/reference entries.
    pub const CITATION: &str = "paperId,title,year,citationCount,authors";

    /// Fields returned by default for author search results.
    pub const AUTHOR_SEARCH: &str = "authorId,name,paperCount,citationCount,hIndex";

    /// Detailed fields for a single author lookup.
    pub const AUTHOR_DETAIL: &str =
        "authorId,name,affiliations,homepage,paperCount,citationCount,hIndex";
}

// ---------------------------------------------------------------------------
// Client
// ---------------------------------------------------------------------------

/// Semantic Scholar API client.
///
/// # Examples
///
/// ```no_run
/// # async fn example() -> semantic_scholar::Result<()> {
/// let client = semantic_scholar::SemanticScholar::new()?;
/// let results = client.search_papers("attention is all you need")
///     .limit(5)
///     .send()
///     .await?;
/// for paper in &results.data {
///     println!("{paper}");
/// }
/// # Ok(())
/// # }
/// ```
#[derive(Clone, Debug)]
pub struct SemanticScholar {
    client: HttpClient,
}

impl SemanticScholar {
    /// Create a client without an API key.
    ///
    /// Unauthenticated requests share a global rate limit pool
    /// (5 000 req / 5 min across *all* unauthenticated users).
    pub fn new() -> Result<Self> {
        Self::build(None)
    }

    /// Create a client with an API key for dedicated rate limits.
    pub fn with_api_key(api_key: &str) -> Result<Self> {
        Self::build(Some(api_key))
    }

    /// Override the base URL (for testing or self-hosted instances).
    pub fn set_base_url(&mut self, url: impl Into<String>) {
        self.client.set_base_url(url);
    }

    fn build(api_key: Option<&str>) -> Result<Self> {
        Ok(Self {
            client: HttpClient::new(api_key)?,
        })
    }

    // -- Paper endpoints ---------------------------------------------------

    /// Search for papers by keyword.
    pub fn search_papers(&self, query: &str) -> SearchPapersRequest<'_> {
        SearchPapersRequest::new(&self.client, query)
    }

    /// Get a single paper by its identifier.
    ///
    /// Accepts S2 paper ID, `DOI:…`, `ARXIV:…`, `CorpusId:…`, etc.
    pub fn get_paper(&self, paper_id: &str) -> GetPaperRequest<'_> {
        GetPaperRequest::new(&self.client, paper_id)
    }

    /// Convenience: get a paper by DOI.
    pub fn get_paper_by_doi(&self, doi: &str) -> GetPaperRequest<'_> {
        self.get_paper(&format!("DOI:{doi}"))
    }

    /// Convenience: get a paper by ArXiv ID.
    pub fn get_paper_by_arxiv(&self, arxiv_id: &str) -> GetPaperRequest<'_> {
        self.get_paper(&format!("ARXIV:{arxiv_id}"))
    }

    /// Get papers that cite the given paper.
    pub fn get_citations(&self, paper_id: &str) -> GetCitationsRequest<'_> {
        GetCitationsRequest::new(&self.client, paper_id)
    }

    /// Get papers referenced by the given paper.
    pub fn get_references(&self, paper_id: &str) -> GetReferencesRequest<'_> {
        GetReferencesRequest::new(&self.client, paper_id)
    }

    // -- Author endpoints --------------------------------------------------

    /// Search for authors by name.
    pub fn search_authors(&self, query: &str) -> SearchAuthorsRequest<'_> {
        SearchAuthorsRequest::new(&self.client, query)
    }

    /// Get a single author by their Semantic Scholar ID.
    pub fn get_author(&self, author_id: &str) -> GetAuthorRequest<'_> {
        GetAuthorRequest::new(&self.client, author_id)
    }

    // -- Recommendations ---------------------------------------------------

    /// Get recommended papers based on a given paper.
    pub fn get_recommendations(&self, paper_id: &str) -> GetRecommendationsRequest<'_> {
        GetRecommendationsRequest::new(&self.client, paper_id)
    }
}

// ===========================================================================
// Request builders
// ===========================================================================

/// Builder for `GET /graph/v1/paper/search`.
pub struct SearchPapersRequest<'a> {
    client: &'a HttpClient,
    query: String,
    limit: Option<u32>,
    offset: Option<u32>,
    year: Option<String>,
    fields_of_study: Option<String>,
    publication_types: Option<String>,
    open_access_pdf: Option<bool>,
    min_citation_count: Option<u32>,
    fields: String,
}

impl<'a> SearchPapersRequest<'a> {
    fn new(client: &'a HttpClient, query: &str) -> Self {
        Self {
            client,
            query: query.to_string(),
            limit: None,
            offset: None,
            year: None,
            fields_of_study: None,
            publication_types: None,
            open_access_pdf: None,
            min_citation_count: None,
            fields: fields::PAPER_SEARCH.to_string(),
        }
    }

    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }
    pub fn offset(mut self, offset: u32) -> Self {
        self.offset = Some(offset);
        self
    }
    /// Filter by year range, e.g. `"2020-2024"`, `"2020-"`, `"-2015"`.
    pub fn year(mut self, year: &str) -> Self {
        self.year = Some(year.to_string());
        self
    }
    /// Filter by fields of study (comma-separated).
    pub fn fields_of_study(mut self, fos: &str) -> Self {
        self.fields_of_study = Some(fos.to_string());
        self
    }
    pub fn publication_types(mut self, types: &str) -> Self {
        self.publication_types = Some(types.to_string());
        self
    }
    pub fn open_access_pdf(mut self) -> Self {
        self.open_access_pdf = Some(true);
        self
    }
    pub fn min_citation_count(mut self, count: u32) -> Self {
        self.min_citation_count = Some(count);
        self
    }
    /// Override which fields are returned (comma-separated).
    pub fn fields(mut self, fields: &str) -> Self {
        self.fields = fields.to_string();
        self
    }

    pub async fn send(self) -> Result<PaperSearchResponse> {
        let limit_str = self.limit.map(|l| l.to_string());
        let offset_str = self.offset.map(|o| o.to_string());
        let min_cc_str = self.min_citation_count.map(|c| c.to_string());

        let mut params: Vec<(&str, &str)> = vec![("query", &self.query), ("fields", &self.fields)];
        if let Some(ref l) = limit_str {
            params.push(("limit", l));
        }
        if let Some(ref o) = offset_str {
            params.push(("offset", o));
        }
        if let Some(ref y) = self.year {
            params.push(("year", y));
        }
        if let Some(ref fos) = self.fields_of_study {
            params.push(("fieldsOfStudy", fos));
        }
        if let Some(ref pt) = self.publication_types {
            params.push(("publicationTypes", pt));
        }
        if self.open_access_pdf == Some(true) {
            params.push(("openAccessPdf", ""));
        }
        if let Some(ref cc) = min_cc_str {
            params.push(("minCitationCount", cc));
        }

        self.client.get("/graph/v1/paper/search", &params).await
    }
}

/// Builder for `GET /graph/v1/paper/{paper_id}`.
pub struct GetPaperRequest<'a> {
    client: &'a HttpClient,
    paper_id: String,
    fields: String,
}

impl<'a> GetPaperRequest<'a> {
    fn new(client: &'a HttpClient, paper_id: &str) -> Self {
        Self {
            client,
            paper_id: paper_id.to_string(),
            fields: fields::PAPER_DETAIL.to_string(),
        }
    }

    pub fn fields(mut self, fields: &str) -> Self {
        self.fields = fields.to_string();
        self
    }

    pub async fn send(self) -> Result<Paper> {
        let path = format!("/graph/v1/paper/{}", self.paper_id);
        let params = [("fields", self.fields.as_str())];
        self.client.get(&path, &params).await
    }
}

/// Builder for `GET /graph/v1/paper/{paper_id}/citations`.
pub struct GetCitationsRequest<'a> {
    client: &'a HttpClient,
    paper_id: String,
    limit: Option<u32>,
    offset: Option<u32>,
    fields: String,
}

impl<'a> GetCitationsRequest<'a> {
    fn new(client: &'a HttpClient, paper_id: &str) -> Self {
        Self {
            client,
            paper_id: paper_id.to_string(),
            limit: None,
            offset: None,
            fields: fields::CITATION.to_string(),
        }
    }

    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }
    pub fn offset(mut self, offset: u32) -> Self {
        self.offset = Some(offset);
        self
    }
    pub fn fields(mut self, fields: &str) -> Self {
        self.fields = fields.to_string();
        self
    }

    pub async fn send(self) -> Result<CitationResponse> {
        let path = format!("/graph/v1/paper/{}/citations", self.paper_id);
        let limit_str = self.limit.map(|l| l.to_string());
        let offset_str = self.offset.map(|o| o.to_string());

        let mut params: Vec<(&str, &str)> = vec![("fields", &self.fields)];
        if let Some(ref l) = limit_str {
            params.push(("limit", l));
        }
        if let Some(ref o) = offset_str {
            params.push(("offset", o));
        }

        self.client.get(&path, &params).await
    }
}

/// Builder for `GET /graph/v1/paper/{paper_id}/references`.
pub struct GetReferencesRequest<'a> {
    client: &'a HttpClient,
    paper_id: String,
    limit: Option<u32>,
    offset: Option<u32>,
    fields: String,
}

impl<'a> GetReferencesRequest<'a> {
    fn new(client: &'a HttpClient, paper_id: &str) -> Self {
        Self {
            client,
            paper_id: paper_id.to_string(),
            limit: None,
            offset: None,
            fields: fields::CITATION.to_string(),
        }
    }

    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }
    pub fn offset(mut self, offset: u32) -> Self {
        self.offset = Some(offset);
        self
    }
    pub fn fields(mut self, fields: &str) -> Self {
        self.fields = fields.to_string();
        self
    }

    pub async fn send(self) -> Result<ReferenceResponse> {
        let path = format!("/graph/v1/paper/{}/references", self.paper_id);
        let limit_str = self.limit.map(|l| l.to_string());
        let offset_str = self.offset.map(|o| o.to_string());

        let mut params: Vec<(&str, &str)> = vec![("fields", &self.fields)];
        if let Some(ref l) = limit_str {
            params.push(("limit", l));
        }
        if let Some(ref o) = offset_str {
            params.push(("offset", o));
        }

        self.client.get(&path, &params).await
    }
}

/// Builder for `GET /graph/v1/author/search`.
pub struct SearchAuthorsRequest<'a> {
    client: &'a HttpClient,
    query: String,
    limit: Option<u32>,
    offset: Option<u32>,
    fields: String,
}

impl<'a> SearchAuthorsRequest<'a> {
    fn new(client: &'a HttpClient, query: &str) -> Self {
        Self {
            client,
            query: query.to_string(),
            limit: None,
            offset: None,
            fields: fields::AUTHOR_SEARCH.to_string(),
        }
    }

    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }
    pub fn offset(mut self, offset: u32) -> Self {
        self.offset = Some(offset);
        self
    }
    pub fn fields(mut self, fields: &str) -> Self {
        self.fields = fields.to_string();
        self
    }

    pub async fn send(self) -> Result<AuthorSearchResponse> {
        let limit_str = self.limit.map(|l| l.to_string());
        let offset_str = self.offset.map(|o| o.to_string());

        let mut params: Vec<(&str, &str)> = vec![("query", &self.query), ("fields", &self.fields)];
        if let Some(ref l) = limit_str {
            params.push(("limit", l));
        }
        if let Some(ref o) = offset_str {
            params.push(("offset", o));
        }

        self.client.get("/graph/v1/author/search", &params).await
    }
}

/// Builder for `GET /graph/v1/author/{author_id}`.
pub struct GetAuthorRequest<'a> {
    client: &'a HttpClient,
    author_id: String,
    fields: String,
}

impl<'a> GetAuthorRequest<'a> {
    fn new(client: &'a HttpClient, author_id: &str) -> Self {
        Self {
            client,
            author_id: author_id.to_string(),
            fields: fields::AUTHOR_DETAIL.to_string(),
        }
    }

    pub fn fields(mut self, fields: &str) -> Self {
        self.fields = fields.to_string();
        self
    }

    pub async fn send(self) -> Result<Author> {
        let path = format!("/graph/v1/author/{}", self.author_id);
        let params = [("fields", self.fields.as_str())];
        self.client.get(&path, &params).await
    }
}

/// Builder for `GET /recommendations/v1/papers/forpaper/{paper_id}`.
pub struct GetRecommendationsRequest<'a> {
    client: &'a HttpClient,
    paper_id: String,
    limit: Option<u32>,
    fields: String,
}

impl<'a> GetRecommendationsRequest<'a> {
    fn new(client: &'a HttpClient, paper_id: &str) -> Self {
        Self {
            client,
            paper_id: paper_id.to_string(),
            limit: None,
            fields: fields::PAPER_SEARCH.to_string(),
        }
    }

    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }
    pub fn fields(mut self, fields: &str) -> Self {
        self.fields = fields.to_string();
        self
    }

    pub async fn send(self) -> Result<RecommendationResponse> {
        let path = format!("/recommendations/v1/papers/forpaper/{}", self.paper_id);
        let limit_str = self.limit.map(|l| l.to_string());

        let mut params: Vec<(&str, &str)> = vec![("fields", &self.fields)];
        if let Some(ref l) = limit_str {
            params.push(("limit", l));
        }

        self.client.get(&path, &params).await
    }
}
