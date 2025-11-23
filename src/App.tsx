import { useEffect, useState } from 'react';
import { Layout } from './ui/components/Layout';
import { FolderSelector } from './ui/components/FolderSelector';
import { RestoreSelector } from './ui/components/RestoreSelector';
import { BackupList } from './ui/components/BackupList';
import ScheduledBackupProgress from './ui/components/ScheduledBackupProgress';
import { useBackupStore, BackupConfig } from './store/useBackupStore';
import { getCurrentWindow } from '@tauri-apps/api/window';

function App() {
  const { loadConfigs, saveConfig, isLoading, error } = useBackupStore();
  const [isScheduledMode, setIsScheduledMode] = useState<boolean | null>(null);

  // Detect if running in scheduled/CLI mode or in progress window
  useEffect(() => {
    // Check URL parameter first (for scheduled-progress window)
    const urlParams = new URLSearchParams(window.location.search);
    const windowParam = urlParams.get('window');

    if (windowParam === 'progress') {
      console.log('[App] Running in progress window (URL param detected)');
      setIsScheduledMode(true);
      return;
    }

    // Otherwise check CLI args
    const detectCLI = async () => {
      try {
        const { getMatches } = await import('@tauri-apps/plugin-cli');
        const matches = await getMatches();
        const backupArg = matches.args.backup;
        const isScheduled = backupArg && backupArg.value !== null;
        setIsScheduledMode(isScheduled);
      } catch (error) {
        console.error('[App] Error detecting CLI mode:', error);
        setIsScheduledMode(false);
      }
    };

    detectCLI();
  }, []);

  // Load configs on mount (only in normal mode)
  useEffect(() => {
    if (isScheduledMode === false) {
      loadConfigs();
    }
  }, [loadConfigs, isScheduledMode]);

  // Emit "window-ready" event when React finishes rendering
  useEffect(() => {
    const emitReady = async () => {
      try {
        const window = getCurrentWindow();
        await window.emit('window-ready', { label: window.label });
        console.log('[App] Emitted window-ready event for:', window.label);
      } catch (error) {
        console.error('[App] Failed to emit window-ready:', error);
      }
    };

    // Emit after a small delay to ensure render is complete
    const timer = setTimeout(emitReady, 100);
    return () => clearTimeout(timer);
  }, []);

  const handleFolderSelected = async (
    sourcePath: string,
    destinationPath: string,
    backupType: 'full' | 'incremental'
  ) => {
    // Create a simple backup name from the source folder
    const folderName = sourcePath.split('/').pop() || 'Backup';
    const timestamp = Date.now();

    const newConfig: BackupConfig = {
      id: `backup-${timestamp}`,
      name: `${folderName} Backup`,
      source_path: sourcePath,
      destination_path: destinationPath,
      schedule: null,
      enabled: true,
      mode: 'compressed', // Default to compressed mode
      backup_type: backupType,
      created_at: timestamp,
      updated_at: timestamp,
      last_backup_at: null,
      last_backup_original_size: null,
      last_backup_compressed_size: null,
      last_backup_files_count: null,
      last_backup_checksum: null,
    };

    await saveConfig(newConfig);
  };

  // Show loading while detecting mode
  if (isScheduledMode === null) {
    console.log('[App] Rendering loading state (isScheduledMode is null)');
    return (
      <div className="min-h-screen bg-gradient-to-br from-blue-50 to-indigo-100 flex items-center justify-center">
        <div className="text-center">
          <div className="w-16 h-16 border-4 border-blue-500 border-t-transparent rounded-full animate-spin mx-auto mb-4"></div>
          <p className="text-gray-700 text-lg">Iniciando InLocker...</p>
          <p className="text-gray-500 text-sm mt-2">Detectando modo de execução...</p>
        </div>
      </div>
    );
  }

  // Show progress UI for scheduled backups
  if (isScheduledMode === true) {
    console.log('[App] Rendering ScheduledBackupProgress component');
    return <ScheduledBackupProgress />;
  }

  console.log('[App] Rendering normal app UI');

  // Normal app UI
  return (
    <Layout>
      <div className="max-w-4xl mx-auto space-y-8">
        {/* Error Display */}
        {error && (
          <div className="bg-red-900/20 border border-red-800 text-red-300 px-4 py-3 rounded">
            <p className="font-medium">Error</p>
            <p className="text-sm">{error}</p>
          </div>
        )}

        {/* Loading State */}
        {isLoading && (
          <div className="text-center text-gray-400 py-4">
            Loading...
          </div>
        )}

        {/* Folder Selector */}
        {!isLoading && (
          <>
            <FolderSelector onFolderSelected={handleFolderSelected} />
            <RestoreSelector />
            <BackupList />
          </>
        )}
      </div>
    </Layout>
  );
}

export default App;
