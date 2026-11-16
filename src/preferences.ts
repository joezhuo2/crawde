export interface Preferences {
  enabled: boolean;
  startup: boolean;
  hotkey: string;
  theme: 'system' | 'light' | 'dark';
  scale: number;
  roaming: boolean;
  reducedMotion: boolean;
  sound: boolean;
  volume: number;
  soundStyle: 'soft' | 'retro';
  approvalSound: boolean;
  completionSound: boolean;
  errorSound: boolean;
  reminders: boolean;
  openTarget: 'desktop' | 'terminal';
  quietHours: boolean;
  quietStart: string;
  quietEnd: string;
  followFocusAssist: boolean;
  ghost: boolean;
  ghostHotkey: string;
  hideInFullscreen: boolean;
}

export const defaults: Preferences = {
  enabled: true, startup: true, hotkey: 'Ctrl+Alt+P', theme: 'system', scale: 3,
  roaming: true, reducedMotion: false, sound: true, volume: .45, soundStyle: 'soft',
  approvalSound: true, completionSound: true, errorSound: true, reminders: true,
  openTarget: 'desktop', quietHours: false, quietStart: '22:00', quietEnd: '08:00', followFocusAssist: true,
  ghost: false, ghostHotkey: 'Ctrl+Alt+G', hideInFullscreen: true,
};
