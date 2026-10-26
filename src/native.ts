import { invoke, isTauri } from '@tauri-apps/api/core';

export const native = isTauri();
export async function command<T = void>(name: string, args?: Record<string, unknown>): Promise<T> {
  if (!native) throw new Error('This action is available in the Windows app.');
  let timeout: ReturnType<typeof setTimeout> | undefined;
  try {
    return await Promise.race([
      invoke<T>(name, args),
      new Promise<never>((_, reject) => { timeout = setTimeout(() => reject(new Error('Crawde did not respond. Please restart the desktop app.')), 15_000); }),
    ]);
  } finally { clearTimeout(timeout); }
}
