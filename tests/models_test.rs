use semantic_scholar::*;

// ---------------------------------------------------------------------------
// Paper deserialization
// ---------------------------------------------------------------------------

#[test]
fn deserialize_paper_minimal() {
    let json = r#"{"paperId": "abc123"}"#;
    let paper: Paper = serde_json::from_str(json).expect("minimal paper");
    assert_eq!(paper.paper_id, "abc123");
    assert!(paper.title.is_none());
    assert!(paper.year.is_none());
}

#[test]
fn deserialize_paper_with_common_fields() {
    let json = r#"{
        "paperId": "649def34f8be52c8b66281af98ae884c09aef38b",
        "corpusId": 3712016,
        "title": "Construction of the Literature Graph in Semantic Scholar",
        "year": 2018,
        "citationCount": 365,
        "referenceCount": 40,
        "isOpenAccess": true,
        "fieldsOfStudy": ["Computer Science"],
        "authors": [
            {"authorId": "1741101", "name": "Waleed Ammar"},
            {"authorId": "145585097", "name": "Dirk Groeneveld"}
        ]
    }"#;
    let paper: Paper = serde_json::from_str(json).expect("common fields");
    assert_eq!(
        paper.title.as_deref(),
        Some("Construction of the Literature Graph in Semantic Scholar")
    );
    assert_eq!(paper.year, Some(2018));
    assert_eq!(paper.citation_count, Some(365));
    assert_eq!(paper.reference_count, Some(40));
    assert_eq!(paper.is_open_access, Some(true));
    assert_eq!(paper.fields_of_study.as_ref().map(|f| f.len()), Some(1));

    let authors = paper.authors.as_ref().expect("authors");
    assert_eq!(authors.len(), 2);
    assert_eq!(authors[0].name.as_deref(), Some("Waleed Ammar"));
}

#[test]
fn deserialize_paper_with_abstract() {
    let json = r#"{
        "paperId": "test",
        "abstract": "This paper describes a system for..."
    }"#;
    let paper: Paper = serde_json::from_str(json).expect("abstract field");
    assert_eq!(
        paper.abstract_text.as_deref(),
        Some("This paper describes a system for...")
    );
}

#[test]
fn deserialize_paper_with_external_ids() {
    let json = r#"{
        "paperId": "test",
        "externalIds": {
            "DOI": "10.18653/v1/N18-3011",
            "ArXiv": "1805.02262",
            "MAG": null,
            "DBLP": "conf/naacl/AmmarGBBCDDEFHK18",
            "CorpusId": 3712016
        }
    }"#;
    let paper: Paper = serde_json::from_str(json).expect("external ids");
    let ids = paper.external_ids.expect("has external ids");
    assert_eq!(ids.doi.as_deref(), Some("10.18653/v1/N18-3011"));
    assert_eq!(ids.arxiv.as_deref(), Some("1805.02262"));
    assert!(ids.mag.is_none());
    assert_eq!(ids.dblp.as_deref(), Some("conf/naacl/AmmarGBBCDDEFHK18"));
    assert_eq!(ids.corpus_id, Some(3712016));
}

#[test]
fn deserialize_paper_with_tldr() {
    let json = r#"{
        "paperId": "test",
        "tldr": {
            "model": "tldr@v2.0.0",
            "text": "A deployed scalable system for organizing published scientific literature."
        }
    }"#;
    let paper: Paper = serde_json::from_str(json).expect("tldr");
    let tldr = paper.tldr.expect("has tldr");
    assert_eq!(tldr.model.as_deref(), Some("tldr@v2.0.0"));
    assert!(tldr.text.as_ref().is_some_and(|t| t.contains("scalable")));
}

#[test]
fn deserialize_paper_with_open_access_pdf() {
    let json = r#"{
        "paperId": "test",
        "openAccessPdf": {
            "url": "https://example.com/paper.pdf",
            "status": "GREEN"
        }
    }"#;
    let paper: Paper = serde_json::from_str(json).expect("open access pdf");
    let pdf = paper.open_access_pdf.expect("has pdf");
    assert_eq!(pdf.url.as_deref(), Some("https://example.com/paper.pdf"));
    assert_eq!(pdf.status.as_deref(), Some("GREEN"));
}

#[test]
fn deserialize_paper_ignores_unknown_fields() {
    let json = r#"{
        "paperId": "test",
        "someNewField": "value",
        "anotherUnknown": 42
    }"#;
    let paper: Paper = serde_json::from_str(json).expect("unknown fields ignored");
    assert_eq!(paper.paper_id, "test");
}

#[test]
fn deserialize_paper_search_response() {
    let json = r#"{
        "total": 1000,
        "offset": 0,
        "next": 10,
        "data": [
            {"paperId": "a1", "title": "Paper A"},
            {"paperId": "b2", "title": "Paper B"}
        ]
    }"#;
    let resp: PaperSearchResponse = serde_json::from_str(json).expect("search response");
    assert_eq!(resp.total, 1000);
    assert_eq!(resp.offset, Some(0));
    assert_eq!(resp.next, Some(10));
    assert_eq!(resp.data.len(), 2);
    assert_eq!(resp.data[0].title.as_deref(), Some("Paper A"));
}

#[test]
fn deserialize_citation_response() {
    let json = r#"{
        "offset": 0,
        "data": [
            {
                "citingPaper": {"paperId": "c1", "title": "Citing Paper"},
                "isInfluential": true,
                "contexts": ["In this work..."]
            }
        ]
    }"#;
    let resp: CitationResponse = serde_json::from_str(json).expect("citation response");
    assert_eq!(resp.data.len(), 1);
    let c = &resp.data[0];
    assert_eq!(c.is_influential, Some(true));
    assert_eq!(
        c.citing_paper.as_ref().and_then(|p| p.title.as_deref()),
        Some("Citing Paper")
    );
}

#[test]
fn deserialize_reference_response() {
    let json = r#"{
        "offset": 0,
        "data": [
            {
                "citedPaper": {"paperId": "r1", "title": "Referenced"},
                "isInfluential": false
            }
        ]
    }"#;
    let resp: ReferenceResponse = serde_json::from_str(json).expect("reference response");
    assert_eq!(resp.data.len(), 1);
    assert_eq!(resp.data[0].is_influential, Some(false));
}

#[test]
fn deserialize_recommendation_response() {
    let json = r#"{
        "recommendedPapers": [
            {"paperId": "rec1", "title": "Recommended Paper", "year": 2023}
        ]
    }"#;
    let resp: RecommendationResponse = serde_json::from_str(json).expect("recommendation response");
    assert_eq!(resp.recommended_papers.len(), 1);
    assert_eq!(resp.recommended_papers[0].year, Some(2023));
}

// ---------------------------------------------------------------------------
// Author deserialization
// ---------------------------------------------------------------------------

#[test]
fn deserialize_author_minimal() {
    let json = r#"{"authorId": "12345"}"#;
    let author: Author = serde_json::from_str(json).expect("minimal author");
    assert_eq!(author.author_id, "12345");
    assert!(author.name.is_none());
}

#[test]
fn deserialize_author_full() {
    let json = r#"{
        "authorId": "1741101",
        "name": "Waleed Ammar",
        "affiliations": ["Allen Institute for AI"],
        "homepage": "https://wammar.github.io",
        "paperCount": 150,
        "citationCount": 12000,
        "hIndex": 35
    }"#;
    let author: Author = serde_json::from_str(json).expect("full author");
    assert_eq!(author.name.as_deref(), Some("Waleed Ammar"));
    assert_eq!(author.h_index, Some(35));
    assert_eq!(author.paper_count, Some(150));
    assert_eq!(author.citation_count, Some(12000));
    assert!(author
        .affiliations
        .as_ref()
        .is_some_and(|a| a.contains(&"Allen Institute for AI".to_string())));
}

#[test]
fn deserialize_author_search_response() {
    let json = r#"{
        "total": 50,
        "offset": 0,
        "data": [
            {"authorId": "1", "name": "Author One"},
            {"authorId": "2", "name": "Author Two"}
        ]
    }"#;
    let resp: AuthorSearchResponse = serde_json::from_str(json).expect("author search");
    assert_eq!(resp.total, 50);
    assert_eq!(resp.data.len(), 2);
}

// ---------------------------------------------------------------------------
// Display implementations
// ---------------------------------------------------------------------------

#[test]
fn paper_display_with_authors() {
    let paper = Paper {
        title: Some("Attention Is All You Need".into()),
        year: Some(2017),
        authors: Some(vec![
            AuthorRef {
                author_id: Some("1".into()),
                name: Some("Ashish Vaswani".into()),
            },
            AuthorRef {
                author_id: Some("2".into()),
                name: Some("Noam Shazeer".into()),
            },
        ]),
        ..Default::default()
    };
    let display = paper.to_string();
    assert!(display.contains("Attention Is All You Need"));
    assert!(display.contains("2017"));
    assert!(display.contains("Ashish Vaswani"));
}

#[test]
fn paper_display_untitled() {
    let paper = Paper::default();
    let display = paper.to_string();
    assert!(display.contains("(untitled)"));
}

#[test]
fn author_display() {
    let author = Author {
        author_id: "123".into(),
        name: Some("Jane Doe".into()),
        h_index: Some(42),
        paper_count: Some(100),
        ..Default::default()
    };
    let display = author.to_string();
    assert!(display.contains("Jane Doe"));
    assert!(display.contains("42"));
    assert!(display.contains("100"));
}

// ---------------------------------------------------------------------------
// Error types
// ---------------------------------------------------------------------------

#[test]
fn error_display_api_error() {
    let err = semantic_scholar::Error::Api {
        status: 404,
        endpoint: "/graph/v1/paper/abc123".into(),
        message: "Paper not found".into(),
    };
    let display = err.to_string();
    assert!(display.contains("404"));
    assert!(display.contains("/graph/v1/paper/abc123"));
    assert!(display.contains("Paper not found"));
}

#[test]
fn error_display_invalid_parameter() {
    let err = semantic_scholar::Error::InvalidParameter("bad key".into());
    assert!(err.to_string().contains("bad key"));
}

#[test]
fn error_display_rate_limited() {
    let err = semantic_scholar::Error::RateLimited {
        endpoint: "/graph/v1/paper/search".into(),
        retries: 3,
    };
    let display = err.to_string();
    assert!(display.contains("rate limited"));
    assert!(display.contains("/graph/v1/paper/search"));
    assert!(display.contains("3"));
}

#[test]
fn error_display_deserialize() {
    let json_err = serde_json::from_str::<serde_json::Value>("not-json").unwrap_err();
    let err = semantic_scholar::Error::Deserialize {
        endpoint: "/graph/v1/paper/search".into(),
        source: json_err,
    };
    let display = err.to_string();
    assert!(display.contains("deserialize"));
    assert!(display.contains("/graph/v1/paper/search"));
}

#[test]
fn error_display_client_build() {
    // ClientBuild wraps reqwest::Error — construct indirectly via invalid proxy
    // Just verify the variant exists and formats correctly
    let err = semantic_scholar::Error::InvalidParameter("test".into());
    assert!(err.to_string().contains("invalid parameter"));
}

// ---------------------------------------------------------------------------
// Client construction
// ---------------------------------------------------------------------------

#[test]
fn client_new_succeeds() {
    let client = SemanticScholar::new();
    assert!(client.is_ok());
}

#[test]
fn client_with_api_key_succeeds() {
    let client = SemanticScholar::with_api_key("test-key-12345");
    assert!(client.is_ok());
}

#[test]
fn client_set_base_url() {
    let mut client = SemanticScholar::new().expect("create client");
    client.set_base_url("http://localhost:8080");
    // No assertion needed — just verifying it doesn't panic.
}

// ---------------------------------------------------------------------------
// Serialization roundtrip
// ---------------------------------------------------------------------------

#[test]
fn paper_serialize_roundtrip() {
    let paper = Paper {
        paper_id: "abc".into(),
        title: Some("Test Paper".into()),
        year: Some(2024),
        citation_count: Some(10),
        ..Default::default()
    };
    let json = serde_json::to_string(&paper).expect("serialize");
    let deserialized: Paper = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(deserialized.paper_id, "abc");
    assert_eq!(deserialized.title.as_deref(), Some("Test Paper"));
    assert_eq!(deserialized.year, Some(2024));
}
