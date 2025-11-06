import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

interface RestoreProgress {
  stage: string;
  message: string;
  details?: string;
  current?: number;
}

export function RestoreSelector() {
  const [backupFilePath, setBackupFilePath] = useState<string>('');
  const [destinationPath, setDestinationPath] = useState<string>('');
  const [isRestoring, setIsRestoring] = useState(false);
  const [restoreProgress, setRestoreProgress] = useState<RestoreProgress | null>(null);
  const [showCancellationInfo, setShowCancellationInfo] = useState(false);
  const [isBrowsingBackup, setIsBrowsingBackup] = useState(false);
  const [isBrowsingDestination, setIsBrowsingDestination] = useState(false);
  const [restoreResult, setRestoreResult] = useState<{ success: boolean; filesCount: number; duration: number } | null>(null);

  // Listen to restore progress events
  useEffect(() => {
    const unlisten = listen<RestoreProgress>('restore:progress', (event) => {
      console.log('[RestoreSelector] Progress event:', event.payload);
      setRestoreProgress(event.payload);
    });

    return () => {
      unlisten.then(fn => fn());
    };
  }, []);

  const handleSelectBackupFile = async () => {
    try {
      setIsBrowsingBackup(true);
      const selected = await invoke<string | null>('select_file');

      if (selected) {
        setBackupFilePath(selected);
        console.log('[RestoreSelector] Selected backup file:', selected);
      }
    } catch (error) {
      console.error('[RestoreSelector] Error selecting backup file:', error);
      alert('Error selecting backup file: ' + error);
    } finally {
      setIsBrowsingBackup(false);
    }
  };

  const handleSelectDestination = async () => {
    try {
      setIsBrowsingDestination(true);
      const selected = await invoke<string | null>('select_folder');

      if (selected) {
        setDestinationPath(selected);
        console.log('[RestoreSelector] Selected destination:', selected);
      }
    } catch (error) {
      console.error('[RestoreSelector] Error selecting destination:', error);
      alert('Error selecting destination: ' + error);
    } finally {
      setIsBrowsingDestination(false);
    }
  };

  const handleCancelRestore = async () => {
    if (!isRestoring || !backupFilePath || !restoreProgress) return;

    try {
      await invoke('cancel_restore', { backupFilePath });
      console.log('[RestoreSelector] Cancel requested');

      // Provide intelligent feedback based on current stage
      const currentStage = restoreProgress.stage;
      let message = 'Cancelling restore...';
      let details = '';

      if (currentStage === 'verifying') {
        details = 'Will cancel after checksum verification completes';
      } else if (currentStage === 'decrypting') {
        details = 'Will cancel after decryption completes (AES-256 cannot be interrupted)';
      } else if (currentStage === 'decompressing') {
        details = 'Will cancel after decompression completes (zstd cannot be interrupted)';
      } else if (currentStage === 'extracting') {
        details = 'Stopping after current file extraction...';
      } else {
        details = 'Cancelling immediately...';
      }

      setRestoreProgress({
        stage: 'cancelling',
        message,
        details
      });
    } catch (error) {
      console.error('[RestoreSelector] Cancel error:', error);
    }
  };

  const handleRestore = async () => {
    if (!backupFilePath || !destinationPath) {
      alert('Please select both a backup file and destination folder');
      return;
    }

    console.log('[RestoreSelector] ===== STARTING RESTORE =====');
    console.log('[RestoreSelector] Backup file:', backupFilePath);
    console.log('[RestoreSelector] Destination:', destinationPath);

    setIsRestoring(true);
    setRestoreProgress({ stage: 'preparing', message: 'Preparing to restore...' });
    setRestoreResult(null);

    const startTime = Date.now();

    const invokeParams = {
      backupFilePath: backupFilePath,
      restoreDestination: destinationPath,
      expectedChecksum: null,
      password: null
    };

    console.log('[RestoreSelector] Calling invoke with params (camelCase):', JSON.stringify(invokeParams, null, 2));

    try {
      // Call the restore_backup command in Rust backend
      const result = await invoke<{
        success: boolean;
        message: string;
        files_count: number;
        started_at: number;
        completed_at: number;
      }>('restore_backup', invokeParams);

      console.log('[RestoreSelector] Restore result:', result);

      if (result.success) {
        const filesCount = result.files_count || 0;
        const duration = Math.round((Date.now() - startTime) / 1000);

        setRestoreProgress({
          stage: 'completed',
          message: `Restore completed successfully! ${filesCount} files restored.`
        });

        setRestoreResult({
          success: true,
          filesCount,
          duration
        });

        setIsRestoring(false);

        // Clear result after 10 seconds
        setTimeout(() => {
          setRestoreResult(null);
        }, 10000);
      } else {
        setRestoreProgress({
          stage: 'failed',
          message: `Restore failed: ${result.message}`
        });
        alert('Restore failed: ' + result.message);
        setIsRestoring(false);
      }
    } catch (error) {
      console.error('[RestoreSelector] Restore error:', error);
      const errorMsg = String(error);
      if (errorMsg.includes('cancelled')) {
        setRestoreProgress({
          stage: 'cancelled',
          message: 'Restore cancelled by user'
        });
      } else {
        setRestoreProgress({
          stage: 'failed',
          message: `Error: ${error}`
        });
        alert('Restore error: ' + error);
      }
      setIsRestoring(false);
    }
  };

  return (
    <div className="bg-gray-900 rounded-lg border border-gray-800 p-3">
      <h2 className="text-base font-semibold mb-3">
        Restore from Backup <span className="text-xs text-gray-400 font-normal">(Select backup file and destination)</span>
      </h2>

      <div className="grid grid-cols-2 gap-2 mb-2">
        {/* Backup File Selection */}
        <div className="flex gap-2">
          <input
            type="text"
            value={backupFilePath}
            onChange={(e) => setBackupFilePath(e.target.value)}
            placeholder="Backup file path (.zst or .enc)..."
            className="flex-1 bg-gray-800 border border-gray-700 rounded px-3 py-2 text-sm text-gray-300 placeholder-gray-500"
            title="Type path or click Browse"
            disabled={isRestoring}
          />
          <button
            onClick={handleSelectBackupFile}
            disabled={isRestoring || isBrowsingBackup}
            className="px-4 py-2 bg-blue-700 hover:bg-blue-600 disabled:bg-gray-700 disabled:cursor-not-allowed rounded text-sm font-medium transition-colors whitespace-nowrap flex items-center gap-2"
            title="Browse for backup file"
          >
            {isBrowsingBackup ? (
              <>
                <svg className="animate-spin h-4 w-4" fill="none" viewBox="0 0 24 24">
                  <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4"></circle>
                  <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                </svg>
                Opening...
              </>
            ) : (
              'Browse'
            )}
          </button>
        </div>

        {/* Destination Folder */}
        <div className="flex gap-2">
          <input
            type="text"
            value={destinationPath}
            onChange={(e) => setDestinationPath(e.target.value)}
            placeholder="Destination folder path..."
            className="flex-1 bg-gray-800 border border-gray-700 rounded px-3 py-2 text-sm text-gray-300 placeholder-gray-500"
            title="Type path or click Browse"
            disabled={isRestoring}
          />
          <button
            onClick={handleSelectDestination}
            disabled={isRestoring || isBrowsingDestination}
            className="px-4 py-2 bg-blue-700 hover:bg-blue-600 disabled:bg-gray-700 disabled:cursor-not-allowed rounded text-sm font-medium transition-colors whitespace-nowrap flex items-center gap-2"
            title="Browse for folder"
          >
            {isBrowsingDestination ? (
              <>
                <svg className="animate-spin h-4 w-4" fill="none" viewBox="0 0 24 24">
                  <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4"></circle>
                  <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                </svg>
                Opening...
              </>
            ) : (
              'Browse'
            )}
          </button>
        </div>
      </div>

      {/* Restore Button and Progress */}
      <div className="space-y-2">
        <div className="flex gap-2 items-center">
          <button
            type="button"
            onClick={handleRestore}
            disabled={!backupFilePath || !destinationPath || isRestoring}
            className="flex-1 px-4 py-2 bg-blue-600 hover:bg-blue-500 disabled:bg-gray-700 disabled:cursor-not-allowed rounded text-sm font-medium transition-colors whitespace-nowrap"
            title="Start restore operation"
          >
            {isRestoring ? 'Restoring...' : 'Restore Files'}
          </button>

          {/* Cancel Button - only show when restoring */}
          {isRestoring && (
            <button
              type="button"
              onClick={handleCancelRestore}
              className="px-3 py-2 bg-red-700 hover:bg-red-600 rounded text-sm font-medium transition-colors"
              title="Cancel restore"
            >
              <svg className="w-4 h-4" fill="none" stroke="currentColor" strokeWidth="2" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" d="M6 18L18 6M6 6l12 12" />
              </svg>
            </button>
          )}
        </div>

        {/* Progress Bar */}
        {isRestoring && restoreProgress && (
          <div className="w-full">
            <div className="flex items-center justify-between text-xs mb-1">
              <span className="text-gray-400">{restoreProgress.message}</span>
              {restoreProgress.details && (
                <span className={`text-gray-500 ${restoreProgress.stage === 'cancelling' ? 'text-yellow-500' : ''}`}>
                  {restoreProgress.details}
                </span>
              )}
            </div>
            <div className="w-full bg-gray-800 rounded-full h-2 overflow-hidden">
              <div
                className={`h-full ${
                  restoreProgress.stage === 'cancelling'
                    ? 'bg-yellow-600'
                    : 'bg-gradient-to-r from-blue-600 via-blue-500 to-blue-600 bg-[length:200%_100%] animate-[shimmer_2s_linear_infinite]'
                }`}
                style={{
                  width: '100%'
                }}
              />
            </div>

            {/* Stage-specific intelligent information */}
            <div className="mt-1.5 text-xs">
              {restoreProgress.stage === 'verifying' && (
                <div className="text-gray-400">
                  Verifying integrity - cannot cancel during this operation
                </div>
              )}
              {restoreProgress.stage === 'decrypting' && (
                <div className="text-gray-400">
                  Decrypting with AES-256-GCM - cannot cancel during this operation
                </div>
              )}
              {restoreProgress.stage === 'decompressing' && (
                <div className="text-gray-400">
                  Decompressing with zstd - cannot cancel during this operation
                </div>
              )}
              {restoreProgress.stage === 'extracting' && (
                <div className="text-green-400">
                  Extracting files - you can cancel now by clicking X button
                </div>
              )}
              {restoreProgress.stage === 'cancelling' && (
                <div className="text-yellow-400 font-medium">
                  {restoreProgress.details}
                </div>
              )}
            </div>
          </div>
        )}

        {/* Success/Error Message */}
        {!isRestoring && restoreProgress && (
          <div className={`text-sm px-3 py-2 rounded ${
            restoreProgress.stage === 'completed'
              ? 'bg-emerald-900/30 text-emerald-300 border border-emerald-800'
              : restoreProgress.stage === 'cancelled'
              ? 'bg-gray-800/30 text-gray-400 border border-gray-700'
              : 'bg-red-900/30 text-red-300 border border-red-800'
          }`}>
            {restoreProgress.message}
          </div>
        )}

        {/* Success Result Box (similar to Backup Successful) */}
        {restoreResult && restoreResult.success && (
          <div className="p-3 bg-emerald-900/30 border border-emerald-800 rounded-lg">
            <div className="flex items-center justify-between mb-1">
              <div className="flex items-center gap-2">
                <svg className="w-5 h-5 text-emerald-400" fill="none" stroke="currentColor" strokeWidth="2" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" d="M9 12.75L11.25 15 15 9.75M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                </svg>
                <span className="text-emerald-300 font-medium">Restore Successful</span>
              </div>
              <span className="text-emerald-400 text-sm">{restoreResult.duration}s</span>
            </div>
            <div className="text-sm text-emerald-300/80">
              Restored: {restoreResult.filesCount.toLocaleString()} files to {destinationPath}
            </div>
          </div>
        )}

        {/* Collapsible Cancellation Info */}
        <div className="border-t border-gray-800 pt-2">
          <button
            onClick={() => setShowCancellationInfo(!showCancellationInfo)}
            className="w-full flex items-center justify-between text-xs text-gray-400 hover:text-gray-300 transition-colors p-2 hover:bg-gray-800/50 rounded"
          >
            <span className="font-medium">Cancellation Behavior</span>
            <svg
              className={`w-4 h-4 transition-transform ${showCancellationInfo ? 'rotate-180' : ''}`}
              fill="none"
              stroke="currentColor"
              strokeWidth="2"
              viewBox="0 0 24 24"
            >
              <path strokeLinecap="round" strokeLinejoin="round" d="M19 9l-7 7-7-7" />
            </svg>
          </button>

          {showCancellationInfo && (
            <div className="mt-2 p-2.5 bg-blue-900/10 border border-blue-800/30 rounded text-xs text-blue-200/80 space-y-1">
              <div><span className="text-green-400">Can cancel:</span> Preparing, File extraction (stops after current file)</div>
              <div><span className="text-red-400">Cannot cancel:</span> Verification, Decryption (AES-256), Decompression (zstd)</div>
              <div className="text-blue-300/60 mt-1.5 italic text-[11px]">These are blocking operations due to library limitations. Cancel will take effect at next cancellable stage.</div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
