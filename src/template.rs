pub fn index_html() -> String {
    r##"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0, viewport-fit=cover">
<title>EchoFS</title>
<link rel="icon" type="image/svg+xml" href="data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='24' height='24' viewBox='0 0 24 24' fill='none'%3E%3Cpath d='M2 6a2 2 0 0 1 2-2h5l2 2h7a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V6z' stroke='%230071e3' stroke-width='1.8'/%3E%3Cpath d='M10 11a1.5 1.5 0 0 1 0 3' stroke='%230071e3' stroke-width='1.5' stroke-linecap='round'/%3E%3Cpath d='M12.5 9.5a4.5 4.5 0 0 1 0 6' stroke='%230071e3' stroke-width='1.5' stroke-linecap='round'/%3E%3Cpath d='M15 8a7 7 0 0 1 0 9' stroke='%230071e3' stroke-width='1.5' stroke-linecap='round'/%3E%3C/svg%3E">
<style>
/* ===== Classic Theme (default) ===== */
:root {
  --bg: #ffffff;
  --bg-secondary: #f5f5f7;
  --text: #1d1d1f;
  --text-secondary: #6e6e73;
  --border: #d2d2d7;
  --accent: #0071e3;

  --hover-bg: #f0f0f5;
  --modal-bg: rgba(0,0,0,0.6);
  --card-bg: #ffffff;
  --shadow: 0 2px 12px rgba(0,0,0,0.08);
  --header-bg: rgba(251, 251, 253, 0.72);
  --visited: #6e5494;
  --body-bg: var(--bg);
  --glass-border: transparent;
  --glass-highlight: transparent;
  --glass-radius-sm: 6px;
  --glass-radius-md: 8px;
  --glass-radius-lg: 12px;
}
[data-mode="dark"] {
  --bg: #1d1d1f;
  --bg-secondary: #2d2d2f;
  --text: #f5f5f7;
  --text-secondary: #a1a1a6;
  --border: #424245;
  --accent: #2997ff;

  --hover-bg: #2d2d30;
  --card-bg: #2d2d2f;
  --shadow: 0 2px 12px rgba(0,0,0,0.3);
  --header-bg: rgba(37, 37, 39, 0.72);
  --visited: #9b8ab8;
  --body-bg: var(--bg);
}

/* ===== Liquid Glass Theme — Light ===== */
[data-style="glass"] {
  --bg: rgba(255, 255, 255, 0.45);
  --bg-secondary: rgba(255, 255, 255, 0.35);
  --text: #1a1a2e;
  --text-secondary: #4a4a6a;
  --border: rgba(255, 255, 255, 0.5);
  --accent: #0071e3;

  --hover-bg: rgba(255, 255, 255, 0.4);
  --modal-bg: rgba(0, 0, 0, 0.4);
  --card-bg: rgba(255, 255, 255, 0.5);
  --shadow: 0 8px 32px rgba(0, 0, 0, 0.08), 0 2px 8px rgba(0, 0, 0, 0.04);
  --header-bg: rgba(255, 255, 255, 0.45);
  --visited: #6e5494;
  --body-bg: linear-gradient(135deg, #e0e8ff 0%, #f5e6ff 25%, #ffe6f0 50%, #e6fff5 75%, #e6f0ff 100%);
  --glass-border: rgba(255, 255, 255, 0.6);
  --glass-highlight: linear-gradient(135deg, rgba(255,255,255,0.7) 0%, rgba(255,255,255,0.1) 50%, transparent 100%);
  --glass-radius-sm: 12px;
  --glass-radius-md: 16px;
  --glass-radius-lg: 24px;
}

/* ===== Liquid Glass Theme — Dark ===== */
[data-style="glass"][data-mode="dark"] {
  --bg: rgba(30, 30, 40, 0.55);
  --bg-secondary: rgba(40, 40, 55, 0.5);
  --text: #f0f0ff;
  --text-secondary: #a8a8c8;
  --border: rgba(255, 255, 255, 0.12);
  --accent: #64b5ff;

  --hover-bg: rgba(255, 255, 255, 0.08);
  --modal-bg: rgba(0, 0, 0, 0.6);
  --card-bg: rgba(40, 40, 55, 0.6);
  --shadow: 0 8px 32px rgba(0, 0, 0, 0.3), 0 2px 8px rgba(0, 0, 0, 0.2);
  --header-bg: rgba(25, 25, 35, 0.55);
  --visited: #b8a0d8;
  --body-bg: linear-gradient(135deg, #1a1a2e 0%, #16213e 25%, #0f3460 50%, #1a1a3e 75%, #2d1b4e 100%);
  --glass-border: rgba(255, 255, 255, 0.15);
  --glass-highlight: linear-gradient(135deg, rgba(255,255,255,0.15) 0%, rgba(255,255,255,0.03) 50%, transparent 100%);
}

/* ===== Cartoon Theme — Light ===== */
[data-style="cartoon"] {
  --bg: #fffdf7;
  --bg-secondary: #fff3d6;
  --text: #2d2d2d;
  --text-secondary: #666666;
  --border: #2d2d2d;
  --accent: #ff6b6b;

  --hover-bg: #fff0c8;
  --modal-bg: rgba(0, 0, 0, 0.5);
  --card-bg: #ffffff;
  --shadow: 3px 3px 0 #2d2d2d;
  --header-bg: #a8e6cf;
  --visited: #9b59b6;
  --body-bg: #fffdf7;
  --glass-border: transparent;
  --glass-highlight: transparent;
  --glass-radius-sm: 12px;
  --glass-radius-md: 16px;
  --glass-radius-lg: 24px;
  --cartoon-border: 2.5px solid #2d2d2d;
  --cartoon-shadow: 3px 3px 0 #2d2d2d;
  --cartoon-shadow-sm: 2px 2px 0 #2d2d2d;
}

/* ===== Cartoon Theme — Dark ===== */
[data-style="cartoon"][data-mode="dark"] {
  --bg: #1a1a2e;
  --bg-secondary: #2a2a4a;
  --text: #f8f8f8;
  --text-secondary: #b0b0c0;
  --border: #f8f8f8;
  --accent: #ff8a80;

  --hover-bg: #3a3a5a;
  --modal-bg: rgba(0, 0, 0, 0.7);
  --card-bg: #2a2a4a;
  --shadow: 3px 3px 0 rgba(255,255,255,0.3);
  --header-bg: #2e4a3e;
  --visited: #ce93d8;
  --body-bg: #1a1a2e;
  --cartoon-border: 2.5px solid rgba(255,255,255,0.7);
  --cartoon-shadow: 3px 3px 0 rgba(255,255,255,0.2);
  --cartoon-shadow-sm: 2px 2px 0 rgba(255,255,255,0.2);
}
* { margin:0; padding:0; box-sizing:border-box; }
body {
  font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
  background: var(--body-bg);
  color: var(--text);
  line-height: 1.5;
  min-height: 100vh;
  -webkit-text-size-adjust: 100%;
}

/* ===== Liquid Glass Background (static, GPU-friendly) ===== */
[data-style="glass"] body {
  position: relative;
  background: var(--body-bg);
  background-attachment: fixed;
}

/* ===== Glass Panels — only key surfaces get backdrop-filter ===== */
[data-style="glass"] .header {
  -webkit-backdrop-filter: blur(20px) saturate(180%);
  backdrop-filter: blur(20px) saturate(180%);
  border-bottom: 1px solid var(--glass-border);
  background: var(--header-bg);
  box-shadow: var(--shadow), inset 0 1px 0 var(--glass-border);
}
[data-style="glass"] .file-table {
  border-collapse: separate;
  border-spacing: 0;
  -webkit-backdrop-filter: blur(20px) saturate(180%);
  backdrop-filter: blur(20px) saturate(180%);
  background: var(--bg);
  border-radius: var(--glass-radius-lg);
  border: 1px solid var(--glass-border);
  box-shadow: var(--shadow);
}
[data-style="glass"] .file-table th:first-child { border-top-left-radius: var(--glass-radius-lg); }
[data-style="glass"] .file-table th:last-child { border-top-right-radius: var(--glass-radius-lg); }
[data-style="glass"] .file-table tr:last-child td:first-child { border-bottom-left-radius: var(--glass-radius-lg); }
[data-style="glass"] .file-table tr:last-child td:last-child { border-bottom-right-radius: var(--glass-radius-lg); }
[data-style="glass"] .file-table th {
  border-bottom: 1px solid var(--glass-border);
}
[data-style="glass"] .file-table td {
  border-bottom: 1px solid rgba(255,255,255,0.1);
}
[data-style="glass"] .file-table tr:last-child td {
  border-bottom: none;
}
[data-style="glass"] .file-table tr:hover td {
  background: var(--hover-bg);
}
[data-style="glass"] .modal {
  -webkit-backdrop-filter: blur(24px) saturate(200%);
  backdrop-filter: blur(24px) saturate(200%);
  border-radius: var(--glass-radius-lg);
  background: var(--card-bg);
  border: 1px solid var(--glass-border);
  box-shadow: var(--shadow);
}
[data-style="glass"] .modal-overlay {
  -webkit-backdrop-filter: blur(6px);
  backdrop-filter: blur(6px);
}
[data-style="glass"] .dialog-overlay {
  -webkit-backdrop-filter: blur(6px);
  backdrop-filter: blur(6px);
}
[data-style="glass"] .dialog {
  -webkit-backdrop-filter: blur(24px) saturate(200%);
  backdrop-filter: blur(24px) saturate(200%);
  border-radius: var(--glass-radius-lg);
  border: 1px solid var(--glass-border);
  box-shadow: var(--shadow);
}

/* Glass lightweight elements — no backdrop-filter, just transparent bg */
[data-style="glass"] .file-card {
  border-radius: var(--glass-radius-md);
  padding: 12px 16px;
  margin-bottom: 8px;
  background: var(--bg);
  border: 1px solid var(--glass-border);
  border-bottom: none;
  box-shadow: 0 2px 8px rgba(0,0,0,0.04);
}
[data-style="glass"] .file-card:last-child { border-bottom: 1px solid var(--glass-border); }
[data-style="glass"] .card-icon {
  background: var(--bg-secondary);
  border-radius: var(--glass-radius-sm);
}
[data-style="glass"] .btn {
  border-radius: var(--glass-radius-sm);
  background: var(--bg-secondary);
  border: 1px solid var(--glass-border);
}
[data-style="glass"] .sort-chip {
  border-radius: 20px;
  border: 1px solid var(--glass-border);
}
[data-style="glass"] .theme-toggle,
[data-style="glass"] .style-toggle {
  border-radius: var(--glass-radius-sm);
  background: var(--bg-secondary);
  border: 1px solid var(--glass-border);
}

/* Glass specular highlight — only on header */
[data-style="glass"] .header::before {
  content: '';
  position: absolute;
  inset: 0;
  border-radius: inherit;
  background: var(--glass-highlight);
  pointer-events: none;
  z-index: 0;
}
[data-style="glass"] .header { position: sticky; top: 0; z-index: 100; }
[data-style="glass"] .header > * { position: relative; z-index: 1; }
[data-style="glass"] .modal { position: relative; }

/* ===== Cartoon Theme Styles ===== */
[data-style="cartoon"] body {
  font-family: "Comic Neue", "Comic Sans MS", "Chalkboard SE", "MarkerFelt-Wide", sans-serif;
  background: var(--body-bg);
}
[data-style="cartoon"] .header {
  border: var(--cartoon-border);
  border-top: none;
  border-left: none;
  border-right: none;
  background: var(--header-bg);
  -webkit-backdrop-filter: none;
  backdrop-filter: none;
}
[data-style="cartoon"] .logo { font-weight: 900; }
[data-style="cartoon"] .file-table {
  border: var(--cartoon-border);
  border-radius: var(--glass-radius-lg);
  box-shadow: var(--cartoon-shadow);
  border-collapse: separate;
  border-spacing: 0;
}
[data-style="cartoon"] .file-table th:first-child { border-top-left-radius: var(--glass-radius-lg); }
[data-style="cartoon"] .file-table th:last-child { border-top-right-radius: var(--glass-radius-lg); }
[data-style="cartoon"] .file-table tr:last-child td:first-child { border-bottom-left-radius: var(--glass-radius-lg); }
[data-style="cartoon"] .file-table tr:last-child td:last-child { border-bottom-right-radius: var(--glass-radius-lg); }
[data-style="cartoon"] .file-table th {
  border-bottom: var(--cartoon-border);
  font-weight: 800;
  text-transform: uppercase;
  letter-spacing: 1px;
}
[data-style="cartoon"] .file-table td {
  border-bottom: 1.5px dashed var(--border);
}
[data-style="cartoon"] .file-table tr:last-child td {
  border-bottom: none;
}
[data-style="cartoon"] .file-table tr:hover td {
  background: var(--hover-bg);
}
[data-style="cartoon"] .btn {
  border: var(--cartoon-border);
  border-radius: 20px;
  box-shadow: var(--cartoon-shadow-sm);
  font-weight: 700;
  background: var(--bg-secondary);
  transition: transform 0.1s, box-shadow 0.1s;
}
[data-style="cartoon"] .btn:hover {
  transform: translate(-1px, -1px);
  box-shadow: 4px 4px 0 var(--border);
}
[data-style="cartoon"] .btn:active {
  transform: translate(1px, 1px);
  box-shadow: 1px 1px 0 var(--border);
  opacity: 1;
}
[data-style="cartoon"] .file-card {
  border: var(--cartoon-border);
  border-radius: var(--glass-radius-md);
  box-shadow: var(--cartoon-shadow-sm);
  margin-bottom: 10px;
  padding: 12px 16px;
  background: var(--card-bg);
}
[data-style="cartoon"] .card-icon {
  border: var(--cartoon-border);
  border-radius: 50%;
  background: var(--bg-secondary);
  font-size: 22px;
}
[data-style="cartoon"] .card-name { font-weight: 700; }
[data-style="cartoon"] .sort-chip {
  border: var(--cartoon-border);
  border-radius: 20px;
  font-weight: 700;
  box-shadow: var(--cartoon-shadow-sm);
}
[data-style="cartoon"] .sort-chip.active {
  background: var(--accent);
  color: #fff;
  border-color: var(--border);
}
[data-style="cartoon"] .theme-toggle,
[data-style="cartoon"] .style-toggle {
  border: var(--cartoon-border);
  border-radius: 50%;
  box-shadow: var(--cartoon-shadow-sm);
  background: var(--bg-secondary);
}
[data-style="cartoon"] .modal {
  border: var(--cartoon-border);
  border-radius: var(--glass-radius-lg);
  box-shadow: 5px 5px 0 var(--border);
}
[data-style="cartoon"] .modal-close {
  border: 2px solid #fff;
  box-shadow: var(--cartoon-shadow-sm);
}
[data-style="cartoon"] .breadcrumbs a {
  font-weight: 600;
  border-radius: 10px;
}
[data-style="cartoon"] .style-menu {
  border: var(--cartoon-border);
  border-radius: var(--glass-radius-md);
  box-shadow: var(--cartoon-shadow);
}

/* ===== Header ===== */
.header {
  background: var(--header-bg);
  -webkit-backdrop-filter: saturate(180%) blur(20px);
  backdrop-filter: saturate(180%) blur(20px);
  border-bottom: 1px solid var(--border);
  padding: 12px 24px;
  padding-left: max(12px, env(safe-area-inset-left));
  padding-right: max(12px, env(safe-area-inset-right));
  display: flex;
  align-items: center;
  justify-content: space-between;
  position: sticky;
  top: 0;
  z-index: 100;
  gap: 8px;
  min-height: 48px;
}
.header-left {
  display: flex;
  align-items: center;
  gap: 12px;
  min-width: 0;
  flex: 1;
}
.logo {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 20px;
  font-weight: 700;
  color: var(--accent);
  text-decoration: none;
  flex-shrink: 0;
}
.logo svg {
  flex-shrink: 0;
}
.breadcrumbs {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 14px;
  flex-wrap: wrap;
  min-width: 0;
}
.breadcrumbs a {
  color: var(--accent);
  text-decoration: none;
  padding: 2px 6px;
  border-radius: 4px;
  white-space: nowrap;
}
.breadcrumbs a:hover { background: var(--hover-bg); }
.breadcrumbs span { white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
.breadcrumbs .sep { color: var(--text-secondary); flex-shrink: 0; }
.theme-toggle, .style-toggle {
  background: var(--bg-secondary);
  border: 1px solid var(--border);
  color: var(--text);
  width: 36px;
  height: 36px;
  border-radius: 8px;
  cursor: pointer;
  font-size: 16px;
  transition: all 0.2s;
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: center;
}
.theme-toggle:hover, .style-toggle:hover { background: var(--hover-bg); }
.header-controls {
  display: flex;
  align-items: center;
  gap: 6px;
  flex-shrink: 0;
}
/* Style switcher popup */
.style-menu {
  position: absolute;
  top: calc(100% + 6px);
  right: 0;
  background: var(--card-bg);
  border: 1px solid var(--border);
  border-radius: var(--glass-radius-md);
  box-shadow: var(--shadow);
  padding: 6px;
  display: none;
  z-index: 200;
  min-width: 150px;
}
[data-style="glass"] .style-menu {
  background: var(--card-bg);
  border: 1px solid var(--glass-border);
}
.style-menu.active { display: block; }
.style-menu-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  border-radius: 6px;
  cursor: pointer;
  font-size: 13px;
  color: var(--text);
  border: none;
  background: none;
  width: 100%;
  text-align: left;
  transition: background 0.15s;
}
.style-menu-item:hover { background: var(--hover-bg); }
.style-menu-item.active { color: var(--accent); font-weight: 600; }
.style-menu-item .check { width: 16px; font-size: 14px; }

/* ===== Sort bar (mobile) ===== */
.sort-bar {
  display: none;
  padding: 8px 0 4px;
  gap: 6px;
  flex-wrap: wrap;
  align-items: center;
}
.sort-bar-label {
  font-size: 12px;
  color: var(--text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.5px;
  font-weight: 600;
}
.sort-chip {
  font-size: 12px;
  padding: 4px 10px;
  border-radius: 14px;
  border: 1px solid var(--border);
  background: var(--bg-secondary);
  color: var(--text-secondary);
  cursor: pointer;
  transition: all 0.15s;
  white-space: nowrap;
}
.sort-chip:hover { border-color: var(--accent); color: var(--accent); }
.sort-chip.active {
  background: var(--accent);
  color: #fff;
  border-color: var(--accent);
}

/* ===== Container ===== */
.container {
  max-width: 1200px;
  margin: 0 auto;
  padding: 16px 24px;
  padding-left: max(16px, env(safe-area-inset-left));
  padding-right: max(16px, env(safe-area-inset-right));
}

/* ===== Table (desktop) ===== */
.file-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 14px;
}
.file-table th {
  text-align: left;
  padding: 10px 12px;
  border-bottom: 2px solid var(--border);
  color: var(--text-secondary);
  font-weight: 600;
  cursor: pointer;
  user-select: none;
  white-space: nowrap;
  font-size: 12px;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}
.file-table th:hover { color: var(--accent); }
.file-table th .sort-arrow { margin-left: 4px; font-size: 10px; }
.file-table td {
  padding: 8px 12px;
  border-bottom: 1px solid var(--border);
  vertical-align: middle;
}
.file-table tr:hover td { background: var(--hover-bg); }
.file-table tr.dir-row { font-weight: 500; }
.file-name-cell {
  display: flex;
  align-items: center;
  gap: 8px;
}
.file-icon {
  width: 20px;
  text-align: center;
  flex-shrink: 0;
  font-size: 16px;
}
.file-name-cell a {
  color: var(--text);
  text-decoration: none;
  word-break: break-word;
}
.file-name-cell a:hover { color: var(--accent); }
.file-name-cell a:visited { color: var(--visited); }
.dir-link { color: var(--accent); font-weight: 500; }
.dir-link:visited { color: var(--visited); }
.size-cell, .date-cell { color: var(--text-secondary); white-space: nowrap; }
.action-cell { white-space: nowrap; }
.btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 4px;
  padding: 6px 12px;
  border-radius: 6px;
  font-size: 12px;
  cursor: pointer;
  border: 1px solid var(--border);
  background: var(--bg-secondary);
  color: var(--text);
  transition: all 0.15s;
  min-height: 32px;
  -webkit-tap-highlight-color: transparent;
}
.btn:hover { background: var(--hover-bg); border-color: var(--accent); color: var(--accent); }
.btn:active { opacity: 0.7; }
.btn-preview { margin-right: 4px; }
.copied {
  background: #30d158 !important;
  color: #fff !important;
  border-color: #30d158 !important;
}

/* ===== QR Modal ===== */
.qr-modal {
  text-align: center;
  padding: 24px;
}
.qr-modal canvas {
  display: block;
  margin: 0 auto 16px;
  border-radius: 8px;
}
.qr-url {
  font-size: 12px;
  color: var(--text-secondary);
  word-break: break-all;
  max-width: 280px;
  margin: 0 auto;
  line-height: 1.4;
}

/* ===== Card list (mobile) ===== */
.file-list { display: none; }
.file-card {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 0;
  border-bottom: 1px solid var(--border);
  -webkit-tap-highlight-color: transparent;
}
.file-card:active { background: var(--hover-bg); border-radius: 8px; }
.card-icon {
  font-size: 28px;
  width: 40px;
  height: 40px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  background: var(--bg-secondary);
  border-radius: 8px;
}
.card-info {
  flex: 1;
  min-width: 0;
}
.card-name {
  font-size: 15px;
  font-weight: 500;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.card-name a {
  color: var(--text);
  text-decoration: none;
}
.card-name a.dir-link { color: var(--accent); }
.card-name a.dir-link:visited { color: var(--visited); }
.card-name a:visited { color: var(--visited); }
.card-meta {
  font-size: 12px;
  color: var(--text-secondary);
  margin-top: 2px;
  display: flex;
  gap: 12px;
  flex-wrap: wrap;
}
.card-actions {
  display: flex;
  gap: 6px;
  flex-shrink: 0;
}

/* ===== Modal ===== */
.modal-overlay {
  display: none;
  position: fixed;
  inset: 0;
  background: var(--modal-bg);
  z-index: 1000;
  justify-content: center;
  align-items: center;
  padding: env(safe-area-inset-top) env(safe-area-inset-right) env(safe-area-inset-bottom) env(safe-area-inset-left);
}
.modal-overlay.active { display: flex; }
.modal {
  background: var(--card-bg);
  border-radius: 12px;
  box-shadow: var(--shadow);
  max-width: 90vw;
  max-height: 90vh;
  overflow: hidden;
  position: relative;
  width: auto;
}
.modal-close {
  position: absolute;
  top: 8px;
  right: 12px;
  background: rgba(0,0,0,0.5);
  color: #fff;
  border: none;
  border-radius: 50%;
  width: 36px;
  height: 36px;
  font-size: 20px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 10;
  -webkit-tap-highlight-color: transparent;
}
.modal-close:hover { background: rgba(0,0,0,0.7); }
.nav-btn {
  position: absolute;
  top: 50%;
  transform: translateY(-50%);
  background: rgba(0,0,0,0.45);
  color: #fff;
  border: none;
  border-radius: 50%;
  width: 40px;
  height: 40px;
  font-size: 20px;
  cursor: pointer;
  display: none;
  align-items: center;
  justify-content: center;
  z-index: 10;
  -webkit-tap-highlight-color: transparent;
  transition: background 0.15s;
}
.nav-btn:hover { background: rgba(0,0,0,0.7); }
.nav-btn.visible { display: flex; }
.nav-prev { left: 8px; }
.nav-next { right: 8px; }
.modal video, .modal audio { display: block; max-width: 88vw; max-height: 80vh; }
.modal img { display: block; max-width: 88vw; max-height: 80vh; object-fit: contain; }
.modal-title {
  padding: 12px 48px 12px 16px;
  font-size: 14px;
  font-weight: 600;
  border-bottom: 1px solid var(--border);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.loading {
  text-align: center;
  padding: 60px 20px;
  color: var(--text-secondary);
  font-size: 16px;
}
.empty-state {
  text-align: center;
  padding: 60px 20px;
  color: var(--text-secondary);
}

/* ===== Pad (768px - 1024px) ===== */
@media (max-width: 1024px) {
  .container { padding: 12px 16px; }
  .file-table td, .file-table th { padding: 8px 10px; }
  .th-created, .date-cell.created { display: none; }
}

/* ===== Mobile (<=768px): switch to card layout ===== */
@media (max-width: 768px) {
  .header {
    padding: 10px 16px;
    min-height: 44px;
  }
  .logo { font-size: 18px; }
  .breadcrumbs { font-size: 13px; gap: 2px; }
  .container { padding: 8px 16px; }

  .file-table { display: none !important; }
  .file-list { display: block; }
  .sort-bar { display: flex; }
}

/* ===== Small mobile (<=480px) ===== */
@media (max-width: 480px) {
  .header { padding: 8px 12px; }
  .breadcrumbs { font-size: 12px; }
  .container { padding: 6px 12px; }
  .card-icon { width: 36px; height: 36px; font-size: 22px; border-radius: 6px; }
  .card-name { font-size: 14px; }
  .card-meta { font-size: 11px; gap: 8px; }
  .card-actions .btn { padding: 4px 8px; font-size: 11px; min-height: 28px; }
  .modal { border-radius: 8px; }
  .modal video, .modal audio { max-width: 94vw; max-height: 75vh; }
  .modal img { max-width: 94vw; max-height: 75vh; }
  .nav-btn { width: 44px; height: 44px; font-size: 22px; }
  .nav-prev { left: 4px; }
  .nav-next { right: 4px; }
}

/* ===== Fullscreen modal on very small screens ===== */
@media (max-width: 480px) and (max-height: 700px) {
  .modal {
    max-width: 100vw;
    max-height: 100vh;
    border-radius: 0;
    width: 100vw;
    height: 100vh;
  }
  .modal video, .modal audio { max-width: 100vw; max-height: calc(100vh - 48px); }
  .modal img { max-width: 100vw; max-height: calc(100vh - 48px); }
}

/* ===== File Management UI ===== */
.btn-upload {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 6px 12px;
  border-radius: 6px;
  font-size: 12px;
  font-weight: 500;
  cursor: pointer;
  border: 1px solid var(--accent);
  background: var(--accent);
  color: #fff;
  transition: opacity 0.15s;
  white-space: nowrap;
}
.btn-upload:hover { opacity: 0.85; }
.btn-upload:active { opacity: 0.7; }
.drop-zone-active {
  outline: 2px dashed var(--accent);
  outline-offset: -4px;
  background: var(--hover-bg);
  border-radius: 8px;
}
/* ===== More Dropdown ===== */
.more-wrap {
  position: relative;
  display: inline-block;
}
.btn-more {
  font-size: 16px;
  padding: 4px 8px;
  min-height: 28px;
  letter-spacing: 1px;
}
.more-menu {
  display: none;
  position: absolute;
  right: 0;
  top: calc(100% + 4px);
  background: var(--card-bg);
  border: 1px solid var(--border);
  border-radius: 8px;
  box-shadow: var(--shadow);
  padding: 4px;
  z-index: 150;
  min-width: 120px;
  white-space: nowrap;
}
.more-menu.active { display: block; }
.more-menu.more-menu-up {
  top: auto;
  bottom: calc(100% + 4px);
}
.more-item {
  display: block;
  width: 100%;
  padding: 7px 12px;
  font-size: 13px;
  border: none;
  background: none;
  color: var(--text);
  text-align: left;
  border-radius: 5px;
  cursor: pointer;
  transition: background 0.12s;
}
.more-item:hover { background: var(--hover-bg); }
.more-item-danger { color: #ff3b30; }
.more-item-danger:hover { background: rgba(255,59,48,0.08); }
[data-style="glass"] .more-menu {
  -webkit-backdrop-filter: blur(20px) saturate(180%);
  backdrop-filter: blur(20px) saturate(180%);
  border: 1px solid var(--glass-border);
}

/* ===== Dialog Modals (auth / rename / confirm) ===== */
.dialog-overlay {
  display: none;
  position: fixed;
  inset: 0;
  background: var(--modal-bg);
  z-index: 2000;
  justify-content: center;
  align-items: center;
  padding: 16px;
}
.dialog-overlay.active { display: flex; }
.dialog {
  background: var(--card-bg);
  border-radius: 12px;
  box-shadow: var(--shadow);
  padding: 24px;
  width: 100%;
  max-width: 360px;
}
.dialog h3 {
  font-size: 16px;
  font-weight: 600;
  margin-bottom: 16px;
}
.dialog-input {
  display: block;
  width: 100%;
  padding: 8px 12px;
  border: 1px solid var(--border);
  border-radius: 6px;
  background: var(--bg);
  color: var(--text);
  font-size: 14px;
  margin-bottom: 12px;
  outline: none;
}
.dialog-input:focus { border-color: var(--accent); }
.dialog-msg {
  font-size: 14px;
  color: var(--text-secondary);
  margin-bottom: 16px;
  word-break: break-word;
}
.dialog-buttons {
  display: flex;
  gap: 8px;
  justify-content: flex-end;
}
.dialog-btn {
  padding: 8px 16px;
  border-radius: 6px;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  border: 1px solid var(--border);
  background: var(--bg-secondary);
  color: var(--text);
  transition: opacity 0.15s;
}
.dialog-btn:hover { opacity: 0.8; }
.dialog-btn-primary {
  background: var(--accent);
  border-color: var(--accent);
  color: #fff;
}
.dialog-btn-danger {
  background: #ff3b30;
  border-color: #ff3b30;
  color: #fff;
}
.dialog-error {
  font-size: 12px;
  color: #ff3b30;
  margin-bottom: 8px;
  display: none;
}
.dialog-error.visible { display: block; }

/* ===== Move Browser ===== */
.move-browser {
  border: 1px solid var(--border);
  border-radius: 8px;
  max-height: 280px;
  overflow-y: auto;
  margin-bottom: 12px;
  background: var(--bg);
}
.move-path {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 8px 12px;
  font-size: 12px;
  color: var(--text-secondary);
  border-bottom: 1px solid var(--border);
  flex-wrap: wrap;
  background: var(--bg-secondary);
  border-radius: 8px 8px 0 0;
  position: sticky;
  top: 0;
  z-index: 1;
}
.move-path-seg {
  cursor: pointer;
  color: var(--accent);
  padding: 1px 4px;
  border-radius: 3px;
}
.move-path-seg:hover { background: var(--hover-bg); }
.move-path-cur {
  color: var(--text);
  font-weight: 500;
}
.move-folder-list {
  padding: 4px;
}
.move-folder-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  border-radius: 6px;
  cursor: pointer;
  font-size: 13px;
  color: var(--text);
  transition: background 0.12s;
}
.move-folder-item:hover { background: var(--hover-bg); }
.move-folder-item .mf-icon {
  font-size: 16px;
  flex-shrink: 0;
  width: 20px;
  text-align: center;
}
.move-folder-item .mf-name {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.move-empty {
  padding: 20px 12px;
  text-align: center;
  font-size: 13px;
  color: var(--text-secondary);
}
.move-loading {
  padding: 20px 12px;
  text-align: center;
  font-size: 13px;
  color: var(--text-secondary);
}

/* ===== Toast Notifications ===== */
.toast-container {
  position: fixed;
  top: 60px;
  right: 16px;
  z-index: 3000;
  display: flex;
  flex-direction: column;
  gap: 8px;
  pointer-events: none;
}
.toast {
  padding: 10px 16px;
  border-radius: 8px;
  font-size: 13px;
  font-weight: 500;
  box-shadow: 0 4px 16px rgba(0,0,0,0.15);
  pointer-events: auto;
  animation: toastIn 0.25s ease;
  max-width: 320px;
  word-break: break-word;
}
.toast-success {
  background: #30d158;
  color: #fff;
}
.toast-error {
  background: #ff3b30;
  color: #fff;
}
@keyframes toastIn {
  from { opacity: 0; transform: translateX(20px); }
  to { opacity: 1; transform: translateX(0); }
}
@keyframes toastOut {
  from { opacity: 1; transform: translateX(0); }
  to { opacity: 0; transform: translateX(20px); }
}

/* ===== Entry Highlight Flash ===== */
@keyframes highlightFlash {
  0% { background: transparent; }
  10% { background: rgba(0,113,227,0.18); }
  20% { background: transparent; }
  30% { background: rgba(0,113,227,0.18); }
  40% { background: transparent; }
  55% { background: rgba(0,113,227,0.18); }
  65% { background: transparent; }
  80% { background: rgba(0,113,227,0.18); }
  90% { background: transparent; }
  100% { background: transparent; }
}
[data-mode="dark"] .entry-highlight { --hl-color: rgba(41,151,255,0.2); }
.file-table tr.entry-highlight td,
.file-card.entry-highlight {
  animation: highlightFlash 2s ease;
}
</style>
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
(function() {
  const ICONS = {
    folder: '\uD83D\uDCC1',
    video: '\uD83C\uDFA5',
    audio: '\uD83C\uDFB5',
    image: '\uD83D\uDDBC\uFE0F',
    text: '\uD83D\uDCC4',
    pdf: '\uD83D\uDCC4',
    archive: '\uD83D\uDCE6',
    document: '\uD83D\uDCC3',
    spreadsheet: '\uD83D\uDCCA',
    presentation: '\uD83D\uDCCA',
    code: '\uD83D\uDCBB',
    file: '\uD83D\uDCC4',
  };

  const SORT_LABELS = { name: 'Name', size: 'Size', created: 'Created', modified: 'Modified' };
  let currentEntries = [];
  let sortField = 'name';
  let sortAsc = true;
  let imageList = [];
  let currentImageIndex = -1;
  let webdavEnabled = false;
  let webdavAuth = false;
  let currentPath = '/';

  // Theme & Style
  function initTheme() {
    // Migrate from old localStorage key
    var oldTheme = localStorage.getItem('echofs-theme');
    if (oldTheme && !localStorage.getItem('echofs-mode')) {
      localStorage.setItem('echofs-mode', oldTheme);
      localStorage.removeItem('echofs-theme');
    }

    const savedMode = localStorage.getItem('echofs-mode');
    const savedStyle = localStorage.getItem('echofs-style') || 'classic';

    // Apply style
    if (savedStyle === 'glass' || savedStyle === 'cartoon') {
      document.documentElement.setAttribute('data-style', savedStyle);
    } else {
      document.documentElement.removeAttribute('data-style');
    }

    // Apply mode
    if (savedMode === 'dark' || (!savedMode && window.matchMedia('(prefers-color-scheme: dark)').matches)) {
      document.documentElement.setAttribute('data-mode', 'dark');
    } else {
      document.documentElement.removeAttribute('data-mode');
    }
    updateThemeButton();
    updateStyleMenu();
  }

  function toggleTheme() {
    const isDark = document.documentElement.getAttribute('data-mode') === 'dark';
    if (isDark) {
      document.documentElement.removeAttribute('data-mode');
      localStorage.setItem('echofs-mode', 'light');
    } else {
      document.documentElement.setAttribute('data-mode', 'dark');
      localStorage.setItem('echofs-mode', 'dark');
    }
    updateThemeButton();
  }

  function setStyle(style) {
    if (style === 'glass' || style === 'cartoon') {
      document.documentElement.setAttribute('data-style', style);
    } else {
      document.documentElement.removeAttribute('data-style');
    }
    localStorage.setItem('echofs-style', style);
    updateStyleMenu();
    closeStyleMenu();
  }

  function updateThemeButton() {
    const btn = document.getElementById('themeToggle');
    const isDark = document.documentElement.getAttribute('data-mode') === 'dark';
    btn.innerHTML = isDark ? '&#9788;' : '&#9790;';
  }

  function updateStyleMenu() {
    const currentStyle = document.documentElement.getAttribute('data-style') || 'classic';
    document.querySelectorAll('.style-menu-item').forEach(function(item) {
      const choice = item.getAttribute('data-style-choice');
      const isActive = choice === currentStyle;
      item.classList.toggle('active', isActive);
      item.querySelector('.check').textContent = isActive ? '✓' : '';
    });
  }

  function toggleStyleMenu() {
    var menu = document.getElementById('styleMenu');
    menu.classList.toggle('active');
  }

  function closeStyleMenu() {
    document.getElementById('styleMenu').classList.remove('active');
  }

  document.getElementById('themeToggle').addEventListener('click', toggleTheme);
  document.getElementById('styleToggle').addEventListener('click', function(e) {
    e.stopPropagation();
    toggleStyleMenu();
  });
  document.querySelectorAll('.style-menu-item').forEach(function(item) {
    item.addEventListener('click', function(e) {
      e.stopPropagation();
      setStyle(this.getAttribute('data-style-choice'));
    });
  });
  initTheme();

  // Navigation
  function navigateTo(path) {
    history.pushState(null, '', path);
    loadDirectory(path);
  }

  window.addEventListener('popstate', () => {
    loadDirectory(location.pathname);
  });

  async function loadDirectory(path) {
    const content = document.getElementById('content');
    content.innerHTML = '<div class="loading">Loading...</div>';
    currentPath = path;

    try {
      const resp = await fetch(path, {
        headers: { 'X-Requested-With': 'XMLHttpRequest' }
      });
      if (!resp.ok) throw new Error('Failed to load directory');
      const data = await resp.json();
      currentEntries = data.entries;
      webdavEnabled = !!data.webdav;
      webdavAuth = !!data.webdav_auth;
      renderBreadcrumbs(data.breadcrumbs);
      document.title = data.path === '/' ? 'EchoFS' : data.path.split('/').filter(Boolean).pop() + ' — EchoFS';
      sortAndRender();
    } catch (e) {
      content.innerHTML = '<div class="empty-state">Failed to load directory</div>';
    }
  }

  function renderBreadcrumbs(crumbs) {
    const el = document.getElementById('breadcrumbs');
    el.innerHTML = crumbs.map((c, i) => {
      if (i === crumbs.length - 1 && i > 0) {
        return '<span class="sep">/</span><span>' + escHtml(c.name) + '</span>';
      }
      if (i === 0) return '';
      return '<span class="sep">/</span><a href="' + escHtml(c.href) + '" data-nav>' + escHtml(c.name) + '</a>';
    }).join('');
  }

  function doSort(entries) {
    const dirs = entries.filter(e => e.is_dir);
    const files = entries.filter(e => !e.is_dir);
    const sortFn = (a, b) => {
      let cmp = 0;
      if (sortField === 'name') cmp = a.name.localeCompare(b.name, undefined, {sensitivity: 'base'});
      else if (sortField === 'size') cmp = a.size - b.size;
      else if (sortField === 'created') cmp = a.created_ts - b.created_ts;
      else if (sortField === 'modified') cmp = a.modified_ts - b.modified_ts;
      return sortAsc ? cmp : -cmp;
    };
    dirs.sort(sortFn);
    files.sort(sortFn);
    return [...dirs, ...files];
  }

  function sortAndRender() {
    const sorted = doSort(currentEntries);
    const content = document.getElementById('content');
    let html = '';

    // Show/hide upload button in header
    var uploadBtn = document.getElementById('uploadBtn');
    if (uploadBtn) uploadBtn.style.display = webdavEnabled ? '' : 'none';

    if (sorted.length === 0) {
      html += '<div class="empty-state">Empty directory — drag files here to upload</div>';
      content.innerHTML = html;
      return;
    }
    html += renderSortBar();
    html += renderTable(sorted);
    html += renderCardList(sorted);
    content.innerHTML = html;
    attachSortHandlers();
    // Highlight entry after render if pending
    if (pendingHighlight) {
      var name = pendingHighlight;
      pendingHighlight = null;
      setTimeout(function() { highlightEntry(name); }, 100);
    }
  }

  function renderSortBar() {
    let html = '<div class="sort-bar">';
    html += '<span class="sort-bar-label">Sort:</span>';
    for (const [key, label] of Object.entries(SORT_LABELS)) {
      const active = sortField === key;
      const arrow = active ? (sortAsc ? ' \u25B2' : ' \u25BC') : '';
      html += '<button class="sort-chip' + (active ? ' active' : '') + '" data-sort="' + key + '">' + label + arrow + '</button>';
    }
    html += '</div>';
    return html;
  }

  function arrow(field) {
    if (field !== sortField) return '';
    return '<span class="sort-arrow">' + (sortAsc ? '\u25B2' : '\u25BC') + '</span>';
  }

  function buildActions(e, isCard) {
    var sp = isCard ? 'event.stopPropagation();' : '';
    var hasPreview = (e.media_type === 'video' || e.media_type === 'audio' || e.media_type === 'image');

    // Primary buttons (max 2 shown)
    var primary = [];
    if (hasPreview) {
      primary.push('<button class="btn btn-preview" onclick="' + sp + 'previewMedia(\'' + escJs(e.href) + '\', \'' + escJs(e.name) + '\', \'' + escJs(e.media_type) + '\')">' + (isCard ? 'Play' : 'Preview') + '</button>');
      primary.push('<button class="btn" onclick="' + sp + 'copyLink(\'' + escJs(e.href) + '\', this)">' + (isCard ? 'Copy' : 'Copy Link') + '</button>');
    } else {
      primary.push('<button class="btn" onclick="' + sp + 'copyLink(\'' + escJs(e.href) + '\', this)">' + (isCard ? 'Copy' : 'Copy Link') + '</button>');
      primary.push('<button class="btn" onclick="' + sp + 'showQr(\'' + escJs(e.href) + '\')">QR</button>');
    }

    // More menu items
    var moreItems = [];
    if (hasPreview) {
      moreItems.push({ label: 'QR Code', action: sp + 'showQr(\'' + escJs(e.href) + '\')' });
    }
    if (webdavEnabled) {
      moreItems.push({ label: 'Rename', action: sp + 'renameEntry(\'' + escJs(e.href) + '\', \'' + escJs(e.name) + '\')' });
      moreItems.push({ label: 'Move to...', action: sp + 'moveEntry(\'' + escJs(e.href) + '\', \'' + escJs(e.name) + '\', ' + (e.is_dir ? 'true' : 'false') + ')' });
      moreItems.push({ label: 'Delete', cls: 'more-item-danger', action: sp + 'deleteEntry(\'' + escJs(e.href) + '\', \'' + escJs(e.name) + '\', ' + (e.is_dir ? 'true' : 'false') + ')' });
    }

    var html = primary.join('');

    if (moreItems.length > 0) {
      html += '<div class="more-wrap"><button class="btn btn-more" onclick="' + sp + 'toggleMore(this)">&#8943;</button>';
      html += '<div class="more-menu">';
      for (var i = 0; i < moreItems.length; i++) {
        var item = moreItems[i];
        html += '<button class="more-item' + (item.cls ? ' ' + item.cls : '') + '" onclick="' + item.action + ';closeAllMore()">' + item.label + '</button>';
      }
      html += '</div></div>';
    }

    return html;
  }

  function renderTable(entries) {
    let html = '<table class="file-table"><thead><tr>';
    html += '<th data-sort="name">Name ' + arrow('name') + '</th>';
    html += '<th data-sort="size">Size ' + arrow('size') + '</th>';
    html += '<th data-sort="created" class="th-created">Created ' + arrow('created') + '</th>';
    html += '<th data-sort="modified" class="th-modified">Modified ' + arrow('modified') + '</th>';
    html += '<th>Actions</th>';
    html += '</tr></thead><tbody>';

    for (const e of entries) {
      const icon = ICONS[e.icon] || ICONS.file;
      const cls = e.is_dir ? 'dir-row' : '';
      const nameLink = e.is_dir
        ? '<a href="' + escHtml(e.href) + '" class="dir-link" data-nav>' + escHtml(e.name) + '</a>'
        : '<a href="' + escHtml(e.href) + '">' + escHtml(e.name) + '</a>';

      let actions = buildActions(e, false);

      html += '<tr class="' + cls + '" data-entry-name="' + escHtml(e.name) + '">';
      html += '<td><div class="file-name-cell"><span class="file-icon">' + icon + '</span>' + nameLink + '</div></td>';
      html += '<td class="size-cell">' + escHtml(e.size_display) + '</td>';
      html += '<td class="date-cell created">' + escHtml(e.created) + '</td>';
      html += '<td class="date-cell modified">' + escHtml(e.modified) + '</td>';
      html += '<td class="action-cell">' + actions + '</td>';
      html += '</tr>';
    }
    html += '</tbody></table>';
    return html;
  }

  function renderCardList(entries) {
    let html = '<div class="file-list">';
    for (const e of entries) {
      const icon = ICONS[e.icon] || ICONS.file;
      const nameLink = e.is_dir
        ? '<a href="' + escHtml(e.href) + '" class="dir-link" data-nav>' + escHtml(e.name) + '</a>'
        : '<a href="' + escHtml(e.href) + '">' + escHtml(e.name) + '</a>';

      const meta = [];
      if (!e.is_dir) meta.push(escHtml(e.size_display));
      if (e.modified) meta.push(escHtml(e.modified));

      let actions = buildActions(e, true);

      html += '<div class="file-card" data-entry-name="' + escHtml(e.name) + '">';
      html += '<div class="card-icon">' + icon + '</div>';
      html += '<div class="card-info">';
      html += '<div class="card-name">' + nameLink + '</div>';
      if (meta.length) html += '<div class="card-meta"><span>' + meta.join('</span><span>') + '</span></div>';
      html += '</div>';
      html += '<div class="card-actions">' + actions + '</div>';
      html += '</div>';
    }
    html += '</div>';
    return html;
  }

  function attachSortHandlers() {
    document.querySelectorAll('[data-sort]').forEach(el => {
      el.addEventListener('click', () => {
        const field = el.getAttribute('data-sort');
        if (sortField === field) sortAsc = !sortAsc;
        else { sortField = field; sortAsc = true; }
        sortAndRender();
      });
    });
  }

  // QR code generation (minimal inline QR encoder)
  // QR Code generation using qrcode-generator library
  function makeQrCanvas(text, cellSize) {
    var qr = qrcode(0, 'M');
    qr.addData(text);
    qr.make();
    var count = qr.getModuleCount();
    var scale = cellSize || 5;
    var canvas = document.createElement('canvas');
    var size = count * scale;
    canvas.width = size; canvas.height = size;
    var ctx = canvas.getContext('2d');
    ctx.fillStyle = '#ffffff';
    ctx.fillRect(0, 0, size, size);
    ctx.fillStyle = '#1d1d1f';
    for (var y = 0; y < count; y++) for (var x = 0; x < count; x++) {
      if (qr.isDark(y, x)) ctx.fillRect(x * scale, y * scale, scale, scale);
    }
    return canvas;
  }

  // QR Code modal
  window.showQr = function(href) {
    var url = location.origin + href;
    var modal = document.getElementById('qrModal');
    var content = document.getElementById('qrModalContent');
    try {
      var canvas = makeQrCanvas(url, 5);
      content.innerHTML = '';
      content.appendChild(canvas);
      var urlDiv = document.createElement('div');
      urlDiv.className = 'qr-url';
      urlDiv.textContent = url;
      content.appendChild(urlDiv);
    } catch(e) {
      content.innerHTML = '<p>URL too long for QR code</p>';
    }
    modal.classList.add('active');
    document.body.style.overflow = 'hidden';
  };

  document.getElementById('qrModalClose').addEventListener('click', closeQrModal);
  document.getElementById('qrModal').addEventListener('click', function(e) {
    if (e.target === this) closeQrModal();
  });

  function closeQrModal() {
    var modal = document.getElementById('qrModal');
    modal.classList.remove('active');
    document.body.style.overflow = '';
  }

  // Copy link
  function copyFallback(text) {
    var el = document.createElement('input');
    el.setAttribute('readonly', '');
    el.style.position = 'fixed';
    el.style.left = '0';
    el.style.top = '0';
    el.style.opacity = '0';
    el.value = text;
    document.body.appendChild(el);
    el.focus();
    el.setSelectionRange(0, text.length);
    var ok = document.execCommand('copy');
    document.body.removeChild(el);
    return ok;
  }
  window.copyLink = function(href, btn) {
    var url = location.origin + href;
    function onSuccess() {
      btn.classList.add('copied');
      var orig = btn.textContent;
      btn.textContent = 'Copied!';
      setTimeout(function() { btn.classList.remove('copied'); btn.textContent = orig; }, 1500);
    }
    if (navigator.clipboard && navigator.clipboard.writeText && window.isSecureContext) {
      navigator.clipboard.writeText(url).then(onSuccess).catch(function() {
        copyFallback(url);
        onSuccess();
      });
    } else {
      copyFallback(url);
      onSuccess();
    }
  };

  // Media preview
  window.previewMedia = function(href, name, type) {
    const modal = document.getElementById('modal');
    const title = document.getElementById('modalTitle');
    const mc = document.getElementById('modalContent');
    title.textContent = name;
    mc.innerHTML = '';

    // Build image list for gallery navigation
    imageList = currentEntries.filter(e => e.media_type === 'image').map(e => ({ href: e.href, name: e.name }));
    currentImageIndex = -1;
    if (type === 'image') {
      currentImageIndex = imageList.findIndex(img => img.href === href);
    }

    if (type === 'video') {
      mc.innerHTML = '<video controls autoplay playsinline><source src="' + escHtml(href) + '">Your browser does not support video playback.</video>';
    } else if (type === 'audio') {
      mc.innerHTML = '<audio controls autoplay><source src="' + escHtml(href) + '">Your browser does not support audio playback.</audio>';
    } else if (type === 'image') {
      mc.innerHTML = '<img src="' + escHtml(href) + '" alt="' + escHtml(name) + '">';
      updateImageCounter(title, name);
    }
    updateNavButtons();
    modal.classList.add('active');
    document.body.style.overflow = 'hidden';
  };

  function updateImageCounter(titleEl, name) {
    if (imageList.length > 1 && currentImageIndex >= 0) {
      titleEl.textContent = name + ' (' + (currentImageIndex + 1) + ' / ' + imageList.length + ')';
    }
  }

  function updateNavButtons() {
    const prevBtn = document.getElementById('navPrev');
    const nextBtn = document.getElementById('navNext');
    if (imageList.length <= 1 || currentImageIndex < 0) {
      prevBtn.classList.remove('visible');
      nextBtn.classList.remove('visible');
      return;
    }
    prevBtn.classList.toggle('visible', currentImageIndex > 0);
    nextBtn.classList.toggle('visible', currentImageIndex < imageList.length - 1);
  }

  function navigateImage(direction) {
    if (imageList.length <= 1) return;
    const newIndex = currentImageIndex + direction;
    if (newIndex < 0 || newIndex >= imageList.length) return;
    currentImageIndex = newIndex;
    const img = imageList[currentImageIndex];
    const mc = document.getElementById('modalContent');
    mc.innerHTML = '<img src="' + escHtml(img.href) + '" alt="' + escHtml(img.name) + '">';
    const title = document.getElementById('modalTitle');
    updateImageCounter(title, img.name);
    updateNavButtons();
  }

  document.getElementById('navPrev').addEventListener('click', function(e) {
    e.stopPropagation();
    navigateImage(-1);
  });
  document.getElementById('navNext').addEventListener('click', function(e) {
    e.stopPropagation();
    navigateImage(1);
  });

  // Touch swipe support for image gallery
  (function() {
    var startX = 0;
    var startY = 0;
    var mc = document.getElementById('modalContent');
    mc.addEventListener('touchstart', function(e) {
      if (imageList.length <= 1) return;
      startX = e.touches[0].clientX;
      startY = e.touches[0].clientY;
    }, { passive: true });
    mc.addEventListener('touchend', function(e) {
      if (imageList.length <= 1 || startX === 0) return;
      var dx = e.changedTouches[0].clientX - startX;
      var dy = e.changedTouches[0].clientY - startY;
      startX = 0;
      if (Math.abs(dx) < 50 || Math.abs(dy) > Math.abs(dx)) return;
      navigateImage(dx > 0 ? -1 : 1);
    }, { passive: true });
  })();

  // Modal close
  document.getElementById('modalClose').addEventListener('click', closeModal);
  document.getElementById('modal').addEventListener('click', function(e) {
    if (e.target === this) closeModal();
  });

  function closeModal() {
    const modal = document.getElementById('modal');
    modal.classList.remove('active');
    document.body.style.overflow = '';
    const mc = document.getElementById('modalContent');
    mc.querySelectorAll('video, audio').forEach(el => { el.pause(); el.src = ''; });
    mc.innerHTML = '';
    imageList = [];
    currentImageIndex = -1;
    updateNavButtons();
  }

  var _escDiv = document.createElement('div');
  function escHtml(s) {
    _escDiv.textContent = s;
    return _escDiv.innerHTML;
  }
  function escJs(s) {
    return s.replace(/\\/g, '\\\\').replace(/'/g, "\\'").replace(/"/g, '\\"');
  }

  // More dropdown management
  window.toggleMore = function(btn) {
    var menu = btn.nextElementSibling;
    var wasActive = menu.classList.contains('active');
    closeAllMore();
    if (!wasActive) {
      menu.classList.add('active');
      // If menu overflows bottom of viewport, open upward
      var menuRect = menu.getBoundingClientRect();
      if (menuRect.bottom > window.innerHeight - 8) {
        menu.classList.add('more-menu-up');
      }
    }
  };
  window.closeAllMore = function() {
    document.querySelectorAll('.more-menu.active').forEach(function(m) {
      m.classList.remove('active');
      m.classList.remove('more-menu-up');
    });
  };

  // ===== File Management Operations =====

  // Toast notifications
  function showToast(message, type) {
    var container = document.getElementById('toastContainer');
    var toast = document.createElement('div');
    toast.className = 'toast toast-' + (type || 'success');
    toast.textContent = message;
    container.appendChild(toast);
    setTimeout(function() {
      toast.style.animation = 'toastOut 0.25s ease forwards';
      setTimeout(function() { toast.remove(); }, 250);
    }, 3000);
  }

  // Highlight & scroll to an entry by name
  var pendingHighlight = null;

  function highlightEntry(name) {
    var el = document.querySelector('[data-entry-name="' + CSS.escape(name) + '"]');
    if (!el) return;
    el.scrollIntoView({ behavior: 'smooth', block: 'center' });
    el.classList.add('entry-highlight');
    setTimeout(function() { el.classList.remove('entry-highlight'); }, 2500);
  }

  function scheduleHighlight(name) {
    pendingHighlight = name;
  }

  // Auth helpers
  function getAuthHeader() {
    var cred = sessionStorage.getItem('echofs-auth');
    if (!cred) return null;
    return 'Basic ' + cred;
  }

  function setAuthCredentials(user, pass) {
    sessionStorage.setItem('echofs-auth', btoa(user + ':' + pass));
  }

  function clearAuthCredentials() {
    sessionStorage.removeItem('echofs-auth');
  }

  // Build headers for WebDAV requests
  function buildHeaders(extra) {
    var h = {};
    var auth = getAuthHeader();
    if (auth) h['Authorization'] = auth;
    if (extra) { for (var k in extra) h[k] = extra[k]; }
    return h;
  }

  // Perform a WebDAV operation with auth retry
  var pendingAuthCallback = null;
  var pendingAuthReject = null;

  async function webdavFetch(url, opts) {
    var resp = await fetch(url, opts);
    if (resp.status === 401 && webdavAuth) {
      // Need credentials — clear bad ones
      clearAuthCredentials();
      return new Promise(function(resolve, reject) {
        pendingAuthCallback = function() {
          // Rebuild headers with new credentials
          if (opts.headers) {
            opts.headers['Authorization'] = getAuthHeader();
          } else {
            opts.headers = { 'Authorization': getAuthHeader() };
          }
          resolve(fetch(url, opts));
        };
        pendingAuthReject = reject;
        showAuthDialog();
      });
    }
    return resp;
  }

  // Auth dialog
  function showAuthDialog() {
    var dialog = document.getElementById('authDialog');
    var userInput = document.getElementById('authUser');
    var passInput = document.getElementById('authPass');
    var err = document.getElementById('authError');
    err.classList.remove('visible');
    err.textContent = '';
    userInput.value = '';
    passInput.value = '';
    dialog.classList.add('active');
    userInput.focus();
  }

  function closeAuthDialog() {
    document.getElementById('authDialog').classList.remove('active');
    if (pendingAuthReject) {
      pendingAuthReject(new Error('Authentication cancelled'));
    }
    pendingAuthCallback = null;
    pendingAuthReject = null;
  }

  document.getElementById('authCancel').addEventListener('click', closeAuthDialog);
  document.getElementById('authDialog').addEventListener('click', function(e) {
    if (e.target === this) closeAuthDialog();
  });
  document.getElementById('authSubmit').addEventListener('click', submitAuth);
  document.getElementById('authPass').addEventListener('keydown', function(e) {
    if (e.key === 'Enter') submitAuth();
  });

  function submitAuth() {
    var user = document.getElementById('authUser').value;
    var pass = document.getElementById('authPass').value;
    if (!user) {
      var err = document.getElementById('authError');
      err.textContent = 'Username is required';
      err.classList.add('visible');
      return;
    }
    setAuthCredentials(user, pass);
    document.getElementById('authDialog').classList.remove('active');
    pendingAuthReject = null; // Don't reject, we're proceeding
    if (pendingAuthCallback) {
      var cb = pendingAuthCallback;
      pendingAuthCallback = null;
      cb();
    }
  }

  // Rename
  var renameTarget = { href: '', name: '' };

  window.renameEntry = function(href, name) {
    renameTarget = { href: href, name: name };
    var input = document.getElementById('renameInput');
    var err = document.getElementById('renameError');
    err.classList.remove('visible');
    err.textContent = '';
    input.value = name;
    document.getElementById('renameDialog').classList.add('active');
    input.focus();
    // Select filename without extension for files
    var dotIdx = name.lastIndexOf('.');
    if (dotIdx > 0) {
      input.setSelectionRange(0, dotIdx);
    } else {
      input.select();
    }
  };

  document.getElementById('renameCancel').addEventListener('click', function() {
    document.getElementById('renameDialog').classList.remove('active');
  });
  document.getElementById('renameDialog').addEventListener('click', function(e) {
    if (e.target === this) this.classList.remove('active');
  });
  document.getElementById('renameSubmit').addEventListener('click', doRename);
  document.getElementById('renameInput').addEventListener('keydown', function(e) {
    if (e.key === 'Enter') doRename();
  });

  async function doRename() {
    var newName = document.getElementById('renameInput').value.trim();
    var err = document.getElementById('renameError');
    if (!newName) {
      err.textContent = 'Name cannot be empty';
      err.classList.add('visible');
      return;
    }
    if (newName === renameTarget.name) {
      document.getElementById('renameDialog').classList.remove('active');
      return;
    }
    if (newName.includes('/') || newName.includes('\\')) {
      err.textContent = 'Name cannot contain / or \\';
      err.classList.add('visible');
      return;
    }

    // Build destination path: same parent directory + new name
    var parts = renameTarget.href.split('/');
    parts.pop(); // remove old name
    var dest = parts.join('/') + '/' + encodeURIComponent(newName);

    try {
      var resp = await webdavFetch(renameTarget.href, {
        method: 'MOVE',
        headers: buildHeaders({ 'Destination': dest })
      });
      if (!resp || (!resp.ok && resp.status !== 201 && resp.status !== 204)) {
        var msg = resp ? (resp.statusText || 'Failed') : 'Failed';
        showToast('Rename failed: ' + msg, 'error');
        err.textContent = 'Rename failed: ' + msg;
        err.classList.add('visible');
        return;
      }
      document.getElementById('renameDialog').classList.remove('active');
      showToast('Renamed to "' + newName + '"', 'success');
      scheduleHighlight(newName);
      loadDirectory(currentPath);
    } catch (e) {
      err.textContent = 'Rename failed: ' + e.message;
      err.classList.add('visible');
    }
  }

  // Delete
  var deleteTarget = { href: '', name: '', isDir: false };

  window.deleteEntry = function(href, name, isDir) {
    deleteTarget = { href: href, name: name, isDir: isDir };
    var msg = document.getElementById('deleteMsg');
    var err = document.getElementById('deleteError');
    err.classList.remove('visible');
    err.textContent = '';
    msg.textContent = isDir
      ? 'Delete folder "' + name + '" and all its contents?'
      : 'Delete "' + name + '"?';
    document.getElementById('deleteDialog').classList.add('active');
  };

  document.getElementById('deleteCancel').addEventListener('click', function() {
    document.getElementById('deleteDialog').classList.remove('active');
  });
  document.getElementById('deleteDialog').addEventListener('click', function(e) {
    if (e.target === this) this.classList.remove('active');
  });
  document.getElementById('deleteSubmit').addEventListener('click', doDelete);

  async function doDelete() {
    var err = document.getElementById('deleteError');
    try {
      var resp = await webdavFetch(deleteTarget.href, {
        method: 'DELETE',
        headers: buildHeaders()
      });
      if (!resp || (resp.status !== 204 && resp.status !== 200)) {
        var msg = resp ? (resp.statusText || 'Failed') : 'Failed';
        showToast('Delete failed: ' + msg, 'error');
        err.textContent = 'Delete failed: ' + msg;
        err.classList.add('visible');
        return;
      }
      document.getElementById('deleteDialog').classList.remove('active');
      showToast('"' + deleteTarget.name + '" deleted', 'success');
      loadDirectory(currentPath);
    } catch (e) {
      err.textContent = 'Delete failed: ' + e.message;
      err.classList.add('visible');
    }
  }

  // Move to folder
  var moveTarget = { href: '', name: '', isDir: false };
  var moveBrowsePath = '/';

  window.moveEntry = function(href, name, isDir) {
    moveTarget = { href: href, name: name, isDir: isDir };
    var err = document.getElementById('moveError');
    err.classList.remove('visible');
    err.textContent = '';
    // Start browsing from the current directory's parent or current directory
    moveBrowsePath = currentPath;
    if (!moveBrowsePath.endsWith('/')) moveBrowsePath += '/';
    document.getElementById('moveDialog').classList.add('active');
    loadMoveFolders(moveBrowsePath);
  };

  document.getElementById('moveCancel').addEventListener('click', function() {
    document.getElementById('moveDialog').classList.remove('active');
  });
  document.getElementById('moveDialog').addEventListener('click', function(e) {
    if (e.target === this) this.classList.remove('active');
  });
  document.getElementById('moveSubmit').addEventListener('click', doMove);

  function loadMoveFolders(path) {
    var browser = document.getElementById('moveBrowser');
    browser.innerHTML = '<div class="move-loading">Loading...</div>';
    fetch(path, { headers: { 'X-Requested-With': 'XMLHttpRequest' } })
      .then(function(resp) {
        if (!resp.ok) throw new Error('Failed to load');
        return resp.json();
      })
      .then(function(data) {
        moveBrowsePath = path;
        renderMoveBrowser(data, path);
      })
      .catch(function() {
        browser.innerHTML = '<div class="move-empty">Failed to load directory</div>';
      });
  }

  function renderMoveBrowser(data, browsePath) {
    var browser = document.getElementById('moveBrowser');
    var html = '';

    // Breadcrumb path bar
    html += '<div class="move-path">';
    var segments = browsePath.split('/').filter(Boolean);
    html += '<span class="move-path-seg" onclick="moveBrowseTo(\'/\')">Root</span>';
    var builtPath = '';
    for (var i = 0; i < segments.length; i++) {
      builtPath += '/' + segments[i];
      var segPath = builtPath + '/';
      if (i === segments.length - 1) {
        html += '<span>/</span><span class="move-path-cur">' + escHtml(decodeURIComponent(segments[i])) + '</span>';
      } else {
        html += '<span>/</span><span class="move-path-seg" onclick="moveBrowseTo(\'' + escJs(segPath) + '\')">' + escHtml(decodeURIComponent(segments[i])) + '</span>';
      }
    }
    html += '</div>';

    // Folder list
    html += '<div class="move-folder-list">';

    // Parent directory link (go up)
    if (browsePath !== '/') {
      var parentParts = browsePath.replace(/\/$/, '').split('/');
      parentParts.pop();
      var parentPath = parentParts.join('/') || '/';
      if (!parentPath.endsWith('/')) parentPath += '/';
      html += '<div class="move-folder-item" onclick="moveBrowseTo(\'' + escJs(parentPath) + '\')">';
      html += '<span class="mf-icon">⬆️</span>';
      html += '<span class="mf-name">..</span>';
      html += '</div>';
    }

    // Filter entries to only show directories
    var folders = (data.entries || []).filter(function(e) { return e.is_dir; });

    // Exclude the folder being moved (cannot move into itself)
    if (moveTarget.isDir) {
      folders = folders.filter(function(e) {
        return e.name !== moveTarget.name || browsePath !== currentPath;
      });
    }

    if (folders.length === 0 && browsePath === '/') {
      html += '<div class="move-empty">No folders available</div>';
    } else {
      for (var j = 0; j < folders.length; j++) {
        var f = folders[j];
        var folderPath = browsePath + encodeURIComponent(f.name) + '/';
        html += '<div class="move-folder-item" onclick="moveBrowseTo(\'' + escJs(folderPath) + '\')">';
        html += '<span class="mf-icon">📁</span>';
        html += '<span class="mf-name">' + escHtml(f.name) + '</span>';
        html += '</div>';
      }
    }

    html += '</div>';
    browser.innerHTML = html;
  }

  window.moveBrowseTo = function(path) {
    loadMoveFolders(path);
  };

  async function doMove() {
    var err = document.getElementById('moveError');
    err.classList.remove('visible');

    // Cannot move to the same directory it's already in
    if (moveBrowsePath === currentPath || moveBrowsePath === currentPath + '/') {
      err.textContent = 'File is already in this folder';
      err.classList.add('visible');
      return;
    }

    // Cannot move a directory into itself or its children
    if (moveTarget.isDir) {
      var sourceDir = moveTarget.href;
      if (!sourceDir.endsWith('/')) sourceDir += '/';
      if (moveBrowsePath.indexOf(sourceDir) === 0) {
        err.textContent = 'Cannot move a folder into itself';
        err.classList.add('visible');
        return;
      }
    }

    // Build destination: browsePath + encoded filename
    var dest = moveBrowsePath;
    if (!dest.endsWith('/')) dest += '/';
    dest += encodeURIComponent(moveTarget.name);

    try {
      var resp = await webdavFetch(moveTarget.href, {
        method: 'MOVE',
        headers: buildHeaders({ 'Destination': dest })
      });
      if (!resp || (!resp.ok && resp.status !== 201 && resp.status !== 204)) {
        var msg = resp ? (resp.statusText || 'Failed') : 'Failed';
        showToast('Move failed: ' + msg, 'error');
        err.textContent = 'Move failed: ' + msg;
        err.classList.add('visible');
        return;
      }
      document.getElementById('moveDialog').classList.remove('active');
      showToast('Moved "' + moveTarget.name + '" successfully', 'success');
      loadDirectory(currentPath);
    } catch (e) {
      err.textContent = 'Move failed: ' + e.message;
      err.classList.add('visible');
    }
  }

  // Upload — button in header (persistent), drag-drop on container (persistent)
  (function initUpload() {
    var uploadBtn = document.getElementById('uploadBtn');
    var uploadInput = document.getElementById('uploadInput');
    if (!uploadBtn || !uploadInput) return;
    uploadBtn.addEventListener('click', function() { uploadInput.click(); });
    uploadInput.addEventListener('change', function() {
      if (this.files.length > 0) uploadFiles(this.files);
      this.value = '';
    });
    // Drag and drop on the container (element persists across renders)
    var container = document.querySelector('.container');
    if (!container) return;
    container.addEventListener('dragover', function(e) {
      if (!webdavEnabled) return;
      e.preventDefault();
      e.stopPropagation();
      container.classList.add('drop-zone-active');
    });
    container.addEventListener('dragleave', function(e) {
      e.preventDefault();
      e.stopPropagation();
      container.classList.remove('drop-zone-active');
    });
    container.addEventListener('drop', function(e) {
      if (!webdavEnabled) return;
      e.preventDefault();
      e.stopPropagation();
      container.classList.remove('drop-zone-active');
      if (e.dataTransfer.files.length > 0) {
        uploadFiles(e.dataTransfer.files);
      }
    });
  })();

  async function uploadFiles(files) {
    var total = files.length;
    var completed = 0;
    var failed = 0;
    var lastName = '';
    var uploadBtn = document.getElementById('uploadBtn');
    if (uploadBtn) uploadBtn.textContent = '0/' + total;

    for (var i = 0; i < files.length; i++) {
      var file = files[i];
      if (uploadBtn) uploadBtn.textContent = (i + 1) + '/' + total;
      var dest = currentPath;
      if (!dest.endsWith('/')) dest += '/';
      dest += encodeURIComponent(file.name);

      try {
        var resp = await webdavFetch(dest, {
          method: 'PUT',
          headers: buildHeaders({ 'Content-Type': file.type || 'application/octet-stream' }),
          body: file
        });
        if (resp && (resp.status === 201 || resp.status === 204)) {
          completed++;
          lastName = file.name;
        } else {
          failed++;
        }
      } catch (e) {
        failed++;
      }
    }

    if (uploadBtn) uploadBtn.innerHTML = '&#8593; Upload';

    if (failed > 0) {
      showToast('Upload: ' + completed + ' succeeded, ' + failed + ' failed', 'error');
    }
    if (completed > 0) {
      showToast(completed + ' file(s) uploaded', 'success');
      scheduleHighlight(lastName);
    }
    loadDirectory(currentPath);
  }

  // ===== Global keyboard handler =====
  document.addEventListener('keydown', function(e) {
    if (e.key === 'Escape') {
      closeModal();
      closeQrModal();
      if (document.getElementById('authDialog').classList.contains('active')) closeAuthDialog();
      document.getElementById('renameDialog').classList.remove('active');
      document.getElementById('deleteDialog').classList.remove('active');
      document.getElementById('moveDialog').classList.remove('active');
    }
    if (imageList.length > 1 && currentImageIndex >= 0) {
      if (e.key === 'ArrowLeft') navigateImage(-1);
      else if (e.key === 'ArrowRight') navigateImage(1);
    }
  });

  // ===== Global click delegation =====
  document.addEventListener('click', function(e) {
    // Close style menu
    if (!e.target.closest('#styleMenu') && !e.target.closest('#styleToggle')) {
      closeStyleMenu();
    }
    // Close more menus
    if (!e.target.closest('.more-wrap')) closeAllMore();
    // Client-side navigation
    var a = e.target.closest('a[data-nav]');
    if (a) {
      e.preventDefault();
      navigateTo(a.getAttribute('href'));
    }
  });

  // Initial load
  loadDirectory(location.pathname);
})();
</script>
</body>
</html>"##.to_string()
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
