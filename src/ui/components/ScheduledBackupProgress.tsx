import { useEffect, useState } from 'react';
import { listen } from '@tauri-apps/api/event';

interface ProgressData {
  stage: string;
  message: string;
  percentage?: number;
  files_processed?: number;
  total_files?: number;
}

export default function ScheduledBackupProgress() {
  console.log('[ScheduledBackupProgress] Component mounting...');
  const [progress, setProgress] = useState<ProgressData>({
    stage: 'initializing',
    message: 'Starting scheduled backup...',
  });

  useEffect(() => {
    console.log('[ScheduledBackupProgress] Setting up event listeners');

    // Listen to test-backup-trigger (from Test Now button)
    const unlistenTrigger = listen<string>('test-backup-trigger', async (event) => {
      console.log('[ScheduledBackupProgress] Test backup trigger received for config:', event.payload);
      const configId = event.payload;

      try {
        // Import invoke dynamically
        const { invoke } = await import('@tauri-apps/api/core');
        console.log('[ScheduledBackupProgress] Calling run_backup_now for config:', configId);
        await invoke('run_backup_now', { configId, password: undefined });
      } catch (error) {
        console.error('[ScheduledBackupProgress] Failed to run backup:', error);
        setProgress({
          stage: 'error',
          message: `Error: ${error}`,
        });
      }
    });

    // Listen to backup progress events from Rust
    const unlistenProgress = listen<ProgressData>('backup:progress', (event) => {
      console.log('[ScheduledBackupProgress] Progress event received:', event.payload);
      setProgress(event.payload);
    });

    return () => {
      console.log('[ScheduledBackupProgress] Cleaning up event listeners');
      unlistenTrigger.then((fn) => fn());
      unlistenProgress.then((fn) => fn());
    };
  }, []);

  const getProgressPercentage = () => {
    if (progress.percentage !== undefined) {
      return progress.percentage;
    }
    if (progress.files_processed && progress.total_files) {
      return Math.round((progress.files_processed / progress.total_files) * 100);
    }
    return 0;
  };

  const percentage = getProgressPercentage();

  return (
    <div className="min-h-screen bg-gray-950 flex flex-col items-center justify-center p-6">
      {/* Logo */}
      <div className="flex items-center gap-3 mb-8">
        <div className="w-10 h-10 bg-emerald-600 rounded-lg flex items-center justify-center">
          <svg className="w-6 h-6 text-white" fill="none" stroke="currentColor" strokeWidth="2" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" d="M16.5 10.5V6.75a4.5 4.5 0 1 0-9 0v3.75m-.75 11.25h10.5a2.25 2.25 0 0 0 2.25-2.25v-6.75a2.25 2.25 0 0 0-2.25-2.25H6.75a2.25 2.25 0 0 0-2.25 2.25v6.75a2.25 2.25 0 0 0 2.25 2.25Z" />
          </svg>
        </div>
        <div>
          <h1 className="text-xl font-semibold text-white">InLocker</h1>
          <p className="text-xs text-gray-400">Scheduled Backup</p>
        </div>
      </div>

      {/* Progress Card */}
      <div className="bg-gray-900 border border-gray-800 rounded-lg p-6 w-full max-w-md">
        {/* Spinner */}
        <div className="flex justify-center mb-6">
          <svg
            className="w-12 h-12 text-emerald-500 animate-spin"
            fill="none"
            viewBox="0 0 24 24"
          >
            <circle
              className="opacity-25"
              cx="12"
              cy="12"
              r="10"
              stroke="currentColor"
              strokeWidth="4"
            />
            <path
              className="opacity-75"
              fill="currentColor"
              d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
            />
          </svg>
        </div>

        {/* Stage */}
        <div className="text-center mb-4">
          <p className="text-sm text-gray-400 uppercase tracking-wider mb-1">{progress.stage}</p>
          <p className="text-gray-200">{progress.message}</p>
        </div>

        {/* Progress Bar */}
        <div className="mb-4">
          <div className="flex justify-between text-xs text-gray-400 mb-2">
            <span>Progress</span>
            <span className="font-mono">{percentage}%</span>
          </div>
          <div className="w-full bg-gray-800 rounded-full h-2 overflow-hidden">
            <div
              className="bg-emerald-600 h-full rounded-full transition-all duration-300 ease-out"
              style={{ width: `${percentage}%` }}
            />
          </div>
        </div>

        {/* File Counter */}
        {progress.files_processed !== undefined && progress.total_files !== undefined && (
          <div className="text-center text-xs text-gray-400 font-mono">
            {progress.files_processed.toLocaleString()} / {progress.total_files.toLocaleString()} files
          </div>
        )}
      </div>

      {/* Footer */}
      <p className="text-xs text-gray-500 text-center mt-6 max-w-xs">
        Window will close automatically when complete
      </p>
    </div>
  );
}
