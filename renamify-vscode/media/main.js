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
  const searchBtn = document.getElementById('searchBtn');
  const planBtn = document.getElementById('planBtn');
  const applyBtn = document.getElementById('applyBtn');
  const clearBtn = document.getElementById('clearBtn');
  const expandAllBtn = document.getElementById('expandAll');
  const collapseAllBtn = document.getElementById('collapseAll');
  const resultsSummary = document.getElementById('resultsSummary');
  const resultsTree = document.getElementById('resultsTree');

  // Event listeners
  searchBtn.addEventListener('click', performSearch);
  planBtn.addEventListener('click', createPlan);
  applyBtn.addEventListener('click', applyChanges);
  clearBtn.addEventListener('click', clearResults);
  expandAllBtn.addEventListener('click', expandAll);
  collapseAllBtn.addEventListener('click', collapseAll);

  searchInput.addEventListener('keypress', (e) => {
    if (e.key === 'Enter') {
      performSearch();
    }
  });

  replaceInput.addEventListener('keypress', (e) => {
    if (e.key === 'Enter') {
      performSearch();
    }
  });

  function getSelectedCaseStyles() {
    const checkboxes = document.querySelectorAll(
      '.case-styles-container input[type="checkbox"]:checked'
    );
    return Array.from(checkboxes).map((cb) => cb.value);
  }

  function performSearch() {
    const searchTerm = searchInput.value;
    const replaceTerm = replaceInput.value;

    if (!searchTerm) {
      return;
    }

    showLoading();

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

  function createPlan() {
    const searchTerm = searchInput.value;
    const replaceTerm = replaceInput.value;

    if (!searchTerm) {
      return;
    }

    vscode.postMessage({
      type: 'plan',
      search: searchTerm,
      replace: replaceTerm,
      include: includeInput.value,
      exclude: excludeInput.value,
      excludeMatchingLines: excludeLinesInput.value,
      caseStyles: getSelectedCaseStyles(),
    });
  }

  function applyChanges() {
    vscode.postMessage({
      type: 'apply',
    });
  }

  function clearResults() {
    currentResults = [];
    expandedFiles.clear();
    resultsTree.innerHTML = '';
    resultsSummary.textContent = '';
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
      return;
    }

    const totalMatches = results.reduce(
      (sum, file) => sum + file.matches.length,
      0
    );
    resultsSummary.textContent = `${totalMatches} results in ${results.length} files`;

    resultsTree.innerHTML = '';

    results.forEach((fileResult, index) => {
      const fileItem = createFileItem(fileResult, index);
      resultsTree.appendChild(fileItem);
    });
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

    const fileName = document.createElement('span');
    fileName.className = 'file-name';
    fileName.textContent = fileResult.file;

    const matchCount = document.createElement('span');
    matchCount.className = 'match-count';
    matchCount.textContent = fileResult.matches.length;

    fileHeader.appendChild(expandIcon);
    fileHeader.appendChild(fileName);
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

    // Escape HTML
    let formatted = escapeHtml(context);

    // Highlight search term with red background and strikethrough
    if (searchTerm) {
      const searchRegex = new RegExp(escapeRegExp(searchTerm), 'gi');
      formatted = formatted.replace(searchRegex, (m) => {
        return `<span class="search-match">${m}</span>`;
      });
    }

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
        formatted += `<span class="replace-match">${escapeHtml(replaceTerm)}</span>`;
      }
    }

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
  }

  function collapseAll() {
    expandedFiles.clear();
    for (const fileItem of resultsTree.querySelectorAll('.file-item')) {
      const expandIcon = fileItem.querySelector('.expand-icon');
      const matchesContainer = fileItem.querySelector('.file-matches');
      expandIcon.textContent = '▶';
      matchesContainer.classList.remove('expanded');
    }
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
    }
  }

  // Save state on input changes
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

  searchInput.addEventListener('input', saveState);
  replaceInput.addEventListener('input', saveState);
  includeInput.addEventListener('input', saveState);
  excludeInput.addEventListener('input', saveState);
  excludeLinesInput.addEventListener('input', saveState);
})();
