(function() {
  const ICONS = {
    folder: '🗂️',
    video: '🎥',
    audio: '🎵',
    image: '🖼️',
    text: '📄',
    pdf: '📄',
    archive: '📦',
    document: '📃',
    spreadsheet: '📊',
    presentation: '📊',
    code: '💻',
    file: '📄',
  };

  const SORT_LABELS = { name: 'Name', size: 'Size', created: 'Created', modified: 'Modified' };
  // Plyr (video player) is loaded from CDN on first video preview only.
  // The bulk of users browsing directories never trigger this, so paying
  // ~80KB JS + ~16KB CSS upfront would be waste. plyrLoaderPromise caches
  // the in-flight or resolved load so concurrent previews share one fetch.
  //
  // When the CDN is unreachable (offline LAN, blocked region, broken DNS,
  // TCP black-hole) we fall back to the native <video controls> that's
  // already rendered into the modal — Plyr only enhances an existing video
  // element, so the placeholder works as a fully functional fallback.
  // After the first failure we set plyrUnavailable=true for the rest of the
  // session; further previews skip the load attempt entirely (no 8s wait
  // per preview).
  const PLYR_JS = 'https://cdn.plyr.io/3.8.4/plyr.js';
  const PLYR_CSS = 'https://cdn.plyr.io/3.8.4/plyr.css';
  const PLYR_LOAD_TIMEOUT_MS = 8000;
  let plyrLoaderPromise = null;
  let plyrUnavailable = false;
  let currentPlyr = null;
  let currentPlyrCleanup = null;
  let currentEntries = [];
  let sortField = 'name';
  let sortAsc = true;
  let imageList = [];
  let currentImageIndex = -1;
  let webdavEnabled = false;
  let webdavAuth = false;
  let currentPath = '/';
  let viewMode = 'list'; // 'list' or 'grid'

  // Lazily inject Plyr <link> + <script>. Returns a promise that resolves to
  // the global Plyr constructor, or rejects on failure (load error, missing
  // global after load, or PLYR_LOAD_TIMEOUT_MS elapsed without onload firing —
  // the timeout case covers TCP black-holes and stalled CDNs that never
  // trigger script.onerror). On rejection plyrLoaderPromise is cleared AND
  // the <script> tag is removed from <head> so a retry doesn't stack
  // duplicate nodes / duplicate network requests; a stale <script> arriving
  // after the timeout would also still fire onload and try to call our
  // resolver, which the `settled` guard ignores. The CSS <link> is left in
  // place: it's idempotent (browser de-dupes by URL on the next attempt is
  // not guaranteed, but a leftover stylesheet doesn't break anything and
  // may even still resolve in the background).
  function loadPlyr() {
    if (window.Plyr) return Promise.resolve(window.Plyr);
    if (plyrLoaderPromise) return plyrLoaderPromise;
    plyrLoaderPromise = new Promise(function(resolve, reject) {
      var settled = false;
      var script = null;
      function detachScript() {
        if (script && script.parentNode) script.parentNode.removeChild(script);
        // Null the handlers so a late-arriving onload (CDN finally responded
        // after the timeout) is a no-op even if the node somehow survives.
        if (script) { script.onload = null; script.onerror = null; }
      }
      function fail(err) {
        if (settled) return;
        settled = true;
        plyrLoaderPromise = null;
        detachScript();
        reject(err);
      }
      function succeed(P) {
        if (settled) return;
        settled = true;
        resolve(P);
      }

      var link = document.createElement('link');
      link.rel = 'stylesheet';
      link.href = PLYR_CSS;
      // CSS load failure is non-fatal for playback (Plyr still works
      // unstyled, and the native <video> below is already visible) — log
      // it so users can see why the player looks unstyled.
      link.onerror = function() { console.warn('Plyr CSS failed to load from CDN'); };
      document.head.appendChild(link);

      script = document.createElement('script');
      script.src = PLYR_JS;
      script.async = true;
      script.onload = function() {
        if (window.Plyr) succeed(window.Plyr);
        else fail(new Error('Plyr global missing after script load'));
      };
      script.onerror = function() {
        fail(new Error('Plyr JS failed to load from CDN'));
      };
      document.head.appendChild(script);

      // Hard timeout: some failure modes (DNS black-hole, hung TCP, captive
      // portal, MITM proxy that swallows requests) never fire onerror. Cap
      // the wait so the user isn't stuck staring at a dead modal.
      setTimeout(function() {
        fail(new Error('Plyr load timed out after ' + PLYR_LOAD_TIMEOUT_MS + 'ms'));
      }, PLYR_LOAD_TIMEOUT_MS);
    });
    return plyrLoaderPromise;
  }

  // Long-press the right half of the video to play at BOOST_RATE× speed
  // (currently 3×) — releases back to the original speed. Mirrors the
  // gesture popularised by YouTube / Bilibili. Plyr exposes `player.speed`;
  // everything else is hand-rolled.
  //
  // Tricky bits handled here:
  //   - Listen on the `.plyr` container, NOT the <video>. Plyr stacks a
  //     poster div, a big "overlaid" play button, and a controls bar as
  //     SIBLINGS of the <video> inside `.plyr`. Pointer events on those
  //     overlays never bubble to the <video> — listeners attached there
  //     get no events. (This was the bug in v1 of this gesture.)
  //   - Trigger only on the right half (relative to the <video> element,
  //     so letterbox bars don't shift the midline) so we don't fight with
  //     the eventual "double-tap left to seek backward" gesture.
  //   - Use Pointer Events so one code path covers mouse + touch + pen.
  //   - 500ms hold threshold filters accidental taps; 10px move threshold
  //     filters scroll/drag attempts.
  //   - Ignore presses that land on Plyr's own controls (the bottom bar,
  //     the big play button) — those need their normal click behaviour.
  //   - Suppress the synthetic click that fires on pointerup after a long
  //     press, otherwise Plyr would interpret it as "toggle pause".
  //   - Restore whatever speed the user had set before the gesture, not a
  //     hardcoded 1.0 — they may be watching at 1.5× already.
  //   - Returns a cleanup function; closeModal() calls it to remove the
  //     document-level listeners and the floating indicator element.
  function attachHoldToSpeedUp(player) {
    var video = player.media; // the underlying <video>
    var container = player.elements && player.elements.container;
    if (!video || !container) return function() {};
    var HOLD_MS = 500;
    var MOVE_TOLERANCE = 10;
    var BOOST_RATE = 3;
    var holdTimer = null;
    var active = false;
    var pointerId = null;
    var startX = 0, startY = 0;
    var savedSpeed = 1;
    var suppressNextClick = false;

    // Floating "▶▶ N×" indicator (text built from BOOST_RATE so it tracks
    // the constant). Created lazily on first activation so non-video
    // previews (audio, image) never see this element.
    var indicator = null;
    function ensureIndicator() {
      if (indicator) return indicator;
      indicator = document.createElement('div');
      indicator.className = 'plyr-speed-indicator';
      indicator.textContent = '▶▶ ' + BOOST_RATE + '×';
      document.body.appendChild(indicator);
      return indicator;
    }
    function showIndicator() { ensureIndicator().classList.add('visible'); }
    function hideIndicator() { if (indicator) indicator.classList.remove('visible'); }

    function onPointerDown(e) {
      // Only primary button for mouse; touch/pen always primary.
      if (e.pointerType === 'mouse' && e.button !== 0) return;
      // Don't hijack presses on Plyr's own UI (controls bar, big play btn).
      // Plyr's controls all carry one of these classes.
      if (e.target.closest('.plyr__controls, .plyr__control, .plyr__menu')) return;
      // Don't trigger when paused — boosting a paused video is meaningless
      // and would surprise users tapping to play.
      if (video.paused) return;
      // Right half only — measured against the <video> rect (not the .plyr
      // container) so letterbox bars on either side don't shift the line.
      var rect = video.getBoundingClientRect();
      if (rect.width === 0) return; // video not laid out yet
      if (e.clientX < rect.left + rect.width / 2) return;
      // Suppress text selection / image drag during the press. preventDefault
      // on pointerdown also keeps mobile from showing the long-press menu.
      e.preventDefault();

      pointerId = e.pointerId;
      startX = e.clientX;
      startY = e.clientY;
      holdTimer = setTimeout(function() {
        holdTimer = null;
        active = true;
        savedSpeed = player.speed || 1;
        try { player.speed = BOOST_RATE; } catch (_) {}
        showIndicator();
      }, HOLD_MS);
    }
    function onPointerMove(e) {
      if (e.pointerId !== pointerId) return;
      // If the user starts dragging before the hold timer fires, abort —
      // they're probably trying to scroll or seek.
      if (holdTimer) {
        var dx = e.clientX - startX;
        var dy = e.clientY - startY;
        if (Math.abs(dx) > MOVE_TOLERANCE || Math.abs(dy) > MOVE_TOLERANCE) {
          clearTimeout(holdTimer);
          holdTimer = null;
          pointerId = null;
        }
      }
    }
    function endHold(e) {
      if (e && e.pointerId !== pointerId) return;
      if (holdTimer) { clearTimeout(holdTimer); holdTimer = null; }
      if (active) {
        active = false;
        try { player.speed = savedSpeed; } catch (_) {}
        hideIndicator();
        // The pointerup that ended a long-press will be followed by a
        // synthetic `click` on the same target. Plyr listens for click on
        // the container to toggle pause, which would be wrong here — swallow it.
        suppressNextClick = true;
      }
      pointerId = null;
    }
    function onClickCapture(e) {
      if (suppressNextClick) {
        suppressNextClick = false;
        e.stopPropagation();
        e.preventDefault();
      }
    }

    // Listen on the .plyr container (not the <video>) — see comment above.
    container.addEventListener('pointerdown', onPointerDown);
    // pointermove / pointerup / pointercancel listen on document so we still
    // get the release event if the pointer drifts off the player.
    document.addEventListener('pointermove', onPointerMove);
    document.addEventListener('pointerup', endHold);
    document.addEventListener('pointercancel', endHold);
    // Capture-phase click handler on the container — beats Plyr's bubbled
    // click → toggle-pause handler.
    container.addEventListener('click', onClickCapture, true);

    return function cleanup() {
      if (holdTimer) { clearTimeout(holdTimer); holdTimer = null; }
      if (active) {
        // Mid-gesture cleanup (e.g. modal closed while held) — restore speed.
        try { player.speed = savedSpeed; } catch (_) {}
        active = false;
      }
      container.removeEventListener('pointerdown', onPointerDown);
      document.removeEventListener('pointermove', onPointerMove);
      document.removeEventListener('pointerup', endHold);
      document.removeEventListener('pointercancel', endHold);
      container.removeEventListener('click', onClickCapture, true);
      if (indicator && indicator.parentNode) indicator.parentNode.removeChild(indicator);
      indicator = null;
    };
  }

  // View mode (list/grid)
  function initViewMode() {
    var saved = localStorage.getItem('echofs-view');
    viewMode = (saved === 'grid') ? 'grid' : 'list';
    applyViewMode();
  }
  function applyViewMode() {
    if (viewMode === 'grid') {
      document.documentElement.setAttribute('data-view', 'grid');
    } else {
      document.documentElement.removeAttribute('data-view');
    }
    updateViewToggleIcon();
  }
  function updateViewToggleIcon() {
    var btn = document.getElementById('viewToggle');
    if (!btn) return;
    if (viewMode === 'grid') {
      // currently grid → button shows "switch to list" icon (lines)
      btn.innerHTML = '<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="8" y1="6" x2="21" y2="6"/><line x1="8" y1="12" x2="21" y2="12"/><line x1="8" y1="18" x2="21" y2="18"/><line x1="3" y1="6" x2="3.01" y2="6"/><line x1="3" y1="12" x2="3.01" y2="12"/><line x1="3" y1="18" x2="3.01" y2="18"/></svg>';
      btn.setAttribute('title', 'Switch to list view');
    } else {
      // currently list → button shows "switch to grid" icon (squares)
      btn.innerHTML = '<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="3" width="7" height="7" rx="1"/><rect x="14" y="3" width="7" height="7" rx="1"/><rect x="3" y="14" width="7" height="7" rx="1"/><rect x="14" y="14" width="7" height="7" rx="1"/></svg>';
      btn.setAttribute('title', 'Switch to grid view');
    }
  }
  function toggleViewMode() {
    viewMode = (viewMode === 'grid') ? 'list' : 'grid';
    localStorage.setItem('echofs-view', viewMode);
    applyViewMode();
    // Re-render so only the active view's DOM is mounted.
    if (currentEntries.length > 0) {
      _lastRenderMode = currentRenderMode();
      sortAndRender();
    }
  }

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
    btn.innerHTML = isDark ? '☀' : '☾';
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
  document.getElementById('viewToggle').addEventListener('click', toggleViewMode);
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
  initViewMode();

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

  // Determine which single view to render right now.
  // - 'grid'  : grid view explicitly chosen
  // - 'cards' : list mode AND viewport is mobile (<= 768px)
  // - 'table' : list mode AND viewport is desktop (> 768px)
  // We render only ONE view at a time (vs. previously rendering all three and
  // hiding two via CSS) — saves 2/3 of DOM nodes on large directories and
  // avoids data-entry-name duplication that broke highlightEntry().
  function currentRenderMode() {
    if (viewMode === 'grid') return 'grid';
    return (window.innerWidth <= 768) ? 'cards' : 'table';
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
    var mode = currentRenderMode();
    if (mode === 'grid')       html += renderGrid(sorted);
    else if (mode === 'cards') html += renderCardList(sorted);
    else                       html += renderTable(sorted);
    content.innerHTML = html;
    attachSortHandlers();
    // Highlight entry after render if pending — use double rAF so layout
    // is settled (replaces the magic-number 100ms setTimeout).
    if (pendingHighlight) {
      var name = pendingHighlight;
      pendingHighlight = null;
      requestAnimationFrame(function() {
        requestAnimationFrame(function() { highlightEntry(name); });
      });
    }
  }

  // Re-render when crossing the 768px breakpoint so the correct single view is mounted.
  // Debounced via rAF to avoid thrashing during continuous resize drags.
  var _lastRenderMode = null;
  var _resizePending = false;
  window.addEventListener('resize', function() {
    if (_resizePending) return;
    _resizePending = true;
    requestAnimationFrame(function() {
      _resizePending = false;
      var mode = currentRenderMode();
      if (mode !== _lastRenderMode && currentEntries.length > 0) {
        _lastRenderMode = mode;
        sortAndRender();
      }
    });
  });

  function renderSortBar() {
    let html = '<div class="sort-bar">';
    html += '<span class="sort-bar-label">Sort:</span>';
    for (const [key, label] of Object.entries(SORT_LABELS)) {
      const active = sortField === key;
      const arrow = active ? (sortAsc ? ' ▲' : ' ▼') : '';
      html += '<button class="sort-chip' + (active ? ' active' : '') + '" data-sort="' + key + '">' + label + arrow + '</button>';
    }
    html += '</div>';
    return html;
  }

  function arrow(field) {
    if (field !== sortField) return '';
    return '<span class="sort-arrow">' + (sortAsc ? '▲' : '▼') + '</span>';
  }

  // Build action buttons. Uses data-action attributes so a single document-level
  // click handler (see end of script) can dispatch — this keeps file names out
  // of HTML/JS double-context (no XSS via filename), and keeps the global
  // namespace clean (no window-level globals for action handlers).
  function buildActions(e, isCard) {
    var hasPreview = (e.media_type === 'video' || e.media_type === 'audio' || e.media_type === 'image');
    // Common data attributes for an entry — escHtml encodes ", ', <, >, &.
    var d = ' data-href="' + escHtml(e.href) + '"'
          + ' data-name="' + escHtml(e.name) + '"'
          + ' data-is-dir="' + (e.is_dir ? '1' : '0') + '"'
          + (e.media_type ? ' data-media="' + escHtml(e.media_type) + '"' : '');

    // Primary buttons (max 2 shown)
    var primary = [];
    if (hasPreview) {
      primary.push('<button class="btn btn-preview" data-action="preview"' + d + '>' + (isCard ? 'Play' : 'Preview') + '</button>');
      primary.push('<button class="btn" data-action="copy"' + d + '>' + (isCard ? 'Copy' : 'Copy Link') + '</button>');
    } else {
      primary.push('<button class="btn" data-action="copy"' + d + '>' + (isCard ? 'Copy' : 'Copy Link') + '</button>');
      primary.push('<button class="btn" data-action="qr"' + d + '>QR</button>');
    }

    // More menu items
    var moreItems = [];
    if (hasPreview) {
      moreItems.push({ label: 'QR Code', action: 'qr' });
    }
    if (webdavEnabled) {
      moreItems.push({ label: 'Rename', action: 'rename' });
      moreItems.push({ label: 'Move to...', action: 'move' });
      moreItems.push({ label: 'Delete', action: 'delete', cls: 'more-item-danger' });
    }

    var html = primary.join('');

    if (moreItems.length > 0) {
      html += '<div class="more-wrap"><button class="btn btn-more" data-action="more-toggle">⋯</button>';
      html += '<div class="more-menu">';
      for (var i = 0; i < moreItems.length; i++) {
        var item = moreItems[i];
        html += '<button class="more-item' + (item.cls ? ' ' + item.cls : '') + '" data-action="' + item.action + '"' + d + '>' + escHtml(item.label) + '</button>';
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

  function renderGrid(entries) {
    let html = '<div class="file-grid">';
    for (const e of entries) {
      const icon = ICONS[e.icon] || ICONS.file;
      const linkCls = e.is_dir ? 'grid-tile-link dir-link' : 'grid-tile-link';
      const navAttr = e.is_dir ? ' data-nav' : '';
      // Image files: show actual thumbnail; everything else (incl. folders): show big emoji icon.
      let visual;
      if (!e.is_dir && e.media_type === 'image') {
        visual = '<img class="grid-tile-img" src="' + escHtml(e.href) + '" alt="' + escHtml(e.name) + '" loading="lazy">';
      } else {
        visual = '<div class="grid-tile-icon">' + icon + '</div>';
      }
      // For media files in grid view, link clicks open preview instead of download.
      // We add data-tile-preview to mark this — handled by document click delegation.
      let tilePreviewAttr = '';
      if (!e.is_dir && (e.media_type === 'video' || e.media_type === 'audio' || e.media_type === 'image')) {
        tilePreviewAttr = ' data-tile-preview="1" data-media="' + escHtml(e.media_type) + '" data-name="' + escHtml(e.name) + '"';
      }
      html += '<div class="grid-tile" data-entry-name="' + escHtml(e.name) + '">';
      html += '<a href="' + escHtml(e.href) + '" class="' + linkCls + '"' + navAttr + tilePreviewAttr + '>';
      html += visual;
      html += '<div class="grid-tile-name">' + escHtml(e.name) + '</div>';
      html += '</a>';
      // Floating actions menu in top-right corner of the tile (more menu only)
      html += '<div class="grid-tile-actions">';
      html += renderTileMore(e);
      html += '</div>';
      html += '</div>';
    }
    html += '</div>';
    return html;
  }

  function renderTileMore(e) {
    var hasPreview = (e.media_type === 'video' || e.media_type === 'audio' || e.media_type === 'image');
    var d = ' data-href="' + escHtml(e.href) + '"'
          + ' data-name="' + escHtml(e.name) + '"'
          + ' data-is-dir="' + (e.is_dir ? '1' : '0') + '"'
          + (e.media_type ? ' data-media="' + escHtml(e.media_type) + '"' : '');
    var items = [];
    if (hasPreview) {
      items.push({ label: 'Preview', action: 'preview' });
    }
    if (!e.is_dir) {
      items.push({ label: 'Download', action: 'download' });
    }
    items.push({ label: 'Copy Link', action: 'copy' });
    items.push({ label: 'QR Code', action: 'qr' });
    if (webdavEnabled) {
      items.push({ label: 'Rename', action: 'rename' });
      items.push({ label: 'Move to...', action: 'move' });
      items.push({ label: 'Delete', action: 'delete', cls: 'more-item-danger' });
    }
    var html = '<div class="more-wrap"><button class="btn btn-more grid-more-btn" data-action="more-toggle">⋯</button>';
    html += '<div class="more-menu">';
    for (var i = 0; i < items.length; i++) {
      var it = items[i];
      html += '<button class="more-item' + (it.cls ? ' ' + it.cls : '') + '" data-action="' + it.action + '"' + d + '>' + escHtml(it.label) + '</button>';
    }
    html += '</div></div>';
    return html;
  }

  function attachSortHandlers() {
    document.querySelectorAll('[data-sort]').forEach(el => {
      el.addEventListener('click', () => {
        const field = el.getAttribute('data-sort');
        if (sortField === field) sortAsc = !sortAsc;
        else { sortField = field; sortAsc = true; }
        // Preserve scroll across the re-render — sorting should reorder the
        // list in place, not jump the user back to the top of a 1000-row table.
        var savedScroll = window.scrollY;
        sortAndRender();
        window.scrollTo(0, savedScroll);
      });
    });
  }

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

  // QR Code modal — internal function, dispatched via data-action="qr".
  function showQr(href) {
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
  }

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
  function copyLink(href, btn) {
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
  }

  // Media preview — internal function, dispatched via data-action="preview".
  function previewMedia(href, name, type) {
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
      // Render a fully functional native <video> immediately. This serves
      // two purposes: (1) the modal opens with no perceptible delay even
      // while Plyr is still loading from CDN, and (2) it IS the fallback —
      // if Plyr load fails / times out / throws, we just leave this element
      // in place and the user gets browser-native controls. Mark the
      // container with data-fallback so future code can tell which mode
      // is active without poking at DOM internals.
      mc.dataset.fallback = '0';
      mc.innerHTML = '<video id="plyrVideo" controls autoplay playsinline style="max-width:88vw;max-height:80vh;display:block;"><source src="' + escHtml(href) + '">Your browser does not support video playback.</video>';
      var videoEl = mc.querySelector('#plyrVideo');

      // Skip the load attempt entirely if a previous preview already proved
      // the CDN is unreachable — no point making the user wait 8 seconds
      // each time. The native <video> is already playing.
      if (plyrUnavailable) {
        mc.dataset.fallback = '1';
      } else {
        loadPlyr().then(function(Plyr) {
          // Bail if the user closed the modal (or moved to another preview)
          // before Plyr finished loading.
          if (!videoEl.isConnected) return;
          // Destroy any previous instance — happens if a video preview is
          // immediately replaced by another video preview.
          if (currentPlyrCleanup) { try { currentPlyrCleanup(); } catch (_) {} currentPlyrCleanup = null; }
          if (currentPlyr) { try { currentPlyr.destroy(); } catch (_) {} currentPlyr = null; }
          // Plyr's constructor can also throw (rare: corrupted CDN response,
          // browser quirk). Catch here too so a broken Plyr never breaks
          // playback — fall back to the native controls already showing.
          try {
            currentPlyr = new Plyr(videoEl, {
              autoplay: true,
              controls: ['play-large', 'play', 'progress', 'current-time', 'duration', 'mute', 'volume', 'settings', 'pip', 'airplay', 'fullscreen'],
            });
          } catch (err) {
            if (window.console) console.warn('Plyr construction failed, using native player:', err);
            mc.dataset.fallback = '1';
            plyrUnavailable = true;
            return;
          }
          // Hold-to-speed-up gesture on the right half of the video. Wait for
          // Plyr's ready event — `player.elements.container` doesn't exist
          // synchronously after `new Plyr()`. The closed-over `currentPlyr`
          // reference is the one we attach to (saved into a local so a fast
          // user closing the modal doesn't make us bind to a stale player).
          var playerForGesture = currentPlyr;
          playerForGesture.on('ready', function() {
            if (currentPlyr !== playerForGesture) return; // superseded
            if (currentPlyrCleanup) { try { currentPlyrCleanup(); } catch (_) {} }
            currentPlyrCleanup = attachHoldToSpeedUp(playerForGesture);
          });
        }).catch(function(err) {
          // CDN unreachable / timeout / global-missing. The native <video>
          // already mounted is fine on its own; just record the failure so
          // future previews skip the wait, and tell the user once.
          if (window.console) console.warn('Plyr unavailable, falling back to native player:', err);
          if (videoEl.isConnected) mc.dataset.fallback = '1';
          var firstTime = !plyrUnavailable;
          plyrUnavailable = true;
          if (firstTime) {
            // Surface the fallback once per session — silent fallback
            // would leave users wondering why the player looks different.
            try { showToast('Enhanced player unavailable, using native video', 'info'); } catch (_) {}
          }
        });
      }
    } else if (type === 'audio') {
      mc.innerHTML = '<audio controls autoplay><source src="' + escHtml(href) + '">Your browser does not support audio playback.</audio>';
    } else if (type === 'image') {
      mc.innerHTML = '<img src="' + escHtml(href) + '" alt="' + escHtml(name) + '">';
      updateImageCounter(title, name);
    }
    updateNavButtons();
    modal.classList.add('active');
    document.body.style.overflow = 'hidden';
  }

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
    // Tear down Plyr first — its destroy() restores the underlying <video>,
    // which we then nuke along with the rest of the modal contents.
    if (currentPlyrCleanup) { try { currentPlyrCleanup(); } catch (_) {} currentPlyrCleanup = null; }
    if (currentPlyr) { try { currentPlyr.destroy(); } catch (_) {} currentPlyr = null; }
    mc.querySelectorAll('video, audio').forEach(el => { el.pause(); el.src = ''; });
    mc.innerHTML = '';
    delete mc.dataset.fallback;
    imageList = [];
    currentImageIndex = -1;
    updateNavButtons();
  }

  // HTML escape — encodes all 5 dangerous characters (&, <, >, ", ').
  // Pure string-replace is ~10x faster than DOM-roundtrip and crucially also
  // encodes quote characters, which are required when output is embedded
  // inside HTML attribute values (href="...", data-*="...", alt="...", etc.).
  function escHtml(s) {
    return String(s == null ? '' : s)
      .replace(/&/g, '&amp;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;')
      .replace(/"/g, '&quot;')
      .replace(/'/g, '&#39;');
  }

  // More dropdown management — internal, called from click delegation.
  function toggleMore(btn) {
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
  }
  function closeAllMore() {
    document.querySelectorAll('.more-menu.active').forEach(function(m) {
      m.classList.remove('active');
      m.classList.remove('more-menu-up');
    });
  }

  // ===== File Management Operations =====

  // Toast notifications. Cap at MAX_TOASTS visible at once — a 50-file upload
  // used to spew 50 stacked toasts that flowed off-screen. When a new one
  // arrives at the cap, drop the oldest immediately.
  var MAX_TOASTS = 3;
  function showToast(message, type) {
    var container = document.getElementById('toastContainer');
    // Drop oldest toasts so we never exceed MAX_TOASTS - 1 BEFORE adding the new one.
    while (container.children.length >= MAX_TOASTS) {
      container.removeChild(container.firstChild);
    }
    var toast = document.createElement('div');
    toast.className = 'toast toast-' + (type || 'success');
    toast.textContent = message;
    container.appendChild(toast);
    setTimeout(function() {
      toast.style.animation = 'toastOut 0.25s ease forwards';
      setTimeout(function() {
        if (toast.parentNode) toast.remove();
      }, 250);
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

  // Map HTTP status to a human message. HTTP/2 strips reason-phrase, so
  // resp.statusText is empty under H2 — relying on it gives users a useless
  // "Failed: " toast. We prefer status codes mapped to action-relevant text.
  var STATUS_MSG = {
    400: 'Bad request',
    401: 'Authentication required',
    403: 'Permission denied',
    404: 'Not found',
    405: 'Operation not allowed',
    409: 'Conflict (target exists or parent missing)',
    412: 'Precondition failed',
    413: 'File too large',
    415: 'Unsupported media type',
    423: 'Locked',
    500: 'Server error',
    502: 'Bad gateway',
    503: 'Service unavailable',
    507: 'Insufficient storage'
  };
  function statusMessage(resp) {
    if (!resp) return 'Network error';
    if (resp.ok) return 'OK';
    var code = resp.status;
    var label = STATUS_MSG[code] || (resp.statusText || 'Request failed');
    return label + ' (' + code + ')';
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

  function renameEntry(href, name) {
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
  }

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
        var msg = statusMessage(resp);
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

  function deleteEntry(href, name, isDir) {
    deleteTarget = { href: href, name: name, isDir: isDir };
    var msg = document.getElementById('deleteMsg');
    var err = document.getElementById('deleteError');
    err.classList.remove('visible');
    err.textContent = '';
    msg.textContent = isDir
      ? 'Delete folder "' + name + '" and all its contents?'
      : 'Delete "' + name + '"?';
    document.getElementById('deleteDialog').classList.add('active');
  }

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
        var msg = statusMessage(resp);
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

  function moveEntry(href, name, isDir) {
    moveTarget = { href: href, name: name, isDir: isDir };
    var err = document.getElementById('moveError');
    err.classList.remove('visible');
    err.textContent = '';
    // Start browsing from the current directory's parent or current directory
    moveBrowsePath = currentPath;
    if (!moveBrowsePath.endsWith('/')) moveBrowsePath += '/';
    document.getElementById('moveDialog').classList.add('active');
    loadMoveFolders(moveBrowsePath);
  }

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
    html += '<span class="move-path-seg" data-action="move-browse-to" data-path="/">Root</span>';
    var builtPath = '';
    for (var i = 0; i < segments.length; i++) {
      builtPath += '/' + segments[i];
      var segPath = builtPath + '/';
      if (i === segments.length - 1) {
        html += '<span>/</span><span class="move-path-cur">' + escHtml(decodeURIComponent(segments[i])) + '</span>';
      } else {
        html += '<span>/</span><span class="move-path-seg" data-action="move-browse-to" data-path="' + escHtml(segPath) + '">' + escHtml(decodeURIComponent(segments[i])) + '</span>';
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
      html += '<div class="move-folder-item" data-action="move-browse-to" data-path="' + escHtml(parentPath) + '">';
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
        html += '<div class="move-folder-item" data-action="move-browse-to" data-path="' + escHtml(folderPath) + '">';
        html += '<span class="mf-icon">📁</span>';
        html += '<span class="mf-name">' + escHtml(f.name) + '</span>';
        html += '</div>';
      }
    }

    html += '</div>';
    browser.innerHTML = html;
  }

  function moveBrowseTo(path) {
    loadMoveFolders(path);
  }

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
        var msg = statusMessage(resp);
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
    // Drag and drop on the container (element persists across renders).
    // Use enter/leave depth counter — dragenter/dragleave fire on every child
    // boundary crossing, so a naive add/remove makes the outline flicker.
    var container = document.querySelector('.container');
    if (!container) return;
    var dragDepth = 0;
    container.addEventListener('dragenter', function(e) {
      if (!webdavEnabled) return;
      e.preventDefault();
      e.stopPropagation();
      dragDepth++;
      container.classList.add('drop-zone-active');
    });
    container.addEventListener('dragover', function(e) {
      if (!webdavEnabled) return;
      // Must preventDefault so 'drop' can fire — but no class change here.
      e.preventDefault();
      e.stopPropagation();
    });
    container.addEventListener('dragleave', function(e) {
      if (!webdavEnabled) return;
      e.preventDefault();
      e.stopPropagation();
      dragDepth = Math.max(0, dragDepth - 1);
      if (dragDepth === 0) container.classList.remove('drop-zone-active');
    });
    container.addEventListener('drop', function(e) {
      if (!webdavEnabled) return;
      e.preventDefault();
      e.stopPropagation();
      dragDepth = 0;
      container.classList.remove('drop-zone-active');
      if (e.dataTransfer.files.length > 0) {
        uploadFiles(e.dataTransfer.files);
      }
    });
  })();

  // Concurrency for parallel uploads. The browser caps to ~6 connections
  // per origin on H1 and many more on H2, but we keep this conservative so
  // a single slow file doesn't starve the rest, and so we don't trigger
  // throttle/server-side rate limits.
  var UPLOAD_CONCURRENCY = 4;

  async function uploadFiles(files) {
    var total = files.length;
    if (total === 0) return;
    var completed = 0;
    var failed = 0;
    var lastName = '';
    var queueIndex = 0;
    var uploadBtn = document.getElementById('uploadBtn');

    function updateBtn() {
      if (uploadBtn) uploadBtn.textContent = (completed + failed) + '/' + total;
    }
    updateBtn();

    async function uploadOne(file) {
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
      updateBtn();
    }

    // Worker pulls from a shared index until the queue drains.
    async function worker() {
      while (queueIndex < total) {
        var i = queueIndex++;
        await uploadOne(files[i]);
      }
    }

    var workers = [];
    var n = Math.min(UPLOAD_CONCURRENCY, total);
    for (var i = 0; i < n; i++) workers.push(worker());
    await Promise.all(workers);

    if (uploadBtn) uploadBtn.innerHTML = '↑ Upload';

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
  // Single document-level handler dispatches all data-action clicks.
  // This replaces the inline 'onclick' pattern (which would be a XSS
  // vector via filenames in HTML/JS double-context) and CSP-incompatible.
  document.addEventListener('click', function(e) {
    // Close style menu when clicking outside
    if (!e.target.closest('#styleMenu') && !e.target.closest('#styleToggle')) {
      closeStyleMenu();
    }
    // Close more menus when clicking outside
    if (!e.target.closest('.more-wrap')) closeAllMore();

    // Client-side navigation for any anchor with data-nav
    var navAnchor = e.target.closest('a[data-nav]');
    if (navAnchor && !e.target.closest('[data-action]')) {
      e.preventDefault();
      navigateTo(navAnchor.getAttribute('href'));
      return;
    }

    // Grid tile preview override: clicking a media file link opens preview
    // instead of navigating away.
    var tileLink = e.target.closest('a[data-tile-preview="1"]');
    if (tileLink) {
      e.preventDefault();
      previewMedia(
        tileLink.getAttribute('href'),
        tileLink.getAttribute('data-name') || '',
        tileLink.getAttribute('data-media') || ''
      );
      return;
    }

    // Action dispatch
    var actEl = e.target.closest('[data-action]');
    if (!actEl) return;
    var action = actEl.getAttribute('data-action');
    var href = actEl.getAttribute('data-href') || '';
    var name = actEl.getAttribute('data-name') || '';
    var isDir = actEl.getAttribute('data-is-dir') === '1';
    var media = actEl.getAttribute('data-media') || '';

    // Always stop propagation so action clicks don't trigger navigation, etc.
    e.stopPropagation();

    switch (action) {
      case 'preview':
        previewMedia(href, name, media);
        break;
      case 'copy':
        copyLink(href, actEl);
        break;
      case 'qr':
        showQr(href);
        break;
      case 'download':
        // Use a temporary anchor so the browser respects Content-Disposition.
        var a = document.createElement('a');
        a.href = href;
        a.rel = 'noopener';
        a.click();
        break;
      case 'rename':
        renameEntry(href, name);
        closeAllMore();
        break;
      case 'move':
        moveEntry(href, name, isDir);
        closeAllMore();
        break;
      case 'delete':
        deleteEntry(href, name, isDir);
        closeAllMore();
        break;
      case 'more-toggle':
        toggleMore(actEl);
        break;
      case 'move-browse-to':
        var path = actEl.getAttribute('data-path') || '/';
        moveBrowseTo(path);
        break;
    }
  });

  // Initial load
  loadDirectory(location.pathname);
})();
