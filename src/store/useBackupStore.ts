import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';

// Types matching Rust backend
export interface BackupConfig {
  id: string;
  name: string;
  source_path: string;
  destination_path: string;
  schedule: ScheduleConfig | null;
  enabled: boolean;
  mode: 'copy' | 'compressed' | 'encrypted'; // Backup mode: copy (no compression), compressed (default), or encrypted
  encryption_password?: string; // Only for encrypted mode, never persisted
  backup_type: 'full' | 'incremental';
  created_at: number;
  updated_at: number;
  last_backup_at: number | null;
  last_backup_original_size: number | null;
  last_backup_compressed_size: number | null;
  last_backup_files_count: number | null;
  last_backup_checksum: string | null;
}

export interface ScheduleConfig {
  cron_expression: string;
  preset: 'hourly' | 'daily' | 'weekly' | 'monthly' | 'custom' | null;
  next_run: number | null;
  enabled: boolean;
}

interface BackupStore {
  // State
  configs: BackupConfig[];
  isLoading: boolean;
  error: string | null;

  // Actions
  loadConfigs: () => Promise<void>;
  saveConfig: (config: BackupConfig) => Promise<void>;
  deleteConfig: (configId: string) => Promise<void>;
  selectFolder: () => Promise<string | null>;
}

export const useBackupStore = create<BackupStore>((set, get) => ({
  // Initial state
  configs: [],
  isLoading: false,
  error: null,

  // Load configs from backend
  loadConfigs: async () => {
    set({ isLoading: true, error: null });
    try {
      const configs = await invoke<BackupConfig[]>('load_configs');
      set({ configs, isLoading: false });
    } catch (error) {
      set({ error: String(error), isLoading: false });
    }
  },

  // Save a config
  saveConfig: async (config: BackupConfig) => {
    set({ isLoading: true, error: null });
    try {
      await invoke('save_config', { config });
      // Reload configs to get updated state
      await get().loadConfigs();
    } catch (error) {
      set({ error: String(error), isLoading: false });
    }
  },

  // Delete a config
  deleteConfig: async (configId: string) => {
    set({ isLoading: true, error: null });
    try {
      await invoke('delete_config', { configId });
      await get().loadConfigs();
    } catch (error) {
      set({ error: String(error), isLoading: false });
    }
  },

  // Open folder selector
  selectFolder: async () => {
    try {
      const folder = await invoke<string | null>('select_folder');
      return folder;
    } catch (error) {
      set({ error: String(error) });
      return null;
    }
  },
}));
