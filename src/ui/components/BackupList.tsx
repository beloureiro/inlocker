import { useBackupStore, BackupConfig } from '../../store/useBackupStore';

export function BackupList() {
  const { configs, deleteConfig } = useBackupStore();

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

  return (
    <div className="space-y-4">
      <h2 className="text-lg font-semibold">Saved Backups</h2>
      {configs.map((config: BackupConfig) => (
        <div
          key={config.id}
          className="bg-gray-900 rounded-lg border border-gray-800 p-4 hover:border-gray-700 transition-colors"
        >
          <div className="flex items-start justify-between">
            <div className="flex-1">
              <h3 className="font-medium text-white mb-2">{config.name}</h3>
              <div className="text-sm space-y-1">
                <div className="flex items-center gap-2">
                  <span className="text-gray-400">From:</span>
                  <span className="text-gray-300 font-mono text-xs">{config.source_path}</span>
                </div>
                <div className="flex items-center gap-2">
                  <span className="text-gray-400">To:</span>
                  <span className="text-gray-300 font-mono text-xs">{config.destination_path}</span>
                </div>
              </div>
              <div className="flex gap-2 mt-3">
                <span className={`px-2 py-1 rounded text-xs ${config.enabled ? 'bg-emerald-900 text-emerald-300' : 'bg-gray-800 text-gray-400'}`}>
                  {config.enabled ? 'Enabled' : 'Disabled'}
                </span>
                {config.encrypt && (
                  <span className="px-2 py-1 rounded text-xs bg-emerald-900/50 text-emerald-400 flex items-center gap-1">
                    <svg className="w-3 h-3" fill="none" stroke="currentColor" strokeWidth="2" viewBox="0 0 24 24">
                      <path strokeLinecap="round" strokeLinejoin="round" d="M16.5 10.5V6.75a4.5 4.5 0 1 0-9 0v3.75m-.75 11.25h10.5a2.25 2.25 0 0 0 2.25-2.25v-6.75a2.25 2.25 0 0 0-2.25-2.25H6.75a2.25 2.25 0 0 0-2.25 2.25v6.75a2.25 2.25 0 0 0 2.25 2.25Z" />
                    </svg>
                    Encrypted
                  </span>
                )}
              </div>
            </div>
            <button
              onClick={() => handleDelete(config.id)}
              className="px-3 py-1 text-sm text-red-400 hover:text-red-300 hover:bg-red-900/20 rounded transition-colors"
            >
              Delete
            </button>
          </div>
        </div>
      ))}
    </div>
  );
}
