import { useEffect } from 'react';
import { Layout } from './ui/components/Layout';
import { FolderSelector } from './ui/components/FolderSelector';
import { BackupList } from './ui/components/BackupList';
import { useBackupStore, BackupConfig } from './store/useBackupStore';

function App() {
  const { loadConfigs, saveConfig, isLoading, error } = useBackupStore();

  // Load configs on mount
  useEffect(() => {
    loadConfigs();
  }, [loadConfigs]);

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
      encrypt: false,
      backup_type: backupType,
      created_at: timestamp,
      updated_at: timestamp,
      last_backup_at: null,
    };

    await saveConfig(newConfig);
  };

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
            <BackupList />
          </>
        )}
      </div>
    </Layout>
  );
}

export default App;
