import { invoke, isTauri } from '@tauri-apps/api/core';

export const native = isTauri();
export async function command<T = void>(name: string, args?: Record<string, unknown>): Promise<T> {
  if (!native) throw new Error('This action is available in the Windows app.');
  return invoke<T>(name, args);
}
