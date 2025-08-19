// Type definitions for the Renamify VS Code extension

export type SearchOptions = {
  include?: string;
  exclude?: string;
  excludeMatchingLines?: string;
  caseStyles?: string[];
};

// Match type is now imported from Rust bindings above

export type SearchResult = {
  file: string;
  matches: MatchHunk[];
};

// Plan, Rename, PlanStats, and HistoryEntry are now imported from Rust bindings above

export type Status = {
  current_plan?: Plan;
  last_operation?: HistoryEntry;
};

// Webview message types
export type SearchMessage = {
  type: 'search';
  search: string;
  replace: string;
  include?: string;
  exclude?: string;
  excludeMatchingLines?: string;
  caseStyles?: string[];
};

export type PlanMessage = {
  type: 'plan';
  search: string;
  replace: string;
  include?: string;
  exclude?: string;
  excludeMatchingLines?: string;
  caseStyles?: string[];
};

export type ApplyMessage = {
  type: 'apply';
  search?: string;
  replace?: string;
  include?: string;
  exclude?: string;
  excludeMatchingLines?: string;
  caseStyles?: string[];
  planId?: string;
};

export type OpenFileMessage = {
  type: 'openFile';
  file: string;
  line?: number;
};

export type OpenPreviewMessage = {
  type: 'openPreview';
  search: string;
  replace?: string;
  include?: string;
  exclude?: string;
  excludeMatchingLines?: string;
  caseStyles?: string[];
};

export type WebviewMessage =
  | SearchMessage
  | PlanMessage
  | ApplyMessage
  | OpenFileMessage
  | OpenPreviewMessage;

// Response messages
export type SearchResultsMessage = {
  type: 'searchResults';
  results: SearchResult[];
  paths: Rename[];
};

export type PlanCreatedMessage = {
  type: 'planCreated';
  plan: Plan;
};

export type ChangesAppliedMessage = {
  type: 'changesApplied';
};

export type ClearResultsMessage = {
  type: 'clearResults';
};

export type ExtensionMessage =
  | SearchResultsMessage
  | PlanCreatedMessage
  | ChangesAppliedMessage
  | ClearResultsMessage;
