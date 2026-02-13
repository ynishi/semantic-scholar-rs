# semantic-scholar-rs

Unofficial Rust SDK and MCP server for the [Semantic Scholar API](https://api.semanticscholar.org/).

> **Note:** This is a community-maintained project and is not affiliated with or endorsed by the Semantic Scholar team at the Allen Institute for AI.

[![Crates.io](https://img.shields.io/crates/v/semantic-scholar-rs.svg)](https://crates.io/crates/semantic-scholar-rs)
[![docs.rs](https://docs.rs/semantic-scholar-rs/badge.svg)](https://docs.rs/semantic-scholar-rs)
[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

## Features

- Paper search, lookup by ID / DOI / ArXiv ID
- Citations and references
- Author search and lookup
- Paper recommendations
- Automatic retry with exponential backoff (429 / 5xx)
- Optional MCP server for LLM tool-use integration

## Quick Start

```toml
[dependencies]
semantic-scholar-rs = "0.1"
tokio = { version = "1", features = ["full"] }
```

```rust
use semantic_scholar::SemanticScholar;

#[tokio::main]
async fn main() -> semantic_scholar::Result<()> {
    let client = SemanticScholar::new()?;

    // Search papers
    let results = client.search_papers("attention is all you need")
        .limit(5)
        .send()
        .await?;

    for paper in &results.data {
        println!("{paper}");
    }

    // Get a specific paper
    let paper = client.get_paper("204e3073870fae3d05bcbc2f6a8e263d9b72e776")
        .send()
        .await?;
    println!("{paper}");

    Ok(())
}
```

## API Key

Unauthenticated requests share a global rate-limit pool (100 req/min across all unauthenticated users). For dedicated limits, obtain a key from [Semantic Scholar API](https://www.semanticscholar.org/product/api#api-key) and pass it to the client:

```rust
let client = SemanticScholar::with_api_key("your-key")?;
```

## Endpoints

| Method | Description |
|---|---|
| `search_papers(query)` | Keyword search with filters (year, field of study, etc.) |
| `get_paper(id)` | Lookup by S2 ID, `DOI:...`, `ARXIV:...`, `CorpusId:...` |
| `get_paper_by_doi(doi)` | Convenience wrapper for DOI lookup |
| `get_paper_by_arxiv(id)` | Convenience wrapper for ArXiv lookup |
| `get_citations(paper_id)` | Papers that cite the given paper |
| `get_references(paper_id)` | Papers referenced by the given paper |
| `search_authors(query)` | Author name search |
| `get_author(author_id)` | Author details by S2 author ID |
| `get_recommendations(paper_id)` | Recommended papers based on a paper |

All request builders support `.limit()`, `.offset()`, and `.fields()` for pagination and field selection.

## MCP Server

An [MCP](https://modelcontextprotocol.io/) server is included as an optional binary, exposing all endpoints as LLM-callable tools over stdio.

### Build & Run

```sh
cargo build --release --features mcp
./target/release/semantic-scholar-mcp
```

### Claude Desktop Configuration

```json
{
  "mcpServers": {
    "semantic-scholar": {
      "command": "/path/to/semantic-scholar-mcp",
      "env": {
        "SEMANTIC_SCHOLAR_API_KEY": "your-key"
      }
    }
  }
}
```

### Available Tools

| Tool | Description |
|---|---|
| `search_papers` | Search academic papers by keyword |
| `get_paper` | Get paper details by identifier |
| `search_authors` | Search authors by name |
| `get_author` | Get author details by ID |
| `get_citations` | Get citing papers |
| `get_references` | Get referenced papers |
| `get_recommendations` | Get paper recommendations |

## Testing

```sh
# Unit tests (offline, CI-safe)
cargo test

# MCP protocol E2E (no API calls)
cargo test --features mcp --test mcp_e2e_test

# Full integration tests (hits real API)
cargo test --features mcp,integration-test -- --test-threads=1
```

## License

MIT
