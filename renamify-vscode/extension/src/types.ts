// Type definitions for the Renamify VS Code extension

export type SearchOptions = {
  include?: string;
  exclude?: string;
  excludeMatchingLines?: string;
  caseStyles?: string[];
  renamePaths?: boolean;
  ignoreAmbiguous?: boolean;
  atomicSearch?: boolean;
  atomicReplace?: boolean;
  enablePluralVariants?: boolean;
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
  renamePaths?: boolean;
  ignoreAmbiguous?: boolean;
  atomicSearch?: boolean;
  atomicReplace?: boolean;
  enablePluralVariants?: boolean;
  searchId?: number;
};

export type PlanMessage = {
  type: 'plan';
  search: string;
  replace: string;
  include?: string;
  exclude?: string;
  excludeMatchingLines?: string;
  caseStyles?: string[];
  renamePaths?: boolean;
  ignoreAmbiguous?: boolean;
  atomicSearch?: boolean;
  atomicReplace?: boolean;
  enablePluralVariants?: boolean;
};

export type ApplyMessage = {
  type: 'apply';
  search?: string;
  replace?: string;
  include?: string;
  exclude?: string;
  excludeMatchingLines?: string;
  caseStyles?: string[];
  renamePaths?: boolean;
  ignoreAmbiguous?: boolean;
  atomicSearch?: boolean;
  atomicReplace?: boolean;
  enablePluralVariants?: boolean;
  planId?: string;
};

export type OpenFileMessage = {
  type: 'openFile';
  file: string;
  line?: number;
  column?: number;
};

export type OpenPreviewMessage = {
  type: 'openPreview';
  search: string;
  replace?: string;
  include?: string;
  exclude?: string;
  excludeMatchingLines?: string;
  caseStyles?: string[];
  enablePluralVariants?: boolean;
};

export type OpenSettingsMessage = {
  type: 'openSettings';
};

export type RefreshMessage = {
  type: 'refresh';
};

export type WebviewMessage =
  | SearchMessage
  | PlanMessage
  | ApplyMessage
  | OpenFileMessage
  | OpenPreviewMessage
  | OpenSettingsMessage
  | RefreshMessage;

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
