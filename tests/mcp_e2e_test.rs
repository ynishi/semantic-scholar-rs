//! MCP server end-to-end tests.
//!
//! Spawns the `semantic-scholar-mcp` binary and communicates via JSON-RPC over stdio.
//! API-calling tests are gated behind `integration-test`.
//!
//! ```sh
//! # Protocol-only tests (no API calls):
//! cargo test --features mcp --test mcp_e2e_test
//!
//! # Full E2E including API calls:
//! cargo test --features mcp,integration-test --test mcp_e2e_test -- --test-threads=1
//! ```
#![cfg(feature = "mcp")]
#![cfg_attr(not(feature = "integration-test"), allow(dead_code))]

use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Child, ChildStdin, Command, Stdio};

// ---------------------------------------------------------------------------
// Test harness
// ---------------------------------------------------------------------------

struct McpClient {
    child: Child,
    stdin: ChildStdin,
    reader: BufReader<std::process::ChildStdout>,
    next_id: u64,
}

impl McpClient {
    fn spawn() -> Self {
        let binary = Self::binary_path();
        assert!(
            binary.exists(),
            "MCP binary not found at {binary:?}. Run: cargo build --features mcp"
        );

        let mut child = Command::new(&binary)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .expect("spawn MCP server");

        let stdin = child.stdin.take().expect("stdin");
        let stdout = child.stdout.take().expect("stdout");

        Self {
            child,
            stdin,
            reader: BufReader::new(stdout),
            next_id: 1,
        }
    }

    fn binary_path() -> PathBuf {
        // cargo test sets OUT_DIR; derive target dir from CARGO_MANIFEST_DIR
        let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        manifest
            .join("target")
            .join("debug")
            .join("semantic-scholar-mcp")
    }

    fn initialize(&mut self) -> Value {
        let resp = self.request(
            "initialize",
            json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": { "name": "e2e-test", "version": "0.1.0" }
            }),
        );
        self.notify("notifications/initialized", json!({}));
        resp
    }

    fn request(&mut self, method: &str, params: Value) -> Value {
        let id = self.next_id;
        self.next_id += 1;

        let msg = json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params,
        });
        self.send_line(&msg);

        // Read lines until we find a response matching our id
        loop {
            let line = self.read_line();
            let v: Value = serde_json::from_str(&line)
                .unwrap_or_else(|e| panic!("invalid JSON from server: {e}\nraw: {line}"));
            if v.get("id").and_then(|v| v.as_u64()) == Some(id) {
                return v;
            }
            // Otherwise it's a notification — skip it
        }
    }

    fn notify(&mut self, method: &str, params: Value) {
        let msg = json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
        });
        self.send_line(&msg);
    }

    fn call_tool(&mut self, name: &str, arguments: Value) -> Value {
        let resp = self.request(
            "tools/call",
            json!({ "name": name, "arguments": arguments }),
        );
        resp["result"].clone()
    }

    fn send_line(&mut self, v: &Value) {
        let mut line = serde_json::to_string(v).expect("serialize");
        line.push('\n');
        self.stdin
            .write_all(line.as_bytes())
            .expect("write to stdin");
        self.stdin.flush().expect("flush stdin");
    }

    fn read_line(&mut self) -> String {
        let mut buf = String::new();
        self.reader.read_line(&mut buf).expect("read from stdout");
        assert!(!buf.is_empty(), "server closed stdout unexpectedly");
        buf
    }
}

impl Drop for McpClient {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// Extract text content from a CallToolResult.
fn result_text(result: &Value) -> &str {
    result["content"][0]["text"]
        .as_str()
        .expect("expected text content in tool result")
}

// ---------------------------------------------------------------------------
// Protocol tests (no API calls)
// ---------------------------------------------------------------------------

#[test]
fn mcp_initialize_returns_server_info() {
    let mut client = McpClient::spawn();
    let resp = client.initialize();

    let result = &resp["result"];
    assert_eq!(
        result["protocolVersion"].as_str(),
        Some("2024-11-05"),
        "protocol version mismatch"
    );
    assert_eq!(
        result["serverInfo"]["name"].as_str(),
        Some("semantic-scholar-mcp")
    );
    assert!(
        result["capabilities"]["tools"].is_object(),
        "tools capability should be present"
    );
}

#[test]
fn mcp_tools_list_returns_all_tools() {
    let mut client = McpClient::spawn();
    client.initialize();

    let resp = client.request("tools/list", json!({}));
    let tools = resp["result"]["tools"]
        .as_array()
        .expect("tools should be array");

    let tool_names: Vec<&str> = tools.iter().filter_map(|t| t["name"].as_str()).collect();

    let expected = [
        "search_papers",
        "get_paper",
        "search_authors",
        "get_author",
        "get_citations",
        "get_references",
        "get_recommendations",
    ];

    for name in &expected {
        assert!(
            tool_names.contains(name),
            "missing tool: {name}. found: {tool_names:?}"
        );
    }
    assert_eq!(
        tool_names.len(),
        expected.len(),
        "unexpected extra tools: {tool_names:?}"
    );
}

#[test]
fn mcp_tools_have_descriptions_and_schemas() {
    let mut client = McpClient::spawn();
    client.initialize();

    let resp = client.request("tools/list", json!({}));
    let tools = resp["result"]["tools"].as_array().expect("tools array");

    for tool in tools {
        let name = tool["name"].as_str().unwrap_or("(unknown)");
        assert!(
            tool["description"].is_string(),
            "tool {name} missing description"
        );
        assert!(
            tool["inputSchema"].is_object(),
            "tool {name} missing inputSchema"
        );
    }
}

// ---------------------------------------------------------------------------
// Tool call E2E tests (hit real API)
// ---------------------------------------------------------------------------

#[cfg(feature = "integration-test")]
mod api {
    use super::*;

    const ATTENTION_PAPER_ID: &str = "204e3073870fae3d05bcbc2f6a8e263d9b72e776";
    const VASWANI_AUTHOR_ID: &str = "40348417";

    #[test]
    fn tool_search_papers() {
        let mut client = McpClient::spawn();
        client.initialize();

        let result = client.call_tool(
            "search_papers",
            json!({ "query": "attention is all you need", "limit": 3 }),
        );

        let text = result_text(&result);
        assert!(text.contains("Found"), "expected 'Found' in: {text}");
        assert!(
            text.contains("Attention"),
            "expected 'Attention' in results: {text}"
        );
    }

    #[test]
    fn tool_get_paper() {
        let mut client = McpClient::spawn();
        client.initialize();

        let result = client.call_tool("get_paper", json!({ "paper_id": ATTENTION_PAPER_ID }));

        let text = result_text(&result);
        assert!(text.contains("Attention"), "expected title: {text}");
        assert!(text.contains("2017"), "expected year: {text}");
        assert!(text.contains("Citations"), "expected citation info: {text}");
    }

    #[test]
    fn tool_get_paper_not_found() {
        let mut client = McpClient::spawn();
        client.initialize();

        let resp = client.request(
            "tools/call",
            json!({
                "name": "get_paper",
                "arguments": { "paper_id": "nonexistent_paper_99999" }
            }),
        );

        // MCP tools return errors via ErrorData, not tool result
        assert!(
            resp.get("error").is_some(),
            "expected error response for nonexistent paper, got: {resp}"
        );
    }

    #[test]
    fn tool_search_authors() {
        let mut client = McpClient::spawn();
        client.initialize();

        let result = client.call_tool(
            "search_authors",
            json!({ "query": "Ashish Vaswani", "limit": 3 }),
        );

        let text = result_text(&result);
        assert!(
            text.contains("Found"),
            "expected 'Found' in author search: {text}"
        );
    }

    #[test]
    fn tool_get_author() {
        let mut client = McpClient::spawn();
        client.initialize();

        let result = client.call_tool("get_author", json!({ "author_id": VASWANI_AUTHOR_ID }));

        let text = result_text(&result);
        assert!(text.contains("Vaswani"), "expected author name: {text}");
        assert!(text.contains("h-index"), "expected h-index: {text}");
    }

    #[test]
    fn tool_get_citations() {
        let mut client = McpClient::spawn();
        client.initialize();

        let result = client.call_tool(
            "get_citations",
            json!({ "paper_id": ATTENTION_PAPER_ID, "limit": 3 }),
        );

        let text = result_text(&result);
        assert!(
            text.contains("citing papers"),
            "expected citation list: {text}"
        );
    }

    #[test]
    fn tool_get_references() {
        let mut client = McpClient::spawn();
        client.initialize();

        let result = client.call_tool(
            "get_references",
            json!({ "paper_id": ATTENTION_PAPER_ID, "limit": 3 }),
        );

        let text = result_text(&result);
        assert!(
            text.contains("referenced papers"),
            "expected reference list: {text}"
        );
    }

    #[test]
    fn tool_get_recommendations() {
        let mut client = McpClient::spawn();
        client.initialize();

        let result = client.call_tool(
            "get_recommendations",
            json!({ "paper_id": ATTENTION_PAPER_ID, "limit": 3 }),
        );

        // Recommendations may return empty, but should not error
        let text = result_text(&result);
        assert!(
            text.contains("recommended papers"),
            "expected recommendation list: {text}"
        );
    }
}
