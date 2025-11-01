import { useState } from 'react';
import { BackupConfig, ScheduleConfig } from '../../store/useBackupStore';

interface BackupConfigModalProps {
  config: BackupConfig;
  onSave: (updatedConfig: BackupConfig) => void;
  onClose: () => void;
}

export function BackupConfigModal({ config, onSave, onClose }: BackupConfigModalProps) {
  const [backupName, setBackupName] = useState<string>(config.name);
  const [backupType, setBackupType] = useState<'full' | 'incremental'>(config.backup_type);
  const [schedulePreset, setSchedulePreset] = useState<string>(
    config.schedule?.preset || 'none'
  );
  const [customCron, setCustomCron] = useState<string>(
    config.schedule?.cron_expression || ''
  );

  const schedulePresets = [
    { value: 'none', label: 'Manual Only', cron: '' },
    { value: 'hourly', label: 'Every Hour', cron: '0 * * * *' },
    { value: 'daily', label: 'Daily at 8 PM', cron: '0 20 * * *' },
    { value: 'weekly', label: 'Weekly (Sunday 8 PM)', cron: '0 20 * * 0' },
    { value: 'monthly', label: 'Monthly (1st at 8 PM)', cron: '0 20 1 * *' },
    { value: 'custom', label: 'Custom Schedule', cron: '' },
  ];

  const handleSave = () => {
    const selectedPreset = schedulePresets.find((p) => p.value === schedulePreset);
    const cronExpression = schedulePreset === 'custom' ? customCron : selectedPreset?.cron || '';

    const scheduleConfig: ScheduleConfig | null =
      schedulePreset === 'none'
        ? null
        : {
            cron_expression: cronExpression,
            preset: schedulePreset as any,
            next_run: null,
            enabled: true,
          };

    const updatedConfig: BackupConfig = {
      ...config,
      name: backupName.trim() || config.name, // Fallback to original if empty
      backup_type: backupType,
      schedule: scheduleConfig,
      updated_at: Date.now(),
    };

    onSave(updatedConfig);
  };

  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
      <div className="bg-gray-900 rounded-lg border border-gray-800 p-6 max-w-lg w-full mx-4 max-h-[90vh] overflow-y-auto">
        <div className="flex items-center justify-between mb-4">
          <h2 className="text-xl font-semibold">Configure Backup</h2>
          <button
            onClick={onClose}
            className="text-gray-400 hover:text-white transition-colors"
          >
            <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>

        <div className="space-y-4">
          {/* Config Name */}
          <div>
            <label className="block text-sm font-medium text-gray-300 mb-2">
              Backup Name
            </label>
            <input
              type="text"
              value={backupName}
              onChange={(e) => setBackupName(e.target.value)}
              className="w-full px-3 py-2 bg-gray-800 border border-gray-700 rounded text-sm text-gray-300 focus:border-emerald-600 focus:outline-none transition-colors"
              placeholder="Enter backup name"
            />
          </div>

          {/* Backup Type */}
          <div>
            <label className="block text-sm font-medium text-gray-300 mb-2">
              Backup Type
            </label>
            <div className="grid grid-cols-2 gap-3">
              <button
                type="button"
                onClick={() => setBackupType('incremental')}
                className={`px-4 py-3 rounded border-2 text-sm font-medium transition-all ${
                  backupType === 'incremental'
                    ? 'border-purple-600 bg-purple-900/30 text-purple-300'
                    : 'border-gray-700 bg-gray-800 text-gray-400 hover:border-gray-600'
                }`}
              >
                <div className="font-semibold">Incremental</div>
                <div className="text-xs mt-1 opacity-80">Only changed files</div>
              </button>
              <button
                type="button"
                onClick={() => setBackupType('full')}
                className={`px-4 py-3 rounded border-2 text-sm font-medium transition-all ${
                  backupType === 'full'
                    ? 'border-blue-600 bg-blue-900/30 text-blue-300'
                    : 'border-gray-700 bg-gray-800 text-gray-400 hover:border-gray-600'
                }`}
              >
                <div className="font-semibold">Full</div>
                <div className="text-xs mt-1 opacity-80">All files</div>
              </button>
            </div>
          </div>

          {/* Schedule */}
          <div>
            <label className="block text-sm font-medium text-gray-300 mb-2">
              Schedule
            </label>
            <select
              value={schedulePreset}
              onChange={(e) => setSchedulePreset(e.target.value)}
              className="w-full px-3 py-2 bg-gray-800 border border-gray-700 rounded text-sm text-gray-300 focus:border-emerald-600 focus:outline-none transition-colors appearance-none cursor-pointer"
              style={{
                backgroundImage: `url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' fill='none' viewBox='0 0 20 20'%3E%3Cpath stroke='%236b7280' stroke-linecap='round' stroke-linejoin='round' stroke-width='1.5' d='M6 8l4 4 4-4'/%3E%3C/svg%3E")`,
                backgroundPosition: 'right 0.5rem center',
                backgroundRepeat: 'no-repeat',
                backgroundSize: '1.5em 1.5em',
                paddingRight: '2.5rem',
              }}
            >
              {schedulePresets.map((preset) => (
                <option key={preset.value} value={preset.value}>
                  {preset.label}
                </option>
              ))}
            </select>
          </div>

          {/* Custom Cron Expression */}
          {schedulePreset === 'custom' && (
            <div>
              <label className="block text-sm font-medium text-gray-300 mb-2">
                Cron Expression
              </label>
              <input
                type="text"
                value={customCron}
                onChange={(e) => setCustomCron(e.target.value)}
                placeholder="0 2 * * *"
                className="w-full px-3 py-2 bg-gray-800 border border-gray-700 rounded text-sm text-gray-300 placeholder-gray-500 focus:border-emerald-600 focus:outline-none transition-colors"
              />
              <p className="text-xs text-gray-400 mt-1">
                Format: minute hour day month weekday (e.g., "0 2 * * *" = daily at 2 AM)
              </p>
            </div>
          )}

          {/* Schedule Info */}
          {schedulePreset !== 'none' && schedulePreset !== 'custom' && (
            <div className="bg-blue-900/20 border border-blue-800 rounded p-3">
              <div className="flex items-start gap-2">
                <svg className="w-5 h-5 text-blue-400 flex-shrink-0 mt-0.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                </svg>
                <div className="text-sm text-blue-300">
                  <strong>Scheduled:</strong>{' '}
                  {schedulePresets.find((p) => p.value === schedulePreset)?.label}
                  <br />
                  <span className="text-xs opacity-80">
                    Cron: {schedulePresets.find((p) => p.value === schedulePreset)?.cron}
                  </span>
                </div>
              </div>
            </div>
          )}
        </div>

        {/* Actions */}
        <div className="flex gap-3 mt-6">
          <button
            onClick={onClose}
            className="flex-1 px-4 py-2 bg-gray-800 hover:bg-gray-700 rounded font-medium transition-colors"
          >
            Cancel
          </button>
          <button
            onClick={handleSave}
            className="flex-1 px-4 py-2 bg-emerald-600 hover:bg-emerald-500 rounded font-medium transition-colors"
          >
            Save Changes
          </button>
        </div>
      </div>
    </div>
  );
}
