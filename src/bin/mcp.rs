// rmcp #[tool_router] macro generates collapsible-if patterns
#![allow(clippy::collapsible_if)]

use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::*;
use rmcp::{tool, tool_handler, tool_router, ServerHandler, ServiceExt};
use schemars::JsonSchema;
use serde::Deserialize;

use semantic_scholar::{
    Author, CitationResponse, Paper, PaperSearchResponse, RecommendationResponse,
    ReferenceResponse, SemanticScholar,
};

// ---------------------------------------------------------------------------
// Tool parameter types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, JsonSchema)]
struct SearchPapersParams {
    /// Search query for academic papers.
    query: String,
    /// Maximum number of results (1–100).
    limit: Option<u32>,
    /// Year or year range, e.g. "2020", "2020-2024", "2020-".
    year: Option<String>,
    /// Comma-separated fields of study, e.g. "Computer Science,Physics".
    fields_of_study: Option<String>,
    /// Minimum citation count filter.
    min_citation_count: Option<u32>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct GetPaperParams {
    /// Paper identifier: S2 ID, DOI:…, ARXIV:…, CorpusId:…, etc.
    paper_id: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct SearchAuthorsParams {
    /// Author name to search for.
    query: String,
    /// Maximum number of results (1–100).
    limit: Option<u32>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct GetAuthorParams {
    /// Semantic Scholar author ID.
    author_id: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct PaperIdWithLimit {
    /// Paper identifier.
    paper_id: String,
    /// Maximum number of results.
    limit: Option<u32>,
}

// ---------------------------------------------------------------------------
// MCP server
// ---------------------------------------------------------------------------

#[derive(Clone)]
struct Server {
    sdk: SemanticScholar,
    tool_router: ToolRouter<Self>,
}

fn mcp_error(msg: impl Into<String>) -> rmcp::ErrorData {
    rmcp::ErrorData::internal_error(msg.into(), None)
}

#[tool_router]
impl Server {
    fn new(sdk: SemanticScholar) -> Self {
        Self {
            sdk,
            tool_router: Self::tool_router(),
        }
    }

    #[tool(
        description = "Search for academic papers by keyword. Returns titles, years, citation counts, and authors."
    )]
    async fn search_papers(
        &self,
        Parameters(params): Parameters<SearchPapersParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let mut req = self.sdk.search_papers(&params.query);
        if let Some(limit) = params.limit {
            req = req.limit(limit);
        }
        if let Some(ref year) = params.year {
            req = req.year(year);
        }
        if let Some(ref fos) = params.fields_of_study {
            req = req.fields_of_study(fos);
        }
        if let Some(mcc) = params.min_citation_count {
            req = req.min_citation_count(mcc);
        }

        let resp: PaperSearchResponse = req.send().await.map_err(|e| mcp_error(e.to_string()))?;
        Ok(CallToolResult::success(vec![Content::text(
            format_paper_search(&resp),
        )]))
    }

    #[tool(
        description = "Get detailed information about a specific paper. Accepts S2 paper ID, DOI:…, ARXIV:…, CorpusId:…"
    )]
    async fn get_paper(
        &self,
        Parameters(params): Parameters<GetPaperParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let paper: Paper = self
            .sdk
            .get_paper(&params.paper_id)
            .send()
            .await
            .map_err(|e| mcp_error(e.to_string()))?;
        Ok(CallToolResult::success(vec![Content::text(
            format_paper_detail(&paper),
        )]))
    }

    #[tool(description = "Search for academic authors by name.")]
    async fn search_authors(
        &self,
        Parameters(params): Parameters<SearchAuthorsParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let mut req = self.sdk.search_authors(&params.query);
        if let Some(limit) = params.limit {
            req = req.limit(limit);
        }

        let resp = req.send().await.map_err(|e| mcp_error(e.to_string()))?;
        Ok(CallToolResult::success(vec![Content::text(
            format_author_search(&resp),
        )]))
    }

    #[tool(
        description = "Get detailed information about a specific author by their Semantic Scholar ID."
    )]
    async fn get_author(
        &self,
        Parameters(params): Parameters<GetAuthorParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let author: Author = self
            .sdk
            .get_author(&params.author_id)
            .send()
            .await
            .map_err(|e| mcp_error(e.to_string()))?;
        Ok(CallToolResult::success(vec![Content::text(
            format_author_detail(&author),
        )]))
    }

    #[tool(description = "Get papers that cite the given paper.")]
    async fn get_citations(
        &self,
        Parameters(params): Parameters<PaperIdWithLimit>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let mut req = self.sdk.get_citations(&params.paper_id);
        if let Some(limit) = params.limit {
            req = req.limit(limit);
        }

        let resp: CitationResponse = req.send().await.map_err(|e| mcp_error(e.to_string()))?;
        Ok(CallToolResult::success(vec![Content::text(
            format_citations(&resp),
        )]))
    }

    #[tool(description = "Get papers referenced by the given paper.")]
    async fn get_references(
        &self,
        Parameters(params): Parameters<PaperIdWithLimit>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let mut req = self.sdk.get_references(&params.paper_id);
        if let Some(limit) = params.limit {
            req = req.limit(limit);
        }

        let resp: ReferenceResponse = req.send().await.map_err(|e| mcp_error(e.to_string()))?;
        Ok(CallToolResult::success(vec![Content::text(
            format_references(&resp),
        )]))
    }

    #[tool(description = "Get paper recommendations based on a given paper.")]
    async fn get_recommendations(
        &self,
        Parameters(params): Parameters<PaperIdWithLimit>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let mut req = self.sdk.get_recommendations(&params.paper_id);
        if let Some(limit) = params.limit {
            req = req.limit(limit);
        }

        let resp: RecommendationResponse =
            req.send().await.map_err(|e| mcp_error(e.to_string()))?;
        Ok(CallToolResult::success(vec![Content::text(
            format_recommendations(&resp),
        )]))
    }
}

#[tool_handler]
impl ServerHandler for Server {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation {
                name: "semantic-scholar-mcp".into(),
                version: env!("CARGO_PKG_VERSION").into(),
                title: None,
                description: None,
                icons: None,
                website_url: None,
            },
            instructions: Some(
                "Semantic Scholar academic paper search. \
                 Search papers, get details, citations, references, \
                 author info, and paper recommendations."
                    .into(),
            ),
        }
    }
}

// ---------------------------------------------------------------------------
// Formatters (Markdown output for LLM consumption)
// ---------------------------------------------------------------------------

fn format_paper_search(resp: &PaperSearchResponse) -> String {
    let mut out = format!("Found {} papers:\n\n", resp.total);
    for (i, p) in resp.data.iter().enumerate() {
        out.push_str(&format!("{}. {}\n", i + 1, format_paper_oneline(p)));
    }
    out
}

fn format_paper_oneline(p: &Paper) -> String {
    let title = p.title.as_deref().unwrap_or("(untitled)");
    let year = p.year.map(|y| format!(" ({y})")).unwrap_or_default();
    let cites = p
        .citation_count
        .map(|c| format!(" [{c} citations]"))
        .unwrap_or_default();
    let authors = p
        .authors
        .as_ref()
        .map(|a| {
            let names: Vec<&str> = a.iter().filter_map(|x| x.name.as_deref()).take(3).collect();
            if names.is_empty() {
                String::new()
            } else {
                let suffix = if a.len() > 3 { " et al." } else { "" };
                format!(" — {}{suffix}", names.join(", "))
            }
        })
        .unwrap_or_default();
    let id = &p.paper_id;
    format!("**{title}**{year}{cites}{authors} `{id}`")
}

fn format_paper_detail(p: &Paper) -> String {
    let mut out = String::new();
    let title = p.title.as_deref().unwrap_or("(untitled)");
    out.push_str(&format!("# {title}\n\n"));
    out.push_str(&format!("- **Paper ID**: {}\n", p.paper_id));

    if let Some(year) = p.year {
        out.push_str(&format!("- **Year**: {year}\n"));
    }
    if let Some(venue) = &p.venue {
        out.push_str(&format!("- **Venue**: {venue}\n"));
    }
    if let Some(cc) = p.citation_count {
        out.push_str(&format!("- **Citations**: {cc}\n"));
    }
    if let Some(rc) = p.reference_count {
        out.push_str(&format!("- **References**: {rc}\n"));
    }
    if let Some(oa) = p.is_open_access {
        out.push_str(&format!("- **Open Access**: {oa}\n"));
    }
    if let Some(ref pdf) = p.open_access_pdf {
        if let Some(ref url) = pdf.url {
            out.push_str(&format!("- **PDF**: {url}\n"));
        }
    }
    if let Some(date) = &p.publication_date {
        out.push_str(&format!("- **Published**: {date}\n"));
    }
    if let Some(fos) = &p.fields_of_study {
        out.push_str(&format!("- **Fields**: {}\n", fos.join(", ")));
    }

    if let Some(authors) = &p.authors {
        let names: Vec<&str> = authors.iter().filter_map(|a| a.name.as_deref()).collect();
        if !names.is_empty() {
            out.push_str(&format!("\n**Authors**: {}\n", names.join(", ")));
        }
    }

    if let Some(ids) = &p.external_ids {
        out.push('\n');
        if let Some(doi) = &ids.doi {
            out.push_str(&format!("- DOI: {doi}\n"));
        }
        if let Some(arxiv) = &ids.arxiv {
            out.push_str(&format!("- ArXiv: {arxiv}\n"));
        }
    }

    if let Some(tldr) = &p.tldr {
        if let Some(text) = &tldr.text {
            out.push_str(&format!("\n**TL;DR**: {text}\n"));
        }
    }
    if let Some(abs) = &p.abstract_text {
        out.push_str(&format!("\n**Abstract**: {abs}\n"));
    }
    out
}

fn format_author_search(resp: &semantic_scholar::AuthorSearchResponse) -> String {
    let mut out = format!("Found {} authors:\n\n", resp.total);
    for (i, a) in resp.data.iter().enumerate() {
        out.push_str(&format!("{}. {}\n", i + 1, a));
    }
    out
}

fn format_author_detail(a: &Author) -> String {
    let mut out = String::new();
    let name = a.name.as_deref().unwrap_or("(unknown)");
    out.push_str(&format!("# {name}\n\n"));
    out.push_str(&format!("- **Author ID**: {}\n", a.author_id));
    if let Some(h) = a.h_index {
        out.push_str(&format!("- **h-index**: {h}\n"));
    }
    if let Some(pc) = a.paper_count {
        out.push_str(&format!("- **Papers**: {pc}\n"));
    }
    if let Some(cc) = a.citation_count {
        out.push_str(&format!("- **Citations**: {cc}\n"));
    }
    if let Some(affs) = &a.affiliations {
        if !affs.is_empty() {
            out.push_str(&format!("- **Affiliations**: {}\n", affs.join(", ")));
        }
    }
    if let Some(hp) = &a.homepage {
        out.push_str(&format!("- **Homepage**: {hp}\n"));
    }
    out
}

fn format_citations(resp: &CitationResponse) -> String {
    let mut out = format!("{} citing papers:\n\n", resp.data.len());
    for (i, c) in resp.data.iter().enumerate() {
        if let Some(p) = &c.citing_paper {
            let influential = if c.is_influential == Some(true) {
                " *influential*"
            } else {
                ""
            };
            out.push_str(&format!(
                "{}. {}{influential}\n",
                i + 1,
                format_paper_oneline(p)
            ));
        }
    }
    out
}

fn format_references(resp: &ReferenceResponse) -> String {
    let mut out = format!("{} referenced papers:\n\n", resp.data.len());
    for (i, r) in resp.data.iter().enumerate() {
        if let Some(p) = &r.cited_paper {
            let influential = if r.is_influential == Some(true) {
                " *influential*"
            } else {
                ""
            };
            out.push_str(&format!(
                "{}. {}{influential}\n",
                i + 1,
                format_paper_oneline(p)
            ));
        }
    }
    out
}

fn format_recommendations(resp: &RecommendationResponse) -> String {
    let mut out = format!("{} recommended papers:\n\n", resp.recommended_papers.len());
    for (i, p) in resp.recommended_papers.iter().enumerate() {
        out.push_str(&format!("{}. {}\n", i + 1, format_paper_oneline(p)));
    }
    out
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = std::env::var("SEMANTIC_SCHOLAR_API_KEY").ok();
    let sdk = match api_key.as_deref() {
        Some(key) => SemanticScholar::with_api_key(key)?,
        None => SemanticScholar::new()?,
    };

    let service = Server::new(sdk).serve(rmcp::transport::stdio()).await?;

    service.waiting().await?;
    Ok(())
}
