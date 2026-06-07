/// Build the SPA index HTML by concatenating: head + style file + body markup
/// + script file + footer. CSS and JS live in their own files for IDE
/// support (syntax highlighting, lint). At compile time `include_str!`
/// embeds them as `&'static str`, so the runtime cost is one allocation
/// (the final concat into a String).
pub fn index_html() -> String {
    concat!(
        r##"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0, viewport-fit=cover">
<title>EchoFS</title>
<link rel="icon" type="image/svg+xml" href="data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='24' height='24' viewBox='0 0 24 24' fill='none'%3E%3Cpath d='M2 6a2 2 0 0 1 2-2h5l2 2h7a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V6z' stroke='%230071e3' stroke-width='1.8'/%3E%3Cpath d='M10 11a1.5 1.5 0 0 1 0 3' stroke='%230071e3' stroke-width='1.5' stroke-linecap='round'/%3E%3Cpath d='M12.5 9.5a4.5 4.5 0 0 1 0 6' stroke='%230071e3' stroke-width='1.5' stroke-linecap='round'/%3E%3Cpath d='M15 8a7 7 0 0 1 0 9' stroke='%230071e3' stroke-width='1.5' stroke-linecap='round'/%3E%3C/svg%3E">
<style>
"##,
        include_str!("template.css"),
        r##"</style>
</head>
<body>
<div class="toast-container" id="toastContainer"></div>
<div class="header">
  <div class="header-left">
    <a href="/" class="logo" data-nav><svg width="24" height="24" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg"><path d="M2 6a2 2 0 0 1 2-2h5l2 2h7a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V6z" stroke="currentColor" stroke-width="1.8" fill="none"/><path d="M10 11a1.5 1.5 0 0 1 0 3" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/><path d="M12.5 9.5a4.5 4.5 0 0 1 0 6" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/><path d="M15 8a7 7 0 0 1 0 9" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/></svg>EchoFS</a>
    <div class="breadcrumbs" id="breadcrumbs"></div>
  </div>
  <div class="header-controls">
    <button class="btn-upload" id="uploadBtn" style="display:none" title="Upload files">&#8593; Upload</button>
    <input type="file" id="uploadInput" multiple style="display:none">
    <button class="view-toggle" id="viewToggle" title="Toggle list/grid view"><svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="8" y1="6" x2="21" y2="6"/><line x1="8" y1="12" x2="21" y2="12"/><line x1="8" y1="18" x2="21" y2="18"/><line x1="3" y1="6" x2="3.01" y2="6"/><line x1="3" y1="12" x2="3.01" y2="12"/><line x1="3" y1="18" x2="3.01" y2="18"/></svg></button>
    <div style="position:relative">
      <button class="style-toggle" id="styleToggle" title="Switch theme style"><svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><path d="M12 2a10 10 0 0 1 0 20V2z" fill="currentColor" opacity="0.3"/><circle cx="8" cy="10" r="1.5" fill="currentColor" stroke="none"/><circle cx="12" cy="7" r="1.5" fill="currentColor" stroke="none"/><circle cx="16" cy="10" r="1.5" fill="currentColor" stroke="none"/><circle cx="14" cy="15" r="1.5" fill="currentColor" stroke="none"/></svg></button>
      <div class="style-menu" id="styleMenu">
        <button class="style-menu-item" data-style-choice="classic">
          <span class="check"></span>Classic
        </button>
        <button class="style-menu-item" data-style-choice="glass">
          <span class="check"></span>Liquid Glass
        </button>
        <button class="style-menu-item" data-style-choice="cartoon">
          <span class="check"></span>Cartoon
        </button>
      </div>
    </div>
    <button class="theme-toggle" id="themeToggle" title="Toggle light/dark mode">&#9790;</button>
  </div>
</div>
<div class="container">
  <div id="content"><div class="loading">Loading...</div></div>
</div>

<!-- Preview Modal -->
<div class="modal-overlay" id="modal">
  <div class="modal">
    <button class="modal-close" id="modalClose">&times;</button>
    <div class="modal-title" id="modalTitle"></div>
    <button class="nav-btn nav-prev" id="navPrev">&#10094;</button>
    <button class="nav-btn nav-next" id="navNext">&#10095;</button>
    <div id="modalContent"></div>
  </div>
</div>

<!-- QR Code Modal -->
<div class="modal-overlay" id="qrModal">
  <div class="modal">
    <button class="modal-close" id="qrModalClose">&times;</button>
    <div class="modal-title" id="qrModalTitle">QR Code</div>
    <div class="qr-modal" id="qrModalContent"></div>
  </div>
</div>

<!-- Auth Dialog -->
<div class="dialog-overlay" id="authDialog">
  <div class="dialog">
    <h3>Authentication Required</h3>
    <div class="dialog-error" id="authError"></div>
    <input class="dialog-input" id="authUser" type="text" placeholder="Username" autocomplete="username">
    <input class="dialog-input" id="authPass" type="password" placeholder="Password" autocomplete="current-password">
    <div class="dialog-buttons">
      <button class="dialog-btn" id="authCancel">Cancel</button>
      <button class="dialog-btn dialog-btn-primary" id="authSubmit">Login</button>
    </div>
  </div>
</div>

<!-- Rename Dialog -->
<div class="dialog-overlay" id="renameDialog">
  <div class="dialog">
    <h3>Rename</h3>
    <div class="dialog-error" id="renameError"></div>
    <input class="dialog-input" id="renameInput" type="text" placeholder="New name">
    <div class="dialog-buttons">
      <button class="dialog-btn" id="renameCancel">Cancel</button>
      <button class="dialog-btn dialog-btn-primary" id="renameSubmit">Rename</button>
    </div>
  </div>
</div>

<!-- Delete Confirm Dialog -->
<div class="dialog-overlay" id="deleteDialog">
  <div class="dialog">
    <h3>Delete</h3>
    <div class="dialog-msg" id="deleteMsg"></div>
    <div class="dialog-error" id="deleteError"></div>
    <div class="dialog-buttons">
      <button class="dialog-btn" id="deleteCancel">Cancel</button>
      <button class="dialog-btn dialog-btn-danger" id="deleteSubmit">Delete</button>
    </div>
  </div>
</div>

<!-- Move Dialog -->
<div class="dialog-overlay" id="moveDialog">
  <div class="dialog" style="max-width:420px">
    <h3>Move to...</h3>
    <div class="dialog-error" id="moveError"></div>
    <div class="move-browser" id="moveBrowser"></div>
    <div class="dialog-buttons">
      <button class="dialog-btn" id="moveCancel">Cancel</button>
      <button class="dialog-btn dialog-btn-primary" id="moveSubmit">Move Here</button>
    </div>
  </div>
</div>

<script src="https://cdn.jsdelivr.net/npm/qrcode-generator@1.4.4/qrcode.min.js"></script>
<script>
"##,
        include_str!("template.js"),
        r##"</script>
</body>
</html>"##
    )
    .to_string()
}

pub fn error_html(status_code: u16, title: &str, message: &str) -> String {
    format!(
        r##"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0, viewport-fit=cover">
<title>{status_code} {title} — EchoFS</title>
<link rel="icon" type="image/svg+xml" href="data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='24' height='24' viewBox='0 0 24 24' fill='none'%3E%3Cpath d='M2 6a2 2 0 0 1 2-2h5l2 2h7a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V6z' stroke='%230071e3' stroke-width='1.8'/%3E%3Cpath d='M10 11a1.5 1.5 0 0 1 0 3' stroke='%230071e3' stroke-width='1.5' stroke-linecap='round'/%3E%3Cpath d='M12.5 9.5a4.5 4.5 0 0 1 0 6' stroke='%230071e3' stroke-width='1.5' stroke-linecap='round'/%3E%3Cpath d='M15 8a7 7 0 0 1 0 9' stroke='%230071e3' stroke-width='1.5' stroke-linecap='round'/%3E%3C/svg%3E">
<style>
:root {{
  --bg: #ffffff;
  --text: #1d1d1f;
  --text-secondary: #6e6e73;
  --border: #d2d2d7;
  --accent: #0071e3;
  --header-bg: rgba(251, 251, 253, 0.72);
}}
@media (prefers-color-scheme: dark) {{
  :root {{
    --bg: #1d1d1f;
    --text: #f5f5f7;
    --text-secondary: #a1a1a6;
    --border: #424245;
    --accent: #2997ff;
    --header-bg: rgba(37, 37, 39, 0.72);
  }}
}}
* {{ margin:0; padding:0; box-sizing:border-box; }}
body {{
  font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
  background: var(--bg);
  color: var(--text);
  line-height: 1.5;
  min-height: 100vh;
}}
.header {{
  background: var(--header-bg);
  -webkit-backdrop-filter: saturate(180%) blur(20px);
  backdrop-filter: saturate(180%) blur(20px);
  border-bottom: 1px solid var(--border);
  padding: 12px 24px;
  display: flex;
  align-items: center;
  min-height: 48px;
}}
.logo {{
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 20px;
  font-weight: 700;
  color: var(--accent);
  text-decoration: none;
}}
.error-container {{
  max-width: 600px;
  margin: 0 auto;
  padding: 80px 24px;
  text-align: center;
}}
.error-code {{
  font-size: 72px;
  font-weight: 700;
  color: var(--text-secondary);
  line-height: 1;
  margin-bottom: 8px;
}}
.error-title {{
  font-size: 24px;
  font-weight: 600;
  margin-bottom: 12px;
}}
.error-message {{
  font-size: 16px;
  color: var(--text-secondary);
  margin-bottom: 32px;
}}
.back-link {{
  display: inline-block;
  color: var(--accent);
  text-decoration: none;
  font-size: 16px;
  padding: 10px 24px;
  border: 1px solid var(--accent);
  border-radius: 8px;
  transition: all 0.15s;
}}
.back-link:hover {{
  background: var(--accent);
  color: #fff;
}}
</style>
</head>
<body>
<div class="header">
  <a href="/" class="logo"><svg width="24" height="24" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg"><path d="M2 6a2 2 0 0 1 2-2h5l2 2h7a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V6z" stroke="currentColor" stroke-width="1.8" fill="none"/><path d="M10 11a1.5 1.5 0 0 1 0 3" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/><path d="M12.5 9.5a4.5 4.5 0 0 1 0 6" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/><path d="M15 8a7 7 0 0 1 0 9" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/></svg>EchoFS</a>
</div>
<div class="error-container">
  <div class="error-code">{status_code}</div>
  <div class="error-title">{title}</div>
  <div class="error-message">{message}</div>
  <a href="/" class="back-link">Back to Home</a>
</div>
</body>
</html>"##,
        status_code = status_code,
        title = html_escape(title),
        message = html_escape(message),
    )
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_with_doctype() {
        assert!(index_html().starts_with("<!DOCTYPE html>"));
    }

    #[test]
    fn contains_echofs() {
        assert!(index_html().contains("EchoFS"));
    }

    #[test]
    fn contains_xhr_header() {
        assert!(index_html().contains("X-Requested-With"));
    }

    #[test]
    fn contains_dynamic_title_update() {
        let html = index_html();
        assert!(html.contains("document.title"));
        assert!(html.contains("EchoFS"));
    }

    #[test]
    fn contains_grid_view() {
        let html = index_html();
        // Grid CSS class is shipped (used at runtime when viewMode='grid')
        assert!(html.contains(".file-grid"));
        // Render functions for all three views are present (rendered on demand)
        assert!(html.contains("renderGrid"));
        assert!(html.contains("renderTable"));
        assert!(html.contains("renderCardList"));
        // Single-view dispatcher
        assert!(html.contains("currentRenderMode"));
        // View toggle button + persistence key
        assert!(html.contains("id=\"viewToggle\""));
        assert!(html.contains("echofs-view"));
    }

    #[test]
    fn folder_icon_uses_card_index_dividers_emoji() {
        let html = index_html();
        // 🗂️ is U+1F5C2 (Card Index Dividers) + U+FE0F (variation selector).
        // After splitting the JS into template.js, the emoji is stored as
        // real UTF-8 bytes, so we check for the literal codepoint.
        assert!(html.contains('\u{1F5C2}'), "expected 🗂️ folder emoji in template");
        // Old SVG helper should be gone.
        assert!(!html.contains("function folderSvg"));
    }

    /// P3 #15: CSS and JS were extracted into template.css and template.js;
    /// the Rust source now glues them with concat!() + include_str!().
    /// This test verifies the composition is intact — key markers from each
    /// segment must all appear in the final HTML, in the right order.
    #[test]
    fn css_and_js_are_composed_via_include_str() {
        // Normalize CRLF → LF so the test works on Windows CI where
        // actions/checkout converts line endings.
        let html = index_html().replace("\r\n", "\n");

        // Order check: <style> opens BEFORE the JS IIFE, which opens BEFORE </body>.
        let style_open = html.find("<style>").expect("missing <style>");
        let css_marker = html
            .find(":root {")
            .expect("missing CSS marker `:root {` — template.css not embedded?");
        let style_close = html.find("</style>").expect("missing </style>");
        let script_open = html
            .find("<script>\n(function() {")
            .expect("missing JS IIFE — template.js not embedded?");
        let script_close = html.find("})();\n</script>").expect("missing </script>");

        assert!(style_open < css_marker, "CSS body must follow <style>");
        assert!(css_marker < style_close, "CSS body must precede </style>");
        assert!(style_close < script_open, "</style> must precede <script>");
        assert!(script_open < script_close, "<script> must precede </script>");

        // Key markers from template.js prove the JS file got embedded fully.
        for marker in ["currentRenderMode", "STATUS_MSG", "UPLOAD_CONCURRENCY", "data-action"] {
            assert!(html.contains(marker), "missing JS marker `{marker}`");
        }
        // Key markers from template.css.
        for marker in [".file-grid", ".grid-tile", "data-style=\"glass\"", "@keyframes toastIn"] {
            assert!(html.contains(marker), "missing CSS marker `{marker}`");
        }
    }

    /// Regression test for P0 XSS — escHtml must NOT use the textContent/innerHTML
    /// DOM round-trip (which leaves `"` and `'` unencoded), and inline onclick
    /// handlers should be replaced by event delegation via data-action.
    #[test]
    fn no_inline_onclick_handlers_in_render_paths() {
        let html = index_html();
        // The render paths used to emit `onclick="..."` strings into the HTML.
        // After the event-delegation refactor, none of those should remain.
        // (The HTML markup itself has no onclick attributes either — we use
        // addEventListener for header buttons and document-level delegation
        // for everything else.)
        assert!(
            !html.contains("onclick=\""),
            "found inline onclick handler — should use data-action delegation"
        );
        // Old escJs helper that powered the inline-onclick pattern should be gone.
        assert!(!html.contains("function escJs"));
    }

    #[test]
    fn esc_html_encodes_quotes_and_apostrophes() {
        let html = index_html();
        // The new escHtml must replace ", ', &, <, > — verify all five replacement
        // patterns are present in the JS source.
        assert!(html.contains("&amp;"), "expected & encoding in escHtml");
        assert!(html.contains("&lt;"), "expected < encoding in escHtml");
        assert!(html.contains("&gt;"), "expected > encoding in escHtml");
        assert!(html.contains("&quot;"), "expected \" encoding in escHtml");
        assert!(html.contains("&#39;"), "expected ' encoding in escHtml");
    }

    #[test]
    fn event_delegation_uses_data_action() {
        let html = index_html();
        // Every action keyword has a dispatcher branch. These are static strings in
        // the document click handler and are guaranteed to exist in the bundled JS.
        for action in ["preview", "copy", "qr", "download", "rename", "move", "delete", "more-toggle", "move-browse-to"] {
            let needle = format!("case '{}':", action);
            assert!(html.contains(&needle), "missing dispatcher case '{action}'");
        }
        // Static buttons (primary actions and "more" toggle) emit data-action literally.
        for action in ["preview", "copy", "qr", "more-toggle", "move-browse-to"] {
            let needle = format!("data-action=\"{}\"", action);
            assert!(html.contains(&needle), "missing data-action=\"{action}\" in render code");
        }
        // Dynamic action items pushed into the more menu — the action name appears
        // as a JS string literal next to the label.
        for action in ["rename", "move", "delete", "download"] {
            let needle = format!("action: '{}'", action);
            assert!(html.contains(&needle), "missing action: '{action}' in more-menu items");
        }
        // Globals that previously polluted window.* should be gone.
        assert!(!html.contains("window.copyLink"));
        assert!(!html.contains("window.previewMedia"));
        assert!(!html.contains("window.showQr"));
    }

    /// P2 #8: HTTP/2 strips reason-phrase, so resp.statusText is empty and the
    /// previous code surfaced "Failed: " toasts. Status codes must be mapped
    /// to readable text via a shared helper.
    #[test]
    fn status_messages_are_status_code_based() {
        let html = index_html();
        assert!(html.contains("function statusMessage"));
        // A representative sampling of mapped status codes.
        for code in ["401", "403", "404", "409", "423", "507"] {
            assert!(html.contains(code), "missing status code mapping {code}");
        }
        // The old fallback pattern should be gone from all 3 call sites.
        assert!(!html.contains("resp.statusText || 'Failed'"));
    }

    /// P2 #10: dragenter/dragleave fire on every child boundary — a counter
    /// must guard the active class so the outline doesn't flicker.
    #[test]
    fn drag_uses_depth_counter() {
        let html = index_html();
        // The fix introduces a dragDepth variable AND a dragenter handler.
        assert!(html.contains("dragDepth"));
        assert!(html.contains("'dragenter'"));
    }

    /// P2 #9: showToast must cap visible toasts so 50-file uploads don't
    /// stack 50 toasts off-screen.
    #[test]
    fn toast_stack_is_capped() {
        let html = index_html();
        assert!(html.contains("MAX_TOASTS"));
    }

    /// P2 #7: clicking a sort header should reorder in place, not jump to the
    /// top of a 1000-row list. The handler must save and restore window.scrollY.
    #[test]
    fn sort_preserves_scroll() {
        let html = index_html();
        assert!(html.contains("savedScroll"));
        assert!(html.contains("window.scrollTo(0, savedScroll)"));
    }

    /// P1 #5: uploads must run in parallel via a worker pool, not a serial
    /// for-await loop.
    #[test]
    fn uploads_use_concurrency_pool() {
        let html = index_html();
        assert!(html.contains("UPLOAD_CONCURRENCY"));
        // The worker pattern uses Promise.all over an array of workers.
        assert!(html.contains("Promise.all(workers)"));
    }

    #[test]
    fn error_html_contains_status_code() {
        let html = error_html(404, "Not Found", "The page was not found");
        assert!(html.contains("404"));
        assert!(html.contains("Not Found"));
        assert!(html.contains("The page was not found"));
    }

    #[test]
    fn error_html_starts_with_doctype() {
        let html = error_html(500, "Internal Error", "Something went wrong");
        assert!(html.starts_with("<!DOCTYPE html>"));
    }

    #[test]
    fn error_html_contains_back_link() {
        let html = error_html(403, "Forbidden", "Access denied");
        assert!(html.contains("Back to Home"));
        assert!(html.contains("href=\"/\""));
    }

    #[test]
    fn error_html_escapes_xss() {
        let html = error_html(400, "<script>alert(1)</script>", "test &\"quotes\"");
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(html.contains("&lt;script&gt;"));
        assert!(html.contains("&amp;&quot;quotes&quot;"));
    }

    #[test]
    fn error_html_contains_echofs() {
        let html = error_html(404, "Not Found", "msg");
        assert!(html.contains("EchoFS"));
    }
}
