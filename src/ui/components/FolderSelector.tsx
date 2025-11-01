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
    <div className="bg-gray-900 rounded-lg border border-gray-800 p-3">
      <h2 className="text-base font-semibold mb-2">
        Add New Backup <span className="text-xs text-gray-400 font-normal">(Tip: Cmd+Shift+. shows hidden folders)</span>
      </h2>

      <div className="grid grid-cols-2 gap-2 mb-2">
        {/* Source Folder */}
        <div className="flex gap-2">
          <input
            type="text"
            value={sourcePath}
            onChange={(e) => setSourcePath(e.target.value)}
            placeholder="Source folder path..."
            className="flex-1 bg-gray-800 border border-gray-700 rounded px-3 py-2 text-sm text-gray-300 placeholder-gray-500"
            title="Type path or click Browse"
          />
          <button
            onClick={handleSelectSource}
            className="px-4 py-2 bg-emerald-700 hover:bg-emerald-600 rounded text-sm font-medium transition-colors whitespace-nowrap"
            title="Browse for folder"
          >
            Browse
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
          />
          <button
            onClick={handleSelectDestination}
            className="px-4 py-2 bg-emerald-700 hover:bg-emerald-600 rounded text-sm font-medium transition-colors whitespace-nowrap"
            title="Browse for folder"
          >
            Browse
          </button>
        </div>
      </div>

      {/* Backup Type + Save Button */}
      <div className="flex gap-2">
        <button
          type="button"
          onClick={() => setBackupType('incremental')}
          className={`flex-1 px-3 py-2 rounded border-2 text-sm transition-all ${
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
          className={`flex-1 px-3 py-2 rounded border-2 text-sm transition-all ${
            backupType === 'full'
              ? 'border-blue-600 bg-blue-900/30 text-blue-300'
              : 'border-gray-700 bg-gray-800 text-gray-400 hover:border-gray-600'
          }`}
        >
          <div className="font-medium">Full</div>
          <div className="text-xs mt-0.5 opacity-75">All files (complete backup)</div>
        </button>
        <button
          onClick={handleSave}
          disabled={!sourcePath || !destinationPath}
          className="flex-1 px-4 py-2 bg-emerald-600 hover:bg-emerald-500 disabled:bg-gray-700 disabled:cursor-not-allowed rounded text-sm font-medium transition-colors whitespace-nowrap"
        >
          Save Backup Configuration
        </button>
      </div>
    </div>
  );
}
