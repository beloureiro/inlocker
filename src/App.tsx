import { useEffect, useState } from 'react';
import { Layout } from './ui/components/Layout';
import { FolderSelector } from './ui/components/FolderSelector';
import { RestoreSelector } from './ui/components/RestoreSelector';
import { BackupList } from './ui/components/BackupList';
import ScheduledBackupProgress from './ui/components/ScheduledBackupProgress';
import { useBackupStore, BackupConfig } from './store/useBackupStore';

function App() {
  const { loadConfigs, saveConfig, isLoading, error } = useBackupStore();
  const [isScheduledMode, setIsScheduledMode] = useState<boolean | null>(null);

  // Detect if running in scheduled/CLI mode
  useEffect(() => {
    // Use official Tauri CLI plugin to detect --backup argument
    import('@tauri-apps/plugin-cli').then(({ getMatches }) => {
      getMatches()
        .then((matches) => {
          // Check if --backup argument was provided
          const backupArg = matches.args.backup;
          const isScheduled = backupArg && backupArg.value !== null;
          setIsScheduledMode(isScheduled);

          if (isScheduled) {
            console.log('Running in scheduled mode for backup:', backupArg.value);
          }
        })
        .catch((error) => {
          console.error('Failed to get CLI matches:', error);
          setIsScheduledMode(false);
        });
    });
  }, []);

  // Load configs on mount (only in normal mode)
  useEffect(() => {
    if (isScheduledMode === false) {
      loadConfigs();
    }
  }, [loadConfigs, isScheduledMode]);

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
    return (
      <div className="min-h-screen bg-gradient-to-br from-blue-50 to-indigo-100 flex items-center justify-center">
        <div className="text-center">
          <div className="w-16 h-16 border-4 border-blue-500 border-t-transparent rounded-full animate-spin mx-auto mb-4"></div>
          <p className="text-gray-700 text-lg">Iniciando InLocker...</p>
        </div>
      </div>
    );
  }

  // Show progress UI for scheduled backups
  if (isScheduledMode === true) {
    return <ScheduledBackupProgress />;
  }

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
