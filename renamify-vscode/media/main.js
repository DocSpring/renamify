(() => {
  const vscode = acquireVsCodeApi();

  let currentResults = [];
  const expandedFiles = new Set();

  // DOM elements
  const searchInput = document.getElementById('search');
  const replaceInput = document.getElementById('replace');
  const includeInput = document.getElementById('include');
  const excludeInput = document.getElementById('exclude');
  const excludeLinesInput = document.getElementById('excludeLines');
  const applyBtn = document.getElementById('applyBtn');
  const expandAllBtn = document.getElementById('expandAll');
  const collapseAllBtn = document.getElementById('collapseAll');
  const resultsSummary = document.getElementById('resultsSummary');
  const resultsTree = document.getElementById('resultsTree');
  const openInEditorLink = document.getElementById('openInEditor');
  const caseStylesHeader = document.getElementById('caseStylesHeader');
  const caseStylesContainer = document.getElementById('caseStylesContainer');
  const checkedCount = document.getElementById('checkedCount');

  // Debounce timer
  let searchDebounceTimer = null;

  // Event listeners
  applyBtn.addEventListener('click', applyChanges);
  expandAllBtn.addEventListener('click', expandAll);
  collapseAllBtn.addEventListener('click', collapseAll);

  openInEditorLink.addEventListener('click', (e) => {
    e.preventDefault();
    openPreviewInEditor();
  });

  // Case styles collapsible section
  caseStylesHeader.addEventListener('click', () => {
    const isCollapsed = caseStylesContainer.classList.contains('collapsed');
    const expandIcon = caseStylesHeader.querySelector('.expand-icon');

    if (isCollapsed) {
      caseStylesContainer.classList.remove('collapsed');
      expandIcon.textContent = '▼';
    } else {
      caseStylesContainer.classList.add('collapsed');
      expandIcon.textContent = '▶';
    }
  });

  // Debounced auto-search on input
  function debouncedSearch() {
    clearTimeout(searchDebounceTimer);
    searchDebounceTimer = setTimeout(() => {
      performSearch();
    }, 300); // 300ms debounce
  }

  searchInput.addEventListener('input', debouncedSearch);
  replaceInput.addEventListener('input', debouncedSearch);
  includeInput.addEventListener('input', debouncedSearch);
  excludeInput.addEventListener('input', debouncedSearch);
  excludeLinesInput.addEventListener('input', debouncedSearch);

  // Update checked count and trigger search when checkboxes change
  function updateCheckedCount() {
    const checked = document.querySelectorAll(
      '.case-styles-container input[type="checkbox"]:checked'
    ).length;
    checkedCount.textContent = checked;
  }

  for (const checkbox of document.querySelectorAll(
    '.case-styles-container input[type="checkbox"]'
  )) {
    checkbox.addEventListener('change', () => {
      updateCheckedCount();
      debouncedSearch();
    });
  }

  // Initial count update
  updateCheckedCount();

  function getSelectedCaseStyles() {
    const checkboxes = document.querySelectorAll(
      '.case-styles-container input[type="checkbox"]:checked'
    );
    return Array.from(checkboxes).map((cb) => cb.value);
  }

  function performSearch() {
    const searchTerm = searchInput.value.trim();
    const replaceTerm = replaceInput.value.trim();

    // Clear results if search is empty
    if (!searchTerm) {
      clearResults();
      return;
    }

    showLoading();

    // Always use search mode (backend will decide to use plan with --dry-run if replace is provided)
    vscode.postMessage({
      type: 'search',
      search: searchTerm,
      replace: replaceTerm,
      include: includeInput.value,
      exclude: excludeInput.value,
      excludeMatchingLines: excludeLinesInput.value,
      caseStyles: getSelectedCaseStyles(),
    });
  }

  function applyChanges() {
    const searchTerm = searchInput.value.trim();
    const replaceTerm = replaceInput.value.trim();

    if (!(searchTerm && replaceTerm)) {
      // Can't apply without both search and replace
      return;
    }

    vscode.postMessage({
      type: 'apply',
      search: searchTerm,
      replace: replaceTerm,
      include: includeInput.value,
      exclude: excludeInput.value,
      excludeMatchingLines: excludeLinesInput.value,
      caseStyles: getSelectedCaseStyles(),
    });
  }

  function clearResults() {
    currentResults = [];
    expandedFiles.clear();
    resultsTree.innerHTML = '';
    resultsSummary.textContent = '';
    openInEditorLink.style.display = 'none';
  }

  function openPreviewInEditor() {
    const searchTerm = searchInput.value.trim();
    const replaceTerm = replaceInput.value.trim();

    if (!searchTerm) {
      return;
    }

    vscode.postMessage({
      type: 'openPreview',
      search: searchTerm,
      replace: replaceTerm,
      include: includeInput.value,
      exclude: excludeInput.value,
      excludeMatchingLines: excludeLinesInput.value,
      caseStyles: getSelectedCaseStyles(),
    });
  }

  function showLoading() {
    resultsTree.innerHTML =
      '<div class="loading"><div class="spinner"></div><p>Searching...</p></div>';
  }

  function renderResults(results) {
    currentResults = results;

    if (!results || results.length === 0) {
      resultsTree.innerHTML = '<div class="empty-state">No results found</div>';
      resultsSummary.textContent = '0 results in 0 files';
      openInEditorLink.style.display = 'none';
      updateExpandCollapseButtons();
      return;
    }

    const totalMatches = results.reduce(
      (sum, file) => sum + file.matches.length,
      0
    );
    resultsSummary.textContent = `${totalMatches} results in ${results.length} files`;
    openInEditorLink.style.display = 'inline-block';

    resultsTree.innerHTML = '';

    // Expand all files by default
    expandedFiles.clear();
    results.forEach((_, index) => {
      expandedFiles.add(index);
    });

    results.forEach((fileResult, index) => {
      const fileItem = createFileItem(fileResult, index);
      resultsTree.appendChild(fileItem);
    });

    updateExpandCollapseButtons();
  }

  function createFileItem(fileResult, index) {
    const fileItem = document.createElement('div');
    fileItem.className = 'file-item';
    fileItem.dataset.index = index;

    const fileHeader = document.createElement('div');
    fileHeader.className = 'file-header';

    const expandIcon = document.createElement('span');
    expandIcon.className = 'expand-icon';
    expandIcon.textContent = expandedFiles.has(index) ? '▼' : '▶';

    // Split filename into basename and directory
    let fullPath = fileResult.file;

    // Normalize path separators to forward slashes
    fullPath = fullPath.replace(/\\/g, '/');

    // Strip workspace root if present
    if (window.workspaceRoot) {
      let workspaceRoot = window.workspaceRoot.replace(/\\/g, '/');
      if (!workspaceRoot.endsWith('/')) {
        workspaceRoot += '/';
      }
      if (fullPath.startsWith(workspaceRoot)) {
        fullPath = fullPath.substring(workspaceRoot.length);
      }
    }

    // Remove leading ./ if present
    if (fullPath.startsWith('./')) {
      fullPath = fullPath.substring(2);
    }

    // Split into basename and directory
    const lastSlash = fullPath.lastIndexOf('/');
    const basename =
      lastSlash >= 0 ? fullPath.substring(lastSlash + 1) : fullPath;
    const dirname = lastSlash >= 0 ? fullPath.substring(0, lastSlash) : '';

    const fileNameContainer = document.createElement('span');
    fileNameContainer.className = 'file-name-container';

    const fileBasename = document.createElement('span');
    fileBasename.className = 'file-basename';
    fileBasename.textContent = basename;

    if (dirname) {
      const fileDirname = document.createElement('span');
      fileDirname.className = 'file-dirname';
      fileDirname.textContent = dirname;
      fileNameContainer.appendChild(fileBasename);
      fileNameContainer.appendChild(fileDirname);
    } else {
      fileNameContainer.appendChild(fileBasename);
    }

    const matchCount = document.createElement('span');
    matchCount.className = 'match-count';
    matchCount.textContent = fileResult.matches.length;

    fileHeader.appendChild(expandIcon);
    fileHeader.appendChild(fileNameContainer);
    fileHeader.appendChild(matchCount);

    fileHeader.addEventListener('click', () => toggleFile(index));

    const matchesContainer = document.createElement('div');
    matchesContainer.className = 'file-matches';

    if (expandedFiles.has(index)) {
      matchesContainer.classList.add('expanded');
      renderMatches(matchesContainer, fileResult);
    }

    fileItem.appendChild(fileHeader);
    fileItem.appendChild(matchesContainer);

    return fileItem;
  }

  function renderMatches(container, fileResult) {
    container.innerHTML = '';

    for (const match of fileResult.matches) {
      const matchItem = document.createElement('div');
      matchItem.className = 'match-item';

      const lineNumber = document.createElement('span');
      lineNumber.className = 'line-number';
      lineNumber.textContent = `${match.line}:`;

      const matchText = document.createElement('span');
      matchText.className = 'match-text';
      matchText.innerHTML = formatMatchText(match);

      matchItem.appendChild(lineNumber);
      matchItem.appendChild(matchText);

      matchItem.addEventListener('click', () => {
        vscode.postMessage({
          type: 'openFile',
          file: fileResult.file,
          line: match.line,
        });
      });

      container.appendChild(matchItem);
    }
  }

  function formatMatchText(match) {
    const context = match.context || match.text;
    const searchTerm = searchInput.value;
    const replaceTerm = replaceInput.value;

    // Create a container div for horizontal scrolling
    let formatted = '<div class="match-content">';

    // Escape HTML
    let escapedContext = escapeHtml(context);

    // Highlight search term with red background and strikethrough
    if (searchTerm) {
      const searchRegex = new RegExp(escapeRegExp(searchTerm), 'gi');
      escapedContext = escapedContext.replace(searchRegex, (m) => {
        return `<span class="search-match">${m}</span>`;
      });
    }

    formatted += escapedContext;

    // Show replacement with green background
    if (replaceTerm && searchTerm) {
      const searchRegex = new RegExp(escapeRegExp(searchTerm), 'gi');
      const matches = context.match(searchRegex);
      if (matches) {
        formatted += ' → ';
        let replaced = context;
        for (const m of matches) {
          replaced = replaced.replace(m, replaceTerm);
        }
        formatted += `<span class="replace-match">${escapeHtml(
          replaceTerm
        )}</span>`;
      }
    }

    formatted += '</div>';
    return formatted;
  }

  function toggleFile(index) {
    const fileItem = resultsTree.querySelector(`[data-index="${index}"]`);
    const expandIcon = fileItem.querySelector('.expand-icon');
    const matchesContainer = fileItem.querySelector('.file-matches');

    if (expandedFiles.has(index)) {
      expandedFiles.delete(index);
      expandIcon.textContent = '▶';
      matchesContainer.classList.remove('expanded');
    } else {
      expandedFiles.add(index);
      expandIcon.textContent = '▼';
      matchesContainer.classList.add('expanded');

      if (matchesContainer.children.length === 0) {
        renderMatches(matchesContainer, currentResults[index]);
      }
    }

    updateExpandCollapseButtons();
  }

  function updateExpandCollapseButtons() {
    const hasExpanded = expandedFiles.size > 0;

    if (hasExpanded) {
      // Show collapse all button
      expandAllBtn.style.display = 'none';
      collapseAllBtn.style.display = 'flex';
    } else {
      // Show expand all button
      expandAllBtn.style.display = 'flex';
      collapseAllBtn.style.display = 'none';
    }
  }

  function expandAll() {
    currentResults.forEach((_, index) => {
      if (!expandedFiles.has(index)) {
        expandedFiles.add(index);
        const fileItem = resultsTree.querySelector(`[data-index="${index}"]`);
        if (fileItem) {
          const expandIcon = fileItem.querySelector('.expand-icon');
          const matchesContainer = fileItem.querySelector('.file-matches');
          expandIcon.textContent = '▼';
          matchesContainer.classList.add('expanded');

          if (matchesContainer.children.length === 0) {
            renderMatches(matchesContainer, currentResults[index]);
          }
        }
      }
    });
    updateExpandCollapseButtons();
  }

  function collapseAll() {
    expandedFiles.clear();
    for (const fileItem of resultsTree.querySelectorAll('.file-item')) {
      const expandIcon = fileItem.querySelector('.expand-icon');
      const matchesContainer = fileItem.querySelector('.file-matches');
      expandIcon.textContent = '▶';
      matchesContainer.classList.remove('expanded');
    }
    updateExpandCollapseButtons();
  }

  function escapeHtml(text) {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
  }

  function escapeRegExp(string) {
    return string.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
  }

  // Handle messages from extension
  window.addEventListener('message', (event) => {
    const message = event.data;

    switch (message.type) {
      case 'searchResults':
        renderResults(message.results);
        break;
      case 'clearResults':
        clearResults();
        break;
      case 'planCreated':
        // Update UI to show plan was created
        break;
      case 'changesApplied':
        // Clear results after successful apply
        clearResults();
        break;
      default:
        console.warn(`Unknown message type: ${message.type}`);
        break;
    }
  });

  // Restore state if any
  const state = vscode.getState();
  if (state) {
    searchInput.value = state.search || '';
    replaceInput.value = state.replace || '';
    includeInput.value = state.include || '';
    excludeInput.value = state.exclude || '';
    excludeLinesInput.value = state.excludeLines || '';

    if (state.results) {
      renderResults(state.results);
    } else if (state.search) {
      // Trigger initial search if we have a search term
      performSearch();
    }
  }

  // Save state on input changes (but don't trigger search)
  function saveState() {
    vscode.setState({
      search: searchInput.value,
      replace: replaceInput.value,
      include: includeInput.value,
      exclude: excludeInput.value,
      excludeLines: excludeLinesInput.value,
      results: currentResults,
    });
  }

  // Update apply button state
  function updateApplyButton() {
    const hasReplace = replaceInput.value.trim() !== '';
    const hasSearch = searchInput.value.trim() !== '';
    applyBtn.disabled = !(hasSearch && hasReplace);
    applyBtn.textContent = hasReplace
      ? `Apply: ${searchInput.value.trim()} → ${replaceInput.value.trim()}`
      : 'Apply Rename';
  }

  searchInput.addEventListener('input', () => {
    saveState();
    updateApplyButton();
  });
  replaceInput.addEventListener('input', () => {
    saveState();
    updateApplyButton();
  });
  includeInput.addEventListener('input', saveState);
  excludeInput.addEventListener('input', saveState);
  excludeLinesInput.addEventListener('input', saveState);

  // Initial button state
  updateApplyButton();
})();
