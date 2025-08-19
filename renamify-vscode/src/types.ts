// Type definitions for the Renamify VS Code extension

export type SearchOptions = {
  include?: string;
  exclude?: string;
  excludeMatchingLines?: string;
  caseStyles?: string[];
};

export type Match = {
  line: number;
  column: number;
  text: string;
  replacement: string;
  context: string;
};

export type SearchResult = {
  file: string;
  matches: Match[];
};

export type Plan = {
  id: string;
  created_at: string;
  search: string;
  replace: string;
  styles: string[];
  includes: string[];
  excludes: string[];
  matches: PlanMatch[];
  paths: Rename[];
  stats: PlanStats;
  version: string;
};

export type PlanMatch = {
  file: string;
  line: number;
  column: number;
  text: string;
  replacement: string;
  context: string;
};

export type Rename = {
  old_path: string;
  new_path: string;
  type: 'file' | 'directory';
};

export type PlanStats = {
  total_matches?: number;
  files_affected?: number;
  paths?: number;
};

export type HistoryEntry = {
  id: string;
  search: string;
  replace: string;
  created_at: string;
  stats?: PlanStats;
};

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
