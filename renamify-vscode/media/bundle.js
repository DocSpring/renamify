// Minimal AMD loader for VS Code webview
(() => {
  const modules = {};
  const waiting = {};

  window.define = (nameParam, depsParam, factoryParam) => {
    let name = nameParam;
    let deps = depsParam;
    let factory = factoryParam;

    if (typeof name !== 'string') {
      // Anonymous module
      factory = deps;
      deps = name;
      name = null;
    }

    if (!Array.isArray(deps)) {
      factory = deps;
      deps = ['require', 'exports'];
    }

    const module = { exports: {} };
    const depModules = deps.map((dep) => {
      if (dep === 'require') {
        return window.require;
      }
      if (dep === 'exports') {
        return module.exports;
      }
      return modules[dep];
    });

    factory(...depModules);

    if (name) {
      modules[name] = module.exports;

      // Check if anything was waiting for this module
      if (waiting[name]) {
        for (const callback of waiting[name]) {
          callback(module.exports);
        }
        delete waiting[name];
      }
    }
  };

  window.require = (deps, callback) => {
    if (typeof deps === 'string') {
      return modules[deps];
    }

    if (callback) {
      const depModules = deps.map((dep) => modules[dep]);
      callback(...depModules);
    }
  };

  // For compatibility
  window.define.amd = {};
})();
define("formatter", ["require", "exports"], function (require, exports) {
    "use strict";
    Object.defineProperty(exports, "__esModule", { value: true });
    exports.escapeHtml = escapeHtml;
    exports.escapeRegExp = escapeRegExp;
    exports.highlightSearchTerm = highlightSearchTerm;
    exports.highlightReplaceTerm = highlightReplaceTerm;
    exports.formatMergedMatchText = formatMergedMatchText;
    function escapeHtml(text) {
        if (!text) {
            return '';
        }
        return text
            .replace(/&/g, '&amp;')
            .replace(/</g, '&lt;')
            .replace(/>/g, '&gt;');
    }
    function escapeRegExp(string) {
        return string.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
    }
    function highlightSearchTerm(text, searchTerm) {
        if (!searchTerm) {
            return text;
        }
        const searchRegex = new RegExp(escapeRegExp(searchTerm), 'gi');
        return text.replace(searchRegex, (m) => `<span class="search-highlight">${m}</span>`);
    }
    function highlightReplaceTerm(text, matches) {
        if (!matches) {
            return text;
        }
        let highlightedText = text;
        // Collect all unique replacement terms from the matches
        const replacementTerms = new Set();
        for (const match of matches) {
            if (match.replace) {
                replacementTerms.add(match.replace);
            }
        }
        // Highlight each replacement term
        for (const replaceTerm of replacementTerms) {
            const replaceRegex = new RegExp(escapeRegExp(replaceTerm), 'gi');
            highlightedText = highlightedText.replace(replaceRegex, (m) => `<span class="replace-highlight">${m}</span>`);
        }
        return highlightedText;
    }
    function formatMergedMatchText(matches, searchTerm, replaceTerm) {
        // Use the first match for the original line text (should be the full line)
        const originalText = matches[0].line_before || matches[0].content || '';
        let formatted = '<div class="match-content">';
        if (replaceTerm && matches.some((m) => m.replace)) {
            // Merge all replacements by applying them in order
            let finalText = originalText;
            // Sort matches by column position to apply replacements in the correct order
            const sortedMatches = [...matches].sort((a, b) => a.col - b.col);
            // Apply replacements from right to left to avoid position shifts
            for (let i = sortedMatches.length - 1; i >= 0; i--) {
                const match = sortedMatches[i];
                if (match.replace && match.content) {
                    // Use line-relative positions from col field
                    const lineStart = match.col; // col is already 0-indexed for our purposes
                    const lineEnd = lineStart + match.content.length;
                    const before = finalText.substring(0, lineStart);
                    const after = finalText.substring(lineEnd);
                    finalText = before + match.replace + after;
                }
            }
            // Show diff format: - original, + final result
            formatted +=
                '<div class="diff-line removed">- ' +
                    highlightSearchTerm(escapeHtml(originalText), searchTerm) +
                    '</div>';
            formatted +=
                '<div class="diff-line added">+ ' +
                    highlightReplaceTerm(escapeHtml(finalText), matches) +
                    '</div>';
        }
        else {
            // Search only mode - just highlight all search terms
            formatted +=
                '<div class="search-line">' +
                    highlightSearchTerm(escapeHtml(originalText), searchTerm) +
                    '</div>';
        }
        formatted += '</div>';
        return formatted;
    }
});
// VS Code webview API types
// MatchHunk type is available globally from typeRoots
define("webview", ["require", "exports", "formatter"], function (require, exports, formatter_1) {
    "use strict";
    Object.defineProperty(exports, "__esModule", { value: true });
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
                if (expandIcon) {
                    expandIcon.innerHTML =
                        '<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="m18 15-6-6-6 6"/></svg>';
                }
            }
            else {
                caseStylesContainer.classList.add('collapsed');
                if (expandIcon) {
                    expandIcon.innerHTML =
                        '<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" style="transform: rotate(-90deg);"><path d="m18 15-6-6-6 6"/></svg>';
                }
            }
        });
        // Debounced auto-search on input
        function debouncedSearch() {
            if (searchDebounceTimer) {
                clearTimeout(searchDebounceTimer);
            }
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
            const checked = document.querySelectorAll('.case-styles-container input[type="checkbox"]:checked').length;
            checkedCount.textContent = checked.toString();
        }
        const checkboxes = document.querySelectorAll('.case-styles-container input[type="checkbox"]');
        for (const checkbox of Array.from(checkboxes)) {
            checkbox.addEventListener('change', () => {
                updateCheckedCount();
                debouncedSearch();
            });
        }
        // Initial count update
        updateCheckedCount();
        function getSelectedCaseStyles() {
            const checkedBoxes = document.querySelectorAll('.case-styles-container input[type="checkbox"]:checked');
            return Array.from(checkedBoxes).map((cb) => cb.value);
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
        function normalizePathForDisplay(path) {
            let normalizedPath = path;
            // Normalize path separators to forward slashes
            normalizedPath = normalizedPath.replace(/\\/g, '/');
            // Strip workspace root if present
            const windowWithRoot = window;
            if (windowWithRoot.workspaceRoot) {
                let workspaceRoot = windowWithRoot.workspaceRoot.replace(/\\/g, '/');
                if (!workspaceRoot.endsWith('/')) {
                    workspaceRoot += '/';
                }
                if (normalizedPath.startsWith(workspaceRoot)) {
                    normalizedPath = normalizedPath.substring(workspaceRoot.length);
                }
            }
            // Remove leading ./ if present
            if (normalizedPath.startsWith('./')) {
                normalizedPath = normalizedPath.substring(2);
            }
            return normalizedPath;
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
        function renderResults(results, paths = []) {
            currentResults = results;
            if (!results || results.length === 0) {
                resultsTree.innerHTML = '<div class="empty-state">No results found</div>';
                resultsSummary.textContent = '0 results in 0 files';
                openInEditorLink.style.display = 'none';
                updateExpandCollapseButtons();
                return;
            }
            const totalMatches = results.reduce((sum, file) => sum + file.matches.length, 0);
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
            // Add file/directory renames section if there are any paths
            if (paths.length > 0) {
                const isReplaceMode = replaceInput.value.trim() !== '';
                const pathsSection = createPathsSection(paths, isReplaceMode);
                resultsTree.appendChild(pathsSection);
            }
            updateExpandCollapseButtons();
        }
        function createPathsSection(paths, isReplaceMode = false) {
            const section = document.createElement('div');
            section.className = 'paths-section';
            const header = document.createElement('div');
            header.className = 'paths-header';
            const title = document.createElement('h3');
            title.textContent = isReplaceMode
                ? 'File & Directory Renames'
                : 'Files & Directories';
            title.className = 'paths-title';
            const count = document.createElement('span');
            count.className = 'paths-count';
            // Determine the label based on count and mode
            let label;
            if (paths.length === 1) {
                label = isReplaceMode ? 'rename' : 'match';
            }
            else {
                label = isReplaceMode ? 'renames' : 'matches';
            }
            count.textContent = `${paths.length} ${label}`;
            header.appendChild(title);
            header.appendChild(count);
            section.appendChild(header);
            const pathsList = document.createElement('div');
            pathsList.className = 'paths-list';
            for (const pathRename of paths) {
                const pathItem = document.createElement('div');
                pathItem.className = `path-item path-${pathRename.kind}`;
                const icon = document.createElement('span');
                icon.className = 'path-icon';
                icon.innerHTML = pathRename.kind === 'dir' ? '📁' : '📄';
                const pathInfo = document.createElement('div');
                pathInfo.className = 'path-info';
                if (isReplaceMode && pathRename.new_path) {
                    // Replace mode - show old -> new path
                    const oldPath = document.createElement('div');
                    oldPath.className = 'path-old';
                    oldPath.textContent = normalizePathForDisplay(pathRename.path);
                    const arrow = document.createElement('div');
                    arrow.className = 'path-arrow';
                    arrow.textContent = '↓';
                    const newPath = document.createElement('div');
                    newPath.className = 'path-new';
                    newPath.textContent = normalizePathForDisplay(pathRename.new_path);
                    pathInfo.appendChild(oldPath);
                    pathInfo.appendChild(arrow);
                    pathInfo.appendChild(newPath);
                }
                else {
                    // Search mode - show only the path
                    const pathElement = document.createElement('div');
                    pathElement.className = 'path-only';
                    pathElement.textContent = normalizePathForDisplay(pathRename.path);
                    pathInfo.appendChild(pathElement);
                }
                if (pathRename.coercion_applied) {
                    const coercion = document.createElement('div');
                    coercion.className = 'path-coercion';
                    coercion.textContent = pathRename.coercion_applied;
                    pathInfo.appendChild(coercion);
                }
                pathItem.appendChild(icon);
                pathItem.appendChild(pathInfo);
                pathsList.appendChild(pathItem);
            }
            section.appendChild(pathsList);
            return section;
        }
        function createFileItem(fileResult, index) {
            const fileItem = document.createElement('div');
            fileItem.className = 'file-item';
            fileItem.dataset.index = index.toString();
            const fileHeader = document.createElement('div');
            fileHeader.className = 'file-header';
            const expandIcon = document.createElement('span');
            expandIcon.className = 'expand-icon';
            expandIcon.innerHTML = expandedFiles.has(index)
                ? getChevronDown()
                : getChevronRight();
            // Split filename into basename and directory
            const fullPath = normalizePathForDisplay(fileResult.file);
            // Split into basename and directory
            const lastSlash = fullPath.lastIndexOf('/');
            const basename = lastSlash >= 0 ? fullPath.substring(lastSlash + 1) : fullPath;
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
            }
            else {
                fileNameContainer.appendChild(fileBasename);
            }
            const matchCount = document.createElement('span');
            matchCount.className = 'match-count';
            matchCount.textContent = fileResult.matches.length.toString();
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
            // Group matches by line number
            const matchesByLine = new Map();
            for (const match of fileResult.matches) {
                const lineNum = match.line;
                if (!matchesByLine.has(lineNum)) {
                    matchesByLine.set(lineNum, []);
                }
                matchesByLine.get(lineNum)?.push(match);
            }
            // Render each unique line with all its matches merged
            for (const [lineNum, matches] of matchesByLine) {
                const matchItem = document.createElement('div');
                matchItem.className = 'match-item';
                const lineNumber = document.createElement('span');
                lineNumber.className = 'line-number';
                lineNumber.textContent = lineNum.toString();
                const matchText = document.createElement('span');
                matchText.className = 'match-text';
                matchText.innerHTML = (0, formatter_1.formatMergedMatchText)(matches, searchInput.value, replaceInput.value);
                matchItem.appendChild(lineNumber);
                matchItem.appendChild(matchText);
                matchItem.addEventListener('click', () => {
                    vscode.postMessage({
                        type: 'openFile',
                        file: fileResult.file,
                        line: lineNum,
                    });
                });
                container.appendChild(matchItem);
            }
        }
        function toggleFile(index) {
            const fileItem = resultsTree.querySelector(`[data-index="${index}"]`);
            const expandIcon = fileItem.querySelector('.expand-icon');
            const matchesContainer = fileItem.querySelector('.file-matches');
            if (expandedFiles.has(index)) {
                expandedFiles.delete(index);
                expandIcon.innerHTML = getChevronRight();
                matchesContainer.classList.remove('expanded');
            }
            else {
                expandedFiles.add(index);
                expandIcon.innerHTML = getChevronDown();
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
            }
            else {
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
                        expandIcon.innerHTML = getChevronDown();
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
            const fileItems = resultsTree.querySelectorAll('.file-item');
            for (const fileItem of Array.from(fileItems)) {
                const expandIcon = fileItem.querySelector('.expand-icon');
                const matchesContainer = fileItem.querySelector('.file-matches');
                expandIcon.innerHTML = getChevronRight();
                matchesContainer.classList.remove('expanded');
            }
            updateExpandCollapseButtons();
        }
        function getChevronDown() {
            return '<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" style="transform: rotate(-180deg);"><path d="m18 15-6-6-6 6"/></svg>';
        }
        function getChevronRight() {
            return '<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" style="transform: rotate(-270deg);"><path d="m18 15-6-6-6 6"/></svg>';
        }
        // Handle messages from extension
        window.addEventListener('message', (event) => {
            const message = event.data;
            switch (message.type) {
                case 'searchResults':
                    renderResults(message.results, message.paths);
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
            else if (state.search) {
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
        // Trigger initial search if search input has content
        if (searchInput.value.trim()) {
            performSearch();
        }
    })();
});
//# sourceMappingURL=webview.js.map
