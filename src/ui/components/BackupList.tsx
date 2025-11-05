import { useState, useEffect, useRef } from 'react';
import { useBackupStore, BackupConfig } from '../../store/useBackupStore';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { BackupConfigModal } from './BackupConfigModal';

interface BackupProgress {
  config_id: string;
  stage: string;
  message: string;
  details?: string;
  current?: number;  // Files processed so far
  total?: number;     // Total files to process
}

export function BackupList() {
  const { configs, deleteConfig, loadConfigs, saveConfig } = useBackupStore();
  const [runningBackups, setRunningBackups] = useState<Set<string>>(new Set());
  const [backupResults, setBackupResults] = useState<Map<string, any>>(new Map());
  const [editingConfig, setEditingConfig] = useState<BackupConfig | null>(null);
  const [backupStartTimes, setBackupStartTimes] = useState<Map<string, number>>(new Map());
  const [elapsedTimes, setElapsedTimes] = useState<Map<string, number>>(new Map());
  const [backupProgress, setBackupProgress] = useState<Map<string, BackupProgress>>(new Map());

  // Debounce loadConfigs to avoid multiple re-renders during parallel backups
  const loadConfigsTimeoutRef = useRef<NodeJS.Timeout | null>(null);

  // Helper: Debounced loadConfigs to prevent multiple re-renders when backups complete
  const debouncedLoadConfigs = () => {
    if (loadConfigsTimeoutRef.current) {
      clearTimeout(loadConfigsTimeoutRef.current);
    }
    loadConfigsTimeoutRef.current = setTimeout(() => {
      loadConfigs();
    }, 500); // Wait 500ms after last backup completes
  };

  // Listen to backup progress events
  useEffect(() => {
    const unlisten = listen<BackupProgress>('backup:progress', (event) => {
      console.log('[BackupList] Progress event:', event.payload);
      setBackupProgress((prev) => new Map(prev).set(event.payload.config_id, event.payload));
    });

    return () => {
      unlisten.then(fn => fn());
      // Clean up debounce timeout on unmount
      if (loadConfigsTimeoutRef.current) {
        clearTimeout(loadConfigsTimeoutRef.current);
      }
    };
  }, []);

  // Update elapsed time every second for running backups
  useEffect(() => {
    const interval = setInterval(() => {
      const now = Date.now();
      setElapsedTimes((prev) => {
        const newMap = new Map(prev);
        backupStartTimes.forEach((startTime, configId) => {
          if (runningBackups.has(configId)) {
            newMap.set(configId, Math.floor((now - startTime) / 1000));
          }
        });
        return newMap;
      });
    }, 1000);

    return () => clearInterval(interval);
  }, [runningBackups, backupStartTimes]);

  if (configs.length === 0) {
    return (
      <div className="bg-gray-900 rounded-lg border border-gray-800 p-8 text-center">
        <p className="text-gray-400">No backup configurations yet.</p>
        <p className="text-sm text-gray-500 mt-2">Add your first backup above to get started.</p>
      </div>
    );
  }

  const handleDelete = async (configId: string) => {
    if (confirm('Are you sure you want to delete this backup configuration?')) {
      await deleteConfig(configId);
    }
  };

  const handleSaveConfig = async (updatedConfig: BackupConfig) => {
    await saveConfig(updatedConfig);

    // Register or unregister schedule based on config
    try {
      if (updatedConfig.schedule && updatedConfig.schedule.enabled) {
        // Register the schedule
        await invoke('register_schedule', { configId: updatedConfig.id });
        console.log(`Schedule registered for config: ${updatedConfig.id}`);
      } else {
        // Unregister the schedule if it was removed
        try {
          await invoke('unregister_schedule', { configId: updatedConfig.id });
          console.log(`Schedule unregistered for config: ${updatedConfig.id}`);
        } catch (err) {
          // It's ok if there was no schedule to unregister
          console.log(`No schedule to unregister for config: ${updatedConfig.id}`);
        }
      }
    } catch (error) {
      console.error('Error managing schedule:', error);
      alert(`Schedule error: ${error}`);
    }

    setEditingConfig(null);
  };

  const handleRunBackup = async (configId: string) => {
    console.log('[BackupList] Starting backup for config:', configId);
    const startTime = Date.now();

    setRunningBackups((prev) => new Set(prev).add(configId));
    setBackupStartTimes((prev) => new Map(prev).set(configId, startTime));
    setBackupResults((prev) => {
      const newMap = new Map(prev);
      newMap.delete(configId);
      return newMap;
    });

    try {
      console.log('[BackupList] Calling run_backup_now...');
      const result = await invoke('run_backup_now', { configId });
      console.log('[BackupList] Backup result:', result);

      setBackupResults((prev) => {
        const newMap = new Map(prev).set(configId, result);
        console.log('[BackupList] Updated backupResults, size:', newMap.size);
        return newMap;
      });

      // Show success alert
      if ((result as any).success) {
        console.log('[BackupList] Backup successful!');
      }
    } catch (error) {
      console.error('[BackupList] Backup error:', error);
      const errorMessage = String(error);

      setBackupResults((prev) =>
        new Map(prev).set(configId, {
          success: false,
          message: errorMessage,
        })
      );

      // Show error alert
      alert(`Backup failed:\n${errorMessage}`);
    } finally {
      // Clean up UI state for this backup
      setRunningBackups((prev) => {
        const newSet = new Set(prev);
        newSet.delete(configId);
        console.log('[BackupList] Removed from running backups. Remaining:', newSet.size);
        return newSet;
      });
      setBackupProgress((prev) => {
        const newMap = new Map(prev);
        newMap.delete(configId);
        return newMap;
      });

      // Reload configs with debounce to avoid multiple re-renders during parallel backups
      // This updates last_backup_at timestamp in the UI
      debouncedLoadConfigs();

      console.log('[BackupList] Backup process finished for:', configId);
    }
  };

  const handleCancelBackup = async (configId: string) => {
    if (confirm('Are you sure you want to cancel this backup?')) {
      try {
        // Call backend to cancel the backup
        const cancelled = await invoke<boolean>('cancel_backup', { configId });

        if (cancelled) {
          console.log('[BackupList] Backup cancellation requested for:', configId);
          // The backup will fail with "Backup cancelled by user" error
          // The finally block in handleRunBackup will clean up the UI state
        } else {
          console.warn('[BackupList] No running backup found for:', configId);
          // Clean up UI state anyway
          setRunningBackups((prev) => {
            const newSet = new Set(prev);
            newSet.delete(configId);
            return newSet;
          });
          setBackupProgress((prev) => {
            const newMap = new Map(prev);
            newMap.delete(configId);
            return newMap;
          });
        }
      } catch (error) {
        console.error('[BackupList] Failed to cancel backup:', error);
        alert(`Failed to cancel backup: ${error}`);
      }
    }
  };

  const handleRestore = async (configId: string) => {
    try {
      // List available backups
      const backups = await invoke('list_available_backups', { configId });

      if (!Array.isArray(backups) || backups.length === 0) {
        alert('No backups found for this configuration');
        return;
      }

      // Show list of backups to choose from
      const backupsList = backups.map((b: any, i: number) =>
        `${i + 1}. ${b.filename} (${(b.size / 1048576).toFixed(1)} MB) - ${new Date(b.created_at * 1000).toLocaleString()}`
      ).join('\n');

      const choice = prompt(`Select backup to restore:\n\n${backupsList}\n\nEnter number (1-${backups.length}):`);

      if (!choice) return;

      const index = parseInt(choice) - 1;
      if (isNaN(index) || index < 0 || index >= backups.length) {
        alert('Invalid selection');
        return;
      }

      const selectedBackup = backups[index];

      // Check if backup is encrypted (needs password)
      const isEncrypted = selectedBackup.filename.endsWith('.enc');
      let password: string | null = null;

      if (isEncrypted) {
        password = prompt('This backup is encrypted.\n\nPlease enter the password used during backup:');
        if (!password) {
          alert('Password is required to restore encrypted backups');
          return;
        }
      }

      // Ask for restore destination
      const destination = await invoke<string | null>('select_folder');
      if (!destination) return;

      // Confirm restore
      if (!confirm(`Restore backup to:\n${destination}\n\nThis will extract all files from the backup. Continue?`)) {
        return;
      }

      // Perform restore
      setRunningBackups((prev) => new Set(prev).add(configId + '-restore'));

      // Get checksum from config for integrity verification
      const config = configs.find(c => c.id === configId);
      const expectedChecksum = config?.last_backup_checksum || null;

      const result = await invoke('restore_backup', {
        backupFilePath: selectedBackup.path,
        restoreDestination: destination,
        expectedChecksum,
        password,
      });

      setBackupResults((prev) =>
        new Map(prev).set(configId, {
          success: true,
          message: `Restore completed: ${(result as any).files_count} files restored`,
        })
      );

      alert(`Restore successful!\n${(result as any).files_count} files restored to:\n${destination}`);
    } catch (error) {
      setBackupResults((prev) =>
        new Map(prev).set(configId, {
          success: false,
          message: `Restore failed: ${error}`,
        })
      );
      alert(`Restore failed: ${error}`);
    } finally {
      setRunningBackups((prev) => {
        const newSet = new Set(prev);
        newSet.delete(configId + '-restore');
        return newSet;
      });
    }
  };

  return (
    <>
      {editingConfig && (
        <BackupConfigModal
          config={editingConfig}
          onSave={handleSaveConfig}
          onClose={() => setEditingConfig(null)}
        />
      )}

      <div className="space-y-3">
        <h2 className="text-sm font-semibold text-gray-300">Saved Backups</h2>
        {configs.map((config: BackupConfig) => {
        const isRunning = runningBackups.has(config.id);
        const result = backupResults.get(config.id);

        return (
          <div
            key={config.id}
            className="bg-gray-900 rounded border border-gray-800 hover:border-gray-700 transition-colors overflow-hidden"
          >
            <div className="p-3">
              <div className="flex items-start justify-between gap-4">
                <div className="flex-1 min-w-0">
                <div className="flex items-center gap-2 mb-2 flex-wrap">
                  <h3 className="text-sm font-medium text-white">{config.name}</h3>
                  <span className={`px-2 py-0.5 rounded text-xs ${config.backup_type === 'full' ? 'bg-blue-900 text-blue-300' : 'bg-purple-900 text-purple-300'}`}>
                    {config.backup_type === 'full' ? 'Full' : 'Incremental'}
                  </span>
                  <span className={`px-2 py-0.5 rounded text-xs ${config.enabled ? 'bg-emerald-900 text-emerald-300' : 'bg-gray-800 text-gray-400'}`}>
                    {config.enabled ? 'Enabled' : 'Disabled'}
                  </span>
                  {config.mode === 'copy' && (
                    <span className="px-2 py-0.5 rounded text-xs bg-gray-700/50 text-gray-300">
                      Copy
                    </span>
                  )}
                  {config.mode === 'compressed' && (
                    <span className="px-2 py-0.5 rounded text-xs bg-emerald-900/50 text-emerald-400">
                      Compressed
                    </span>
                  )}
                  {config.mode === 'encrypted' && (
                    <span className="px-2 py-0.5 rounded text-xs bg-amber-900/50 text-amber-400 flex items-center gap-1">
                      <svg className="w-3 h-3" fill="none" stroke="currentColor" strokeWidth="2" viewBox="0 0 24 24">
                        <path strokeLinecap="round" strokeLinejoin="round" d="M16.5 10.5V6.75a4.5 4.5 0 1 0-9 0v3.75m-.75 11.25h10.5a2.25 2.25 0 0 0 2.25-2.25v-6.75a2.25 2.25 0 0 0-2.25-2.25H6.75a2.25 2.25 0 0 0-2.25 2.25v6.75a2.25 2.25 0 0 0 2.25 2.25Z" />
                      </svg>
                      Encrypted
                    </span>
                  )}
                  {config.schedule?.enabled && config.schedule?.cron_expression && (
                    <span className="px-1.5 py-0.5 rounded text-xs bg-blue-900/50 text-blue-400 flex items-center" title="Scheduled">
                      <svg className="w-3.5 h-3.5" fill="none" stroke="currentColor" strokeWidth="2" viewBox="0 0 24 24">
                        <path strokeLinecap="round" strokeLinejoin="round" d="M12 6v6h4.5m4.5 0a9 9 0 11-18 0 9 9 0 0118 0z" />
                      </svg>
                    </span>
                  )}
                </div>
                <div className="text-sm space-y-1.5">
                  <div className="flex items-center gap-2">
                    <span className="text-gray-400">From:</span>
                    <span className="text-gray-300 font-mono text-xs">{config.source_path}</span>
                  </div>
                  <div className="flex items-center gap-2">
                    <span className="text-gray-400">To:</span>
                    <span className="text-gray-300 font-mono text-xs">{config.destination_path}</span>
                  </div>

                  {/* Grid layout for stats - 2x2 grid */}
                  <div className="grid grid-cols-2 gap-x-8 gap-y-1 pt-1">
                    {/* Row 1, Col 1 */}
                    {config.last_backup_at && (
                      <div className="flex items-baseline gap-2">
                        <span className="text-gray-400 text-xs whitespace-nowrap">Last:</span>
                        <span className="text-gray-300 text-xs whitespace-nowrap">
                          {new Date(config.last_backup_at * 1000).toLocaleString()}
                        </span>
                      </div>
                    )}

                    {/* Row 1, Col 2 */}
                    {config.last_backup_original_size && config.last_backup_compressed_size && (
                      <div className="flex items-baseline gap-2">
                        <span className="text-gray-400 text-xs whitespace-nowrap">Size:</span>
                        <span className="text-gray-300 font-mono text-xs whitespace-nowrap">
                          {(config.last_backup_original_size / 1048576).toFixed(1)} MB → {(config.last_backup_compressed_size / 1048576).toFixed(1)} MB
                          <span className="text-gray-500 ml-1">
                            ({((1 - config.last_backup_compressed_size / config.last_backup_original_size) * 100).toFixed(0)}%)
                          </span>
                        </span>
                      </div>
                    )}

                    {/* Row 2, Col 1 */}
                    {config.last_backup_files_count !== null && config.last_backup_files_count !== undefined && (
                      <div className="flex items-baseline gap-2">
                        <span className="text-gray-400 text-xs whitespace-nowrap">Files:</span>
                        <span className="text-gray-300 text-xs whitespace-nowrap">
                          {config.last_backup_files_count.toLocaleString()}
                        </span>
                      </div>
                    )}

                    {/* Row 2, Col 2 */}
                    {config.schedule && (
                      <div className="flex items-baseline gap-2">
                        <span className="text-gray-400 text-xs whitespace-nowrap">Schedule:</span>
                        <span className="text-gray-300 font-mono text-xs whitespace-nowrap">
                          {config.schedule.preset && config.schedule.preset !== 'custom'
                            ? config.schedule.preset.charAt(0).toUpperCase() + config.schedule.preset.slice(1)
                            : config.schedule.cron_expression}
                        </span>
                      </div>
                    )}
                  </div>
                </div>
              </div>

              <div className="flex flex-col gap-1.5 min-w-[110px]">
                  <button
                    onClick={() => handleRunBackup(config.id)}
                    disabled={isRunning}
                    className="px-3 py-1 bg-emerald-700 hover:bg-emerald-600 disabled:bg-gray-700 disabled:cursor-not-allowed rounded text-xs font-medium transition-colors whitespace-nowrap"
                  >
                    {isRunning ? 'Running...' : 'Run Backup'}
                  </button>
                  {/* Only show Restore button for compressed or encrypted backups */}
                  {config.mode !== 'copy' && (
                    <button
                      onClick={() => handleRestore(config.id)}
                      disabled={isRunning}
                      className="px-3 py-1 bg-blue-700 hover:bg-blue-600 disabled:bg-gray-700 disabled:cursor-not-allowed rounded text-xs transition-colors flex items-center justify-center gap-1 whitespace-nowrap"
                      title="Restore from backup"
                    >
                      <svg className="w-3 h-3" fill="none" stroke="currentColor" strokeWidth="2" viewBox="0 0 24 24">
                        <path strokeLinecap="round" strokeLinejoin="round" d="M9 15L3 9m0 0l6-6M3 9h12a6 6 0 010 12h-3" />
                      </svg>
                      Restore
                    </button>
                  )}
                  <button
                    onClick={() => setEditingConfig(config)}
                    className="px-3 py-1 text-xs text-gray-300 hover:text-white hover:bg-gray-800 rounded transition-colors flex items-center justify-center gap-1 whitespace-nowrap"
                    title="Configure backup settings"
                  >
                    <svg className="w-3 h-3" fill="none" stroke="currentColor" strokeWidth="2" viewBox="0 0 24 24">
                      <path strokeLinecap="round" strokeLinejoin="round" d="M10.343 3.94c.09-.542.56-.94 1.11-.94h1.093c.55 0 1.02.398 1.11.94l.149.894c.07.424.384.764.78.93.398.164.855.142 1.205-.108l.737-.527a1.125 1.125 0 011.45.12l.773.774c.39.389.44 1.002.12 1.45l-.527.737c-.25.35-.272.806-.107 1.204.165.397.505.71.93.78l.893.15c.543.09.94.56.94 1.109v1.094c0 .55-.397 1.02-.94 1.11l-.893.149c-.425.07-.765.383-.93.78-.165.398-.143.854.107 1.204l.527.738c.32.447.269 1.06-.12 1.45l-.774.773a1.125 1.125 0 01-1.449.12l-.738-.527c-.35-.25-.806-.272-1.203-.107-.397.165-.71.505-.781.929l-.149.894c-.09.542-.56.94-1.11.94h-1.094c-.55 0-1.019-.398-1.11-.94l-.148-.894c-.071-.424-.384-.764-.781-.93-.398-.164-.854-.142-1.204.108l-.738.527c-.447.32-1.06.269-1.45-.12l-.773-.774a1.125 1.125 0 01-.12-1.45l.527-.737c.25-.35.273-.806.108-1.204-.165-.397-.505-.71-.93-.78l-.894-.15c-.542-.09-.94-.56-.94-1.109v-1.094c0-.55.398-1.02.94-1.11l.894-.149c.424-.07.765-.383.93-.78.165-.398.143-.854-.107-1.204l-.527-.738a1.125 1.125 0 01.12-1.45l.773-.773a1.125 1.125 0 011.45-.12l.737.527c.35.25.807.272 1.204.107.397-.165.71-.505.78-.929l.15-.894z" />
                      <path strokeLinecap="round" strokeLinejoin="round" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
                    </svg>
                    Settings
                  </button>
                  <button
                    onClick={() => handleDelete(config.id)}
                    className="px-3 py-1 text-xs text-red-400 hover:text-red-300 hover:bg-red-900/20 rounded transition-colors whitespace-nowrap"
                  >
                    Delete
                  </button>
                </div>
              </div>
            </div>

            {/* Backup status - Outside padding to span full width */}
            {isRunning && (() => {
              const progress = backupProgress.get(config.id);
              return (
                <div className="p-3 pt-0">
                  <div className="p-2 rounded bg-blue-900/30 border border-blue-800 text-blue-300 text-sm">
                    <div className="flex items-center justify-between gap-3">
                      <div className="flex items-center gap-2 flex-1 min-w-0">
                        <svg className="animate-spin h-4 w-4 flex-shrink-0" fill="none" viewBox="0 0 24 24">
                          <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4"></circle>
                          <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                        </svg>
                        <span className="font-medium whitespace-nowrap">{progress?.message || 'Starting backup...'}</span>
                        {progress?.details && (
                          <>
                            <span className="text-blue-400/50">•</span>
                            <span className="text-xs font-mono opacity-75 whitespace-nowrap">{progress.details}</span>
                          </>
                        )}
                      </div>
                      <div className="flex items-center gap-2 flex-shrink-0">
                        <span className="text-xs font-mono whitespace-nowrap">
                          {elapsedTimes.get(config.id) !== undefined
                            ? `${Math.floor(elapsedTimes.get(config.id)! / 60)}:${String(elapsedTimes.get(config.id)! % 60).padStart(2, '0')}`
                            : '0:00'}
                        </span>
                        <button
                          onClick={() => handleCancelBackup(config.id)}
                          className="p-0.5 hover:bg-red-900/30 rounded transition-colors group"
                          title="Cancel backup"
                        >
                          <svg className="w-3.5 h-3.5 text-blue-400 group-hover:text-red-400" fill="none" stroke="currentColor" strokeWidth="2" viewBox="0 0 24 24">
                            <path strokeLinecap="round" strokeLinejoin="round" d="M6 18L18 6M6 6l12 12" />
                          </svg>
                        </button>
                      </div>
                    </div>
                    {/* Thin progress bar */}
                    {(progress?.current !== undefined && progress?.total !== undefined && progress.total > 0) ||
                     (progress?.stage && ['compressing', 'encrypting', 'writing', 'checksum'].includes(progress.stage)) ? (
                      <div className="mt-2 h-0.5 bg-blue-900/50 rounded-full overflow-hidden">
                        {progress?.current !== undefined && progress?.total !== undefined && progress.total > 0 ? (
                          // Determinate: percentage-based progress (during TAR creation)
                          <div
                            className="h-full bg-blue-400 transition-all duration-300 ease-out"
                            style={{ width: `${Math.min(100, (progress.current / progress.total) * 100)}%` }}
                          />
                        ) : (
                          // Indeterminate: animated striped bar (barberpole effect after TAR)
                          <div
                            className="h-full w-full bg-blue-400"
                            style={{
                              backgroundImage: 'repeating-linear-gradient(45deg, transparent, transparent 4px, rgba(255,255,255,0.3) 4px, rgba(255,255,255,0.3) 8px)',
                              backgroundSize: '12px 12px',
                              animation: 'barberpole 1s linear infinite'
                            }}
                          />
                        )}
                      </div>
                    ) : null}
                  </div>
                </div>
              );
            })()}

            {/* Backup result - Outside padding to span full width */}
            {!isRunning && result && (
              <div className="p-3 pt-0">
                <div
                  className={`p-2 rounded text-sm ${
                    result.success
                      ? 'bg-emerald-900/30 border border-emerald-800 text-emerald-300'
                      : 'bg-red-900/30 border border-red-800 text-red-300'
                  }`}
                >
                  <div className="flex items-start gap-2">
                    {result.success ? (
                      <svg className="w-4 h-4 flex-shrink-0 mt-0.5" fill="currentColor" viewBox="0 0 20 20">
                        <path fillRule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clipRule="evenodd" />
                      </svg>
                    ) : (
                      <svg className="w-4 h-4 flex-shrink-0 mt-0.5" fill="currentColor" viewBox="0 0 20 20">
                        <path fillRule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clipRule="evenodd" />
                      </svg>
                    )}
                    <div className="flex-1">
                      <div className="flex items-center justify-between mb-0.5">
                        <span className="font-medium">
                          {result.success ? 'Backup Successful' : 'Backup Failed'}
                        </span>
                        {result.job && result.job.started_at && result.job.completed_at && (
                          <span className="text-xs font-mono opacity-75">
                            {Math.floor((result.job.completed_at - result.job.started_at) / 60)}m {(result.job.completed_at - result.job.started_at) % 60}s
                          </span>
                        )}
                      </div>
                      <div className="text-xs opacity-90">{result.message}</div>
                    </div>
                  </div>
                </div>
              </div>
            )}
          </div>
          );
        })}
      </div>
    </>
  );
}
