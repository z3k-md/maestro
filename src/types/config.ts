export interface Config {
  sessions: SessionConfig[];
  mixer: MixerConfig;
  system: SystemConfig;
}

export interface KeybindConfig {
  key: string;
  action: string;
}

export interface SessionConfig {
  name: string;
  keybinds: KeybindConfig[] | null;
}

export interface MixerConfig {
  enabled: boolean;
  hotkey: string | null;
}

export interface SystemConfig {
  autostart: boolean;
  show_console: boolean;
  theme: string | null;
}
