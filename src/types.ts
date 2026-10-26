export interface Session {
  id: string;
  cwd: string;
  project: string;
  title: string;
  status: 'working' | 'approval' | 'done' | 'error';
  activity: string;
  tool: string;
  agents: string[];
  updatedAt: number;
  lastPromptAt: number;
  finishedAt: number | null;
}

export interface Snapshot {
  sessions: Session[];
  hooksInstalled: boolean;
  bridgeError: string | null;
  claudeOpen?: boolean;
  dockRight?: boolean;
  dockTop?: boolean;
  preferences?: import('./preferences').Preferences;
  lastPromptAt?: number;
  lastActivityAt?: number;
  lastEventAt?: number;
  update?: { checkedAt: number; available: boolean; version: string | null; message: string };
}
