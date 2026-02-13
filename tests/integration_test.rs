//! Integration tests that hit the real Semantic Scholar API.
//!
//! Gated behind `integration-test` feature to avoid running in CI.
//!
//! ```sh
//! # Recommended: set API key to avoid shared rate limits
//! export SEMANTIC_SCHOLAR_API_KEY="your-key"
//! cargo test --features integration-test -- --test-threads=1
//! ```
#![cfg(feature = "integration-test")]

use semantic_scholar::SemanticScholar;

/// "Attention Is All You Need" — stable, highly-cited paper.
const ATTENTION_PAPER_ID: &str = "204e3073870fae3d05bcbc2f6a8e263d9b72e776";

/// Ashish Vaswani — first author of "Attention Is All You Need".
const VASWANI_AUTHOR_ID: &str = "40348417";

fn build_client() -> SemanticScholar {
    match std::env::var("SEMANTIC_SCHOLAR_API_KEY") {
        Ok(key) => SemanticScholar::with_api_key(&key).expect("build client with API key"),
        Err(_) => SemanticScholar::new().expect("build client"),
    }
}

// ---------------------------------------------------------------------------
// Paper endpoints
// ---------------------------------------------------------------------------

#[tokio::test]
async fn search_papers_returns_results() {
    let client = build_client();
    let resp = client
        .search_papers("attention is all you need")
        .limit(3)
        .send()
        .await
        .expect("search_papers");

    assert!(resp.total > 0, "expected non-zero total");
    assert!(!resp.data.is_empty(), "expected at least one result");
    assert!(resp.data.len() <= 3, "limit should cap results");
}

#[tokio::test]
async fn search_papers_with_year_filter() {
    let client = build_client();
    let resp = client
        .search_papers("transformer")
        .limit(5)
        .year("2020-2023")
        .send()
        .await
        .expect("search with year filter");

    assert!(resp.total > 0);
    for paper in &resp.data {
        if let Some(year) = paper.year {
            assert!(
                (2020..=2023).contains(&year),
                "paper year {year} outside filter range"
            );
        }
    }
}

#[tokio::test]
async fn get_paper_by_id() {
    let client = build_client();
    let paper = client
        .get_paper(ATTENTION_PAPER_ID)
        .send()
        .await
        .expect("get_paper");

    assert_eq!(paper.paper_id, ATTENTION_PAPER_ID);
    assert_eq!(paper.title.as_deref(), Some("Attention is All you Need"));
    assert_eq!(paper.year, Some(2017));
    assert!(
        paper.citation_count.unwrap_or(0) > 1000,
        "expected highly-cited paper"
    );
}

#[tokio::test]
async fn get_paper_by_arxiv_id() {
    let client = build_client();
    let paper = client
        .get_paper_by_arxiv("1706.03762")
        .send()
        .await
        .expect("get_paper_by_arxiv");

    assert_eq!(paper.paper_id, ATTENTION_PAPER_ID);
    assert!(
        paper
            .title
            .as_deref()
            .is_some_and(|t| t.to_lowercase().contains("attention")),
        "expected Attention paper, got: {:?}",
        paper.title
    );
}

#[tokio::test]
async fn get_paper_not_found() {
    let client = build_client();
    let result = client.get_paper("nonexistent_paper_id_12345").send().await;

    assert!(result.is_err(), "expected error for nonexistent paper");
    let err = result.unwrap_err();
    match &err {
        semantic_scholar::Error::Api { status, .. } => {
            assert_eq!(*status, 404, "expected 404, got {status}");
        }
        other => panic!("expected Api error, got: {other}"),
    }
}

#[tokio::test]
async fn get_citations() {
    let client = build_client();
    let resp = client
        .get_citations(ATTENTION_PAPER_ID)
        .limit(5)
        .send()
        .await
        .expect("get_citations");

    assert!(!resp.data.is_empty(), "expected citations for famous paper");
    for citation in &resp.data {
        assert!(
            citation.citing_paper.is_some(),
            "citingPaper should be present"
        );
    }
}

#[tokio::test]
async fn get_references() {
    let client = build_client();
    let resp = client
        .get_references(ATTENTION_PAPER_ID)
        .limit(5)
        .send()
        .await
        .expect("get_references");

    assert!(!resp.data.is_empty(), "expected references");
    for reference in &resp.data {
        assert!(
            reference.cited_paper.is_some(),
            "citedPaper should be present"
        );
    }
}

// ---------------------------------------------------------------------------
// Author endpoints
// ---------------------------------------------------------------------------

#[tokio::test]
async fn search_authors_returns_results() {
    let client = build_client();
    let resp = client
        .search_authors("Ashish Vaswani")
        .limit(3)
        .send()
        .await
        .expect("search_authors");

    assert!(resp.total > 0);
    assert!(!resp.data.is_empty());
}

#[tokio::test]
async fn get_author_by_id() {
    let client = build_client();
    let author = client
        .get_author(VASWANI_AUTHOR_ID)
        .send()
        .await
        .expect("get_author");

    assert_eq!(author.author_id, VASWANI_AUTHOR_ID);
    assert!(
        author
            .name
            .as_deref()
            .is_some_and(|n| n.contains("Vaswani")),
        "expected Vaswani, got: {:?}",
        author.name
    );
    assert!(
        author.paper_count.unwrap_or(0) > 0,
        "expected non-zero paper count"
    );
}

// ---------------------------------------------------------------------------
// Recommendations
// ---------------------------------------------------------------------------

#[tokio::test]
async fn get_recommendations() {
    let client = build_client();
    let result = client
        .get_recommendations(ATTENTION_PAPER_ID)
        .limit(3)
        .send()
        .await;

    // Recommendations endpoint may return empty or fail depending on API state.
    // We only verify the request completes and response deserializes correctly.
    match result {
        Ok(resp) => {
            // If successful, recommendedPapers should be a valid (possibly empty) vec
            assert!(resp.recommended_papers.len() <= 3);
        }
        Err(semantic_scholar::Error::Api { status, .. }) => {
            // Some papers may not have recommendations available
            assert!(
                status == 404 || status == 500,
                "unexpected status: {status}"
            );
        }
        Err(other) => panic!("unexpected error: {other}"),
    }
}

// ---------------------------------------------------------------------------
// Error handling (these don't hit the real API)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn invalid_api_key_header_rejected() {
    let result = SemanticScholar::with_api_key("bad\x00key");
    assert!(result.is_err(), "null byte in API key should fail");
    match result.unwrap_err() {
        semantic_scholar::Error::InvalidParameter(msg) => {
            assert!(msg.contains("header characters"), "got: {msg}");
        }
        other => panic!("expected InvalidParameter, got: {other}"),
    }
}

#[tokio::test]
async fn request_to_invalid_base_url_fails() {
    let mut client = SemanticScholar::new().expect("build client");
    client.set_base_url("http://localhost:1");

    let result = client.search_papers("test").limit(1).send().await;
    assert!(result.is_err(), "request to dead host should fail");
    match result.unwrap_err() {
        semantic_scholar::Error::Http { endpoint, .. } => {
            assert!(
                endpoint.contains("paper/search"),
                "endpoint context missing: {endpoint}"
            );
        }
        other => panic!("expected Http error, got: {other}"),
    }
}
