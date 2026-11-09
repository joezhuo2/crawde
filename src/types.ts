export interface Subagent {
  id: string;
  name: string;
  task: string;
  state: 'starting' | 'working' | 'thinking' | 'approval';
  tool: string;
}

export interface Session {
  id: string;
  cwd: string;
  project: string;
  title: string;
  status: 'working' | 'approval' | 'done' | 'error';
  activity: string;
  tool: string;
  agents: Subagent[];
  updatedAt: number;
  lastPromptAt: number;
  finishedAt: number | null;
  workingMs: number;
  waitingMs: number;
  statusSince: number;
  contextPct?: number | null;
}

export interface Snapshot {
  sessions: Session[];
  hooksInstalled: boolean;
  bridgeError: string | null;
  claudeOpen?: boolean;
  dockRight?: boolean;
  dockTop?: boolean;
  petX?: number | null;
  petY?: number | null;
  preferences?: import('./preferences').Preferences;
  lastPromptAt?: number;
  lastActivityAt?: number;
  lastEventAt?: number;
  update?: { checkedAt: number; available: boolean; version: string | null; message: string };
}
