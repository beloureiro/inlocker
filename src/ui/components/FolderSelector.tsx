import { useState } from 'react';
import { useBackupStore } from '../../store/useBackupStore';

interface FolderSelectorProps {
  onFolderSelected: (sourcePath: string, destinationPath: string) => void;
}

export function FolderSelector({ onFolderSelected }: FolderSelectorProps) {
  const { selectFolder } = useBackupStore();
  const [sourcePath, setSourcePath] = useState<string>('');
  const [destinationPath, setDestinationPath] = useState<string>('');

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
      onFolderSelected(sourcePath, destinationPath);
      setSourcePath('');
      setDestinationPath('');
    }
  };

  return (
    <div className="bg-gray-900 rounded-lg border border-gray-800 p-6">
      <h2 className="text-lg font-semibold mb-4">Add New Backup</h2>

      {/* Source Folder */}
      <div className="mb-4">
        <label className="block text-sm font-medium text-gray-300 mb-2">
          Source Folder
        </label>
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
      <div className="mb-4">
        <label className="block text-sm font-medium text-gray-300 mb-2">
          Destination Folder
        </label>
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

      {/* Save Button */}
      <button
        onClick={handleSave}
        disabled={!sourcePath || !destinationPath}
        className="w-full px-4 py-2 bg-emerald-600 hover:bg-emerald-500 disabled:bg-gray-700 disabled:cursor-not-allowed rounded font-medium transition-colors shadow-lg disabled:shadow-none"
      >
        Save Backup Configuration
      </button>
    </div>
  );
}
