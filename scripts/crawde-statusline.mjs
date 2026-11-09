import { spawnSync } from 'node:child_process';
import { mkdirSync, renameSync, rmSync, writeFileSync } from 'node:fs';
import { homedir } from 'node:os';
import { join } from 'node:path';

const SESSION_ID = /^[A-Za-z0-9-]{1,128}$/;

function writeAtomic(file, text) {
  const staging = `${file}.${process.pid}.tmp`;
  try {
    writeFileSync(staging, text);
    renameSync(staging, file);
  } catch {
    try { rmSync(staging, { force: true }); } catch {}
  }
}

function record(raw) {
  try {
    const data = JSON.parse(raw);
    if (!data || typeof data !== 'object' || Array.isArray(data)) return;
    const directory = join(homedir(), '.crawde', 'statusline');
    mkdirSync(directory, { recursive: true });
    if (typeof data.session_id === 'string' && SESSION_ID.test(data.session_id)) {
      writeAtomic(join(directory, `${data.session_id}.json`), raw);
    }
    writeAtomic(join(directory, 'latest.json'), raw);
  } catch {}
}

function chain(raw) {
  const next = process.env.CRAWDE_STATUSLINE_NEXT;
  if (!next) return;
  try {
    const result = spawnSync(next, { shell: true, input: raw, encoding: 'utf8', stdio: ['pipe', 'pipe', 'ignore'], timeout: 5000 });
    if (result.stdout) process.stdout.write(result.stdout);
  } catch {}
}

let finished = false;
const chunks = [];

function finish() {
  if (finished) return;
  finished = true;
  const raw = Buffer.concat(chunks).toString('utf8').replace(/^\uFEFF/, '');
  record(raw);
  chain(raw);
}

process.on('uncaughtException', () => process.exit(0));
process.stdin.on('data', chunk => chunks.push(chunk));
process.stdin.on('end', finish);
process.stdin.on('error', finish);
process.stdin.on('close', finish);
