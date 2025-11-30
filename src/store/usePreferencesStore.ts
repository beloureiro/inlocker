import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';

export interface AppPreferences {
  auto_close_progress_window: boolean;
  auto_close_delay_ms: number;
}

interface PreferencesStore {
  // State
  preferences: AppPreferences;
  isLoading: boolean;
  error: string | null;

  // Actions
  loadPreferences: () => Promise<void>;
  savePreferences: (preferences: AppPreferences) => Promise<void>;
  setAutoCloseProgressWindow: (value: boolean) => Promise<void>;
}

const defaultPreferences: AppPreferences = {
  auto_close_progress_window: true,
  auto_close_delay_ms: 3000,
};

export const usePreferencesStore = create<PreferencesStore>((set, get) => ({
  // Initial state
  preferences: defaultPreferences,
  isLoading: false,
  error: null,

  // Load preferences from backend
  loadPreferences: async () => {
    set({ isLoading: true, error: null });
    try {
      const preferences = await invoke<AppPreferences>('load_preferences');
      set({ preferences, isLoading: false });
    } catch (error) {
      set({ error: String(error), isLoading: false });
    }
  },

  // Save preferences to backend
  savePreferences: async (preferences: AppPreferences) => {
    set({ isLoading: true, error: null });
    try {
      await invoke('save_preferences', { preferences });
      set({ preferences, isLoading: false });
    } catch (error) {
      set({ error: String(error), isLoading: false });
    }
  },

  // Convenience method to toggle auto-close
  setAutoCloseProgressWindow: async (value: boolean) => {
    const currentPrefs = get().preferences;
    const newPrefs = { ...currentPrefs, auto_close_progress_window: value };
    await get().savePreferences(newPrefs);
  },
}));
