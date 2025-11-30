import { useEffect } from 'react';
import { Layout } from './ui/components/Layout';
import { FolderSelector } from './ui/components/FolderSelector';
import { RestoreSelector } from './ui/components/RestoreSelector';
import { BackupList } from './ui/components/BackupList';
import { useBackupStore, BackupConfig } from './store/useBackupStore';
import { getCurrentWindow } from '@tauri-apps/api/window';

function App() {
  const { loadConfigs, saveConfig, isLoading, error } = useBackupStore();

  // Load configs on mount
  useEffect(() => {
    loadConfigs();
  }, [loadConfigs]);

  // Emit "window-ready" event when React finishes rendering
  useEffect(() => {
    const emitReady = async () => {
      try {
        const window = getCurrentWindow();
        await window.emit('window-ready', { label: window.label });
      } catch (error) {
        console.error('[App] Failed to emit window-ready:', error);
      }
    };

    const timer = setTimeout(emitReady, 100);
    return () => clearTimeout(timer);
  }, []);

  const handleFolderSelected = async (
    sourcePath: string,
    destinationPath: string,
    backupType: 'full' | 'incremental'
  ) => {
    const folderName = sourcePath.split('/').pop() || 'Backup';
    const timestamp = Date.now();

    const newConfig: BackupConfig = {
      id: `backup-${timestamp}`,
      name: `${folderName} Backup`,
      source_path: sourcePath,
      destination_path: destinationPath,
      schedule: null,
      enabled: true,
      mode: 'compressed',
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

  return (
    <Layout>
      <div className="max-w-4xl mx-auto space-y-8">
        {error && (
          <div className="bg-red-900/20 border border-red-800 text-red-300 px-4 py-3 rounded">
            <p className="font-medium">Error</p>
            <p className="text-sm">{error}</p>
          </div>
        )}

        {isLoading && (
          <div className="text-center text-gray-400 py-4">
            Loading...
          </div>
        )}

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
