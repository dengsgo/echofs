pub fn index_html() -> String {
    r##"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0, viewport-fit=cover">
<title>EchoFS</title>
<link rel="icon" type="image/svg+xml" href="data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='24' height='24' viewBox='0 0 24 24' fill='none'%3E%3Cpath d='M2 6a2 2 0 0 1 2-2h5l2 2h7a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V6z' stroke='%230071e3' stroke-width='1.8'/%3E%3Cpath d='M10 11a1.5 1.5 0 0 1 0 3' stroke='%230071e3' stroke-width='1.5' stroke-linecap='round'/%3E%3Cpath d='M12.5 9.5a4.5 4.5 0 0 1 0 6' stroke='%230071e3' stroke-width='1.5' stroke-linecap='round'/%3E%3Cpath d='M15 8a7 7 0 0 1 0 9' stroke='%230071e3' stroke-width='1.5' stroke-linecap='round'/%3E%3C/svg%3E">
<style>
:root {
  --bg: #ffffff;
  --bg-secondary: #f5f5f7;
  --text: #1d1d1f;
  --text-secondary: #6e6e73;
  --border: #d2d2d7;
  --accent: #0071e3;
  --accent-hover: #0077ED;
  --hover-bg: #f0f0f5;
  --modal-bg: rgba(0,0,0,0.6);
  --card-bg: #ffffff;
  --shadow: 0 2px 12px rgba(0,0,0,0.08);
  --header-bg: rgba(251, 251, 253, 0.72);
}
[data-theme="dark"] {
  --bg: #1d1d1f;
  --bg-secondary: #2d2d2f;
  --text: #f5f5f7;
  --text-secondary: #a1a1a6;
  --border: #424245;
  --accent: #2997ff;
  --accent-hover: #3da5ff;
  --hover-bg: #2d2d30;
  --card-bg: #2d2d2f;
  --shadow: 0 2px 12px rgba(0,0,0,0.3);
  --header-bg: rgba(37, 37, 39, 0.72);
}
* { margin:0; padding:0; box-sizing:border-box; }
body {
  font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
  background: var(--bg);
  color: var(--text);
  line-height: 1.5;
  min-height: 100vh;
  -webkit-text-size-adjust: 100%;
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
.theme-toggle {
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
.theme-toggle:hover { background: var(--hover-bg); }

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
.dir-link { color: var(--accent) !important; font-weight: 500; }
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
.btn:active { transform: scale(0.96); }
.btn-preview { margin-right: 4px; }
.copied {
  background: #30d158 !important;
  color: #fff !important;
  border-color: #30d158 !important;
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
.file-card:active { background: var(--hover-bg); margin: 0 -16px; padding: 12px 16px; border-radius: 8px; }
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
</style>
</head>
<body>
<div class="header">
  <div class="header-left">
    <a href="/" class="logo" data-nav><svg width="24" height="24" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg"><path d="M2 6a2 2 0 0 1 2-2h5l2 2h7a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V6z" stroke="currentColor" stroke-width="1.8" fill="none"/><path d="M10 11a1.5 1.5 0 0 1 0 3" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/><path d="M12.5 9.5a4.5 4.5 0 0 1 0 6" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/><path d="M15 8a7 7 0 0 1 0 9" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/></svg>EchoFS</a>
    <div class="breadcrumbs" id="breadcrumbs"></div>
  </div>
  <button class="theme-toggle" id="themeToggle" title="Toggle theme">&#9790;</button>
</div>
<div class="container">
  <div id="content"><div class="loading">Loading...</div></div>
</div>

<!-- Preview Modal -->
<div class="modal-overlay" id="modal">
  <div class="modal">
    <button class="modal-close" id="modalClose">&times;</button>
    <div class="modal-title" id="modalTitle"></div>
    <div id="modalContent"></div>
  </div>
</div>

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

  function isMobile() { return window.innerWidth <= 768; }

  // Theme
  function initTheme() {
    const saved = localStorage.getItem('echofs-theme');
    if (saved === 'dark' || (!saved && window.matchMedia('(prefers-color-scheme: dark)').matches)) {
      document.documentElement.setAttribute('data-theme', 'dark');
    }
    updateThemeButton();
  }
  function toggleTheme() {
    const current = document.documentElement.getAttribute('data-theme');
    const next = current === 'dark' ? 'light' : 'dark';
    if (next === 'dark') {
      document.documentElement.setAttribute('data-theme', 'dark');
    } else {
      document.documentElement.removeAttribute('data-theme');
    }
    localStorage.setItem('echofs-theme', next);
    updateThemeButton();
  }
  function updateThemeButton() {
    const btn = document.getElementById('themeToggle');
    const isDark = document.documentElement.getAttribute('data-theme') === 'dark';
    btn.innerHTML = isDark ? '&#9788;' : '&#9790;';
  }
  document.getElementById('themeToggle').addEventListener('click', toggleTheme);
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

    const apiPath = path === '/' ? '/api/ls' : '/api/ls' + path;
    try {
      const resp = await fetch(apiPath);
      if (!resp.ok) throw new Error('Failed to load directory');
      const data = await resp.json();
      currentEntries = data.entries;
      renderBreadcrumbs(data.breadcrumbs);
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
    if (sorted.length === 0) {
      content.innerHTML = '<div class="empty-state">Empty directory</div>';
      return;
    }
    let html = renderSortBar();
    html += renderTable(sorted);
    html += renderCardList(sorted);
    content.innerHTML = html;
    attachSortHandlers();
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

      let actions = '<button class="btn" onclick="copyLink(\'' + escJs(e.href) + '\', this)">Copy Link</button>';
      if (e.media_type === 'video' || e.media_type === 'audio' || e.media_type === 'image') {
        actions = '<button class="btn btn-preview" onclick="previewMedia(\'' + escJs(e.href) + '\', \'' + escJs(e.name) + '\', \'' + escJs(e.media_type) + '\')">Preview</button>' + actions;
      }

      html += '<tr class="' + cls + '">';
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

      let actions = '<button class="btn" onclick="event.stopPropagation();copyLink(\'' + escJs(e.href) + '\', this)">Copy</button>';
      if (e.media_type === 'video' || e.media_type === 'audio' || e.media_type === 'image') {
        actions = '<button class="btn btn-preview" onclick="event.stopPropagation();previewMedia(\'' + escJs(e.href) + '\', \'' + escJs(e.name) + '\', \'' + escJs(e.media_type) + '\')">Play</button>' + actions;
      }

      html += '<div class="file-card">';
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

  // Copy link
  window.copyLink = function(href, btn) {
    const url = location.origin + href;
    navigator.clipboard.writeText(url).then(() => {
      btn.classList.add('copied');
      const orig = btn.textContent;
      btn.textContent = 'Copied!';
      setTimeout(() => { btn.classList.remove('copied'); btn.textContent = orig; }, 1500);
    }).catch(() => {
      const ta = document.createElement('textarea');
      ta.value = url;
      document.body.appendChild(ta);
      ta.select();
      document.execCommand('copy');
      document.body.removeChild(ta);
      btn.classList.add('copied');
      const orig = btn.textContent;
      btn.textContent = 'Copied!';
      setTimeout(() => { btn.classList.remove('copied'); btn.textContent = orig; }, 1500);
    });
  };

  // Media preview
  window.previewMedia = function(href, name, type) {
    const modal = document.getElementById('modal');
    const title = document.getElementById('modalTitle');
    const mc = document.getElementById('modalContent');
    title.textContent = name;
    mc.innerHTML = '';

    if (type === 'video') {
      mc.innerHTML = '<video controls autoplay playsinline><source src="' + escHtml(href) + '">Your browser does not support video playback.</video>';
    } else if (type === 'audio') {
      mc.innerHTML = '<audio controls autoplay><source src="' + escHtml(href) + '">Your browser does not support audio playback.</audio>';
    } else if (type === 'image') {
      mc.innerHTML = '<img src="' + escHtml(href) + '" alt="' + escHtml(name) + '">';
    }
    modal.classList.add('active');
    document.body.style.overflow = 'hidden';
  };

  // Modal close
  document.getElementById('modalClose').addEventListener('click', closeModal);
  document.getElementById('modal').addEventListener('click', function(e) {
    if (e.target === this) closeModal();
  });
  document.addEventListener('keydown', function(e) {
    if (e.key === 'Escape') closeModal();
  });

  function closeModal() {
    const modal = document.getElementById('modal');
    modal.classList.remove('active');
    document.body.style.overflow = '';
    const mc = document.getElementById('modalContent');
    mc.querySelectorAll('video, audio').forEach(el => { el.pause(); el.src = ''; });
    mc.innerHTML = '';
  }

  // Client-side navigation
  document.addEventListener('click', function(e) {
    const a = e.target.closest('a[data-nav]');
    if (a) {
      e.preventDefault();
      navigateTo(a.getAttribute('href'));
    }
  });

  function escHtml(s) {
    const d = document.createElement('div');
    d.textContent = s;
    return d.innerHTML;
  }
  function escJs(s) {
    return s.replace(/\\/g, '\\\\').replace(/'/g, "\\'").replace(/"/g, '\\"');
  }

  // Initial load
  loadDirectory(location.pathname);
})();
</script>
</body>
</html>"##.to_string()
}
