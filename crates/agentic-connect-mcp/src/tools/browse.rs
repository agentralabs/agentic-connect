//! Browser tools — Inventions 5-7: Browser Consciousness, Scraping, Forms.

use serde_json::{json, Value};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::session::SessionManager;
use crate::types::{McpResult, ToolCallResult, ToolDefinition};

pub fn definitions() -> Vec<ToolDefinition> {
    vec![
        // Invention 5: Browser Consciousness
        def("connect_browse_navigate", "Navigate to URL with full page load", json!({
            "type": "object", "properties": {
                "url": { "type": "string", "description": "URL to navigate to" },
                "wait_for": { "type": "string", "description": "CSS selector or text to wait for" }
            }, "required": ["url"]
        })),
        def("connect_browse_understand", "Get semantic understanding of current page", json!({
            "type": "object", "properties": {
                "depth": { "type": "string", "enum": ["summary","structure","full"], "default": "summary" }
            }
        })),
        def("connect_browse_interact", "Interact with page by intent", json!({
            "type": "object", "properties": {
                "intent": { "type": "string", "description": "What to do: 'click login', 'fill email'" },
                "value": { "type": "string", "description": "Value to fill (for fill intents)" }
            }, "required": ["intent"]
        })),
        def("connect_browse_state", "Get current page state including forms and content", json!({
            "type": "object", "properties": {}
        })),
        def("connect_browse_screenshot", "Capture page screenshot", json!({
            "type": "object", "properties": {
                "selector": { "type": "string", "description": "CSS selector for element screenshot" },
                "full_page": { "type": "boolean", "default": false }
            }
        })),
        def("connect_browse_wait", "Wait for condition on page", json!({
            "type": "object", "properties": {
                "condition": { "type": "string", "enum": ["element","text","navigation","idle"], "description": "What to wait for" },
                "value": { "type": "string", "description": "Selector or text to match" },
                "timeout_ms": { "type": "integer", "default": 10000 }
            }, "required": ["condition"]
        })),
        // Invention 6: Web Scraping Intelligence
        def("connect_scrape_extract", "Extract structured data from page", json!({
            "type": "object", "properties": {
                "url": { "type": "string" },
                "selectors": { "type": "object", "description": "Named CSS selectors for extraction" }
            }, "required": ["url"]
        })),
        def("connect_scrape_monitor", "Monitor page for changes", json!({
            "type": "object", "properties": {
                "url": { "type": "string" },
                "selector": { "type": "string" },
                "interval_secs": { "type": "integer", "default": 3600 }
            }, "required": ["url"]
        })),
        def("connect_scrape_adapt", "Adapt extractors to page layout changes", json!({
            "type": "object", "properties": {
                "url": { "type": "string" },
                "previous_selector": { "type": "string" },
                "expected_content": { "type": "string" }
            }, "required": ["url"]
        })),
        def("connect_scrape_batch", "Scrape multiple pages with throttling", json!({
            "type": "object", "properties": {
                "urls": { "type": "array", "items": { "type": "string" } },
                "selectors": { "type": "object" },
                "delay_ms": { "type": "integer", "default": 1000 }
            }, "required": ["urls"]
        })),
        def("connect_scrape_history", "View extraction history and detected changes", json!({
            "type": "object", "properties": {
                "url": { "type": "string" },
                "limit": { "type": "integer", "default": 20 }
            }, "required": ["url"]
        })),
        // Invention 7: Form Automation
        def("connect_form_analyze", "Analyze form structure and field types", json!({
            "type": "object", "properties": {
                "url": { "type": "string" },
                "form_selector": { "type": "string", "description": "CSS selector for form" }
            }, "required": ["url"]
        })),
        def("connect_form_fill", "Fill form fields with data", json!({
            "type": "object", "properties": {
                "data": { "type": "object", "description": "Field name → value mapping" }
            }, "required": ["data"]
        })),
        def("connect_form_submit", "Submit form and capture result", json!({
            "type": "object", "properties": {
                "form_selector": { "type": "string" },
                "wait_for_navigation": { "type": "boolean", "default": true }
            }
        })),
        def("connect_form_wizard", "Navigate multi-step form flow", json!({
            "type": "object", "properties": {
                "steps": { "type": "array", "items": { "type": "object" }, "description": "Array of {data, submit} per step" }
            }, "required": ["steps"]
        })),
        def("connect_form_validate", "Check form state for validation errors", json!({
            "type": "object", "properties": {}
        })),
    ]
}

pub async fn try_execute(
    name: &str,
    args: Value,
    _session: &Arc<Mutex<SessionManager>>,
) -> Option<McpResult<ToolCallResult>> {
    if name.starts_with("connect_browse_") || name.starts_with("connect_scrape_") || name.starts_with("connect_form_") {
        Some(Ok(ToolCallResult::json(&json!({
            "tool": name,
            "status": "requires_browser_feature",
            "hint": "Browser tools require the 'browser' feature and a headless Chrome installation"
        }))))
    } else {
        None
    }
}

fn def(name: &str, desc: &str, schema: Value) -> ToolDefinition {
    ToolDefinition { name: name.into(), description: Some(desc.into()), input_schema: schema }
}
