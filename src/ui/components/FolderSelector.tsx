import { useState } from 'react';
import { useBackupStore } from '../../store/useBackupStore';

interface FolderSelectorProps {
  onFolderSelected: (sourcePath: string, destinationPath: string, backupType: 'full' | 'incremental') => void;
}

export function FolderSelector({ onFolderSelected }: FolderSelectorProps) {
  const { selectFolder } = useBackupStore();
  const [sourcePath, setSourcePath] = useState<string>('');
  const [destinationPath, setDestinationPath] = useState<string>('');
  const [backupType, setBackupType] = useState<'full' | 'incremental'>('incremental');

  const handleSelectSource = async () => {
    const folder = await selectFolder();
    if (folder) {
      setSourcePath(folder);
    }
  };

  const handleSelectDestination = async () => {
    const folder = await selectFolder();
    if (folder) {
      setDestinationPath(folder);
    }
  };

  const handleSave = () => {
    if (sourcePath && destinationPath) {
      onFolderSelected(sourcePath, destinationPath, backupType);
      setSourcePath('');
      setDestinationPath('');
      setBackupType('incremental'); // Reset to default
    }
  };

  return (
    <div className="bg-gray-900 rounded-lg border border-gray-800 p-4">
      <h2 className="text-base font-semibold mb-3">Add New Backup</h2>

      {/* Source Folder */}
      <div className="mb-3">
        <div className="flex gap-2">
          <input
            type="text"
            value={sourcePath}
            readOnly
            placeholder="Select folder to backup..."
            className="flex-1 bg-gray-800 border border-gray-700 rounded px-3 py-2 text-sm text-gray-300 placeholder-gray-500"
          />
          <button
            onClick={handleSelectSource}
            className="px-4 py-2 bg-emerald-700 hover:bg-emerald-600 rounded text-sm font-medium transition-colors"
          >
            Browse
          </button>
        </div>
      </div>

      {/* Destination Folder */}
      <div className="mb-3">
        <div className="flex gap-2">
          <input
            type="text"
            value={destinationPath}
            readOnly
            placeholder="Select where to save backups..."
            className="flex-1 bg-gray-800 border border-gray-700 rounded px-3 py-2 text-sm text-gray-300 placeholder-gray-500"
          />
          <button
            onClick={handleSelectDestination}
            className="px-4 py-2 bg-emerald-700 hover:bg-emerald-600 rounded text-sm font-medium transition-colors"
          >
            Browse
          </button>
        </div>
      </div>

      {/* Backup Type */}
      <div className="mb-3">
        <div className="grid grid-cols-2 gap-2">
          <button
            type="button"
            onClick={() => setBackupType('incremental')}
            className={`px-3 py-2 rounded border-2 text-sm transition-all ${
              backupType === 'incremental'
                ? 'border-purple-600 bg-purple-900/30 text-purple-300'
                : 'border-gray-700 bg-gray-800 text-gray-400 hover:border-gray-600'
            }`}
          >
            <div className="font-medium">Incremental</div>
            <div className="text-xs mt-0.5 opacity-75">Only changed files (faster)</div>
          </button>
          <button
            type="button"
            onClick={() => setBackupType('full')}
            className={`px-3 py-2 rounded border-2 text-sm transition-all ${
              backupType === 'full'
                ? 'border-blue-600 bg-blue-900/30 text-blue-300'
                : 'border-gray-700 bg-gray-800 text-gray-400 hover:border-gray-600'
            }`}
          >
            <div className="font-medium">Full</div>
            <div className="text-xs mt-0.5 opacity-75">All files (complete backup)</div>
          </button>
        </div>
      </div>

      {/* Save Button */}
      <button
        onClick={handleSave}
        disabled={!sourcePath || !destinationPath}
        className="w-full px-4 py-2 bg-emerald-600 hover:bg-emerald-500 disabled:bg-gray-700 disabled:cursor-not-allowed rounded text-sm font-medium transition-colors"
      >
        Save Backup Configuration
      </button>
    </div>
  );
}
