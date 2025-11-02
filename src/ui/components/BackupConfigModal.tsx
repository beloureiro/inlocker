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
  const [backupMode, setBackupMode] = useState<'copy' | 'compressed' | 'encrypted'>(
    config.mode || 'compressed'
  );
  const [password, setPassword] = useState<string>('');
  const [confirmPassword, setConfirmPassword] = useState<string>('');
  const [schedulePreset, setSchedulePreset] = useState<string>(
    config.schedule?.preset || 'none'
  );
  const [customCron, setCustomCron] = useState<string>(
    config.schedule?.cron_expression || ''
  );

  const formatCronExpression = (value: string) => {
    // Smart auto-format cron expression (called onBlur)
    const cleanValue = value.replace(/\s+/g, '');

    if (!cleanValue) {
      return '';
    }

    // Check if it's purely numeric (like "2005" or "014")
    const isNumericOnly = /^[0-9]+$/.test(cleanValue);

    if (isNumericOnly && cleanValue.length >= 2) {
      // Parse following CRON order (as shown in the tree):
      // minute hour day month weekday
      const parts: string[] = [];
      let remaining = cleanValue;

      // Extract MINUTE (0-59, max 2 digits)
      let minute = '';
      if (remaining.length >= 2 && parseInt(remaining.substring(0, 2)) <= 59) {
        minute = remaining.substring(0, 2);
        remaining = remaining.substring(2);
      } else if (remaining.length >= 1) {
        // Single digit minute
        minute = remaining[0];
        remaining = remaining.substring(1);
      }

      // Extract HOUR (0-23, max 2 digits)
      let hour = '';
      if (remaining.length >= 2 && parseInt(remaining.substring(0, 2)) <= 23) {
        hour = remaining.substring(0, 2);
        remaining = remaining.substring(2);
      } else if (remaining.length >= 1 && parseInt(remaining[0]) <= 23) {
        // Single digit hour
        hour = remaining[0];
        remaining = remaining.substring(1);
      }

      // If we got at least minute and hour, create valid cron
      if (minute && hour) {
        parts.push(minute, hour, '*', '*', '*');
        return parts.join(' ');
      }

      // If only minute provided, keep original
      return value;
    }

    // For non-numeric, return as-is
    return value;
  };

  const schedulePresets = [
    { value: 'none', label: 'Manual Only', cron: '' },
    { value: 'hourly', label: 'Every Hour', cron: '0 * * * *' },
    { value: 'daily', label: 'Daily at 2 PM', cron: '0 14 * * *' },
    { value: 'weekly', label: 'Weekly (Sunday 2 PM)', cron: '0 14 * * 0' },
    { value: 'monthly', label: 'Monthly (1st at 2 PM)', cron: '0 14 1 * *' },
    { value: 'custom', label: 'Custom Schedule', cron: '' },
  ];

  const handleSave = () => {
    // Validate password for encrypted mode
    if (backupMode === 'encrypted') {
      if (!password) {
        alert('Password is required for encrypted backups');
        return;
      }
      if (password !== confirmPassword) {
        alert('Passwords do not match');
        return;
      }
      if (password.length < 8) {
        alert('Password must be at least 8 characters');
        return;
      }
    }

    const selectedPreset = schedulePresets.find((p) => p.value === schedulePreset);
    let cronExpression = schedulePreset === 'custom' ? customCron : selectedPreset?.cron || '';

    // Auto-fill custom cron with asterisks to ensure 5 fields
    if (schedulePreset === 'custom' && cronExpression) {
      const parts = cronExpression.trim().split(/\s+/);
      while (parts.length < 5) {
        parts.push('*');
      }
      cronExpression = parts.slice(0, 5).join(' ');
    }

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
      mode: backupMode,
      encryption_password: backupMode === 'encrypted' ? password : undefined,
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

          {/* Backup Mode */}
          <div>
            <label className="block text-sm font-medium text-gray-300 mb-2">
              Backup Mode
            </label>
            <div className="grid grid-cols-3 gap-2">
              <button
                type="button"
                onClick={() => setBackupMode('copy')}
                className={`px-3 py-2.5 rounded border-2 text-xs font-medium transition-all ${
                  backupMode === 'copy'
                    ? 'border-gray-500 bg-gray-700/40 text-gray-200'
                    : 'border-gray-700 bg-gray-800 text-gray-400 hover:border-gray-600'
                }`}
              >
                <div className="font-semibold">Copy</div>
                <div className="text-[10px] mt-0.5 opacity-70">No compression</div>
              </button>
              <button
                type="button"
                onClick={() => setBackupMode('compressed')}
                className={`px-3 py-2.5 rounded border-2 text-xs font-medium transition-all ${
                  backupMode === 'compressed'
                    ? 'border-emerald-600 bg-emerald-900/30 text-emerald-300'
                    : 'border-gray-700 bg-gray-800 text-gray-400 hover:border-gray-600'
                }`}
              >
                <div className="font-semibold">Compressed</div>
                <div className="text-[10px] mt-0.5 opacity-70">Default</div>
              </button>
              <button
                type="button"
                onClick={() => setBackupMode('encrypted')}
                className={`px-3 py-2.5 rounded border-2 text-xs font-medium transition-all ${
                  backupMode === 'encrypted'
                    ? 'border-amber-600 bg-amber-900/30 text-amber-300'
                    : 'border-gray-700 bg-gray-800 text-gray-400 hover:border-gray-600'
                }`}
              >
                <div className="font-semibold flex items-center justify-center gap-1">
                  <svg className="w-3 h-3" fill="none" stroke="currentColor" strokeWidth="2" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" d="M16.5 10.5V6.75a4.5 4.5 0 1 0-9 0v3.75m-.75 11.25h10.5a2.25 2.25 0 0 0 2.25-2.25v-6.75a2.25 2.25 0 0 0-2.25-2.25H6.75a2.25 2.25 0 0 0-2.25 2.25v6.75a2.25 2.25 0 0 0 2.25 2.25Z" />
                  </svg>
                  Encrypted
                </div>
                <div className="text-[10px] mt-0.5 opacity-70">AES-256</div>
              </button>
            </div>
          </div>

          {/* Password fields (only for encrypted mode) */}
          {backupMode === 'encrypted' && (
            <div className="space-y-3 bg-amber-900/10 border border-amber-800/30 rounded p-3">
              <div>
                <label className="block text-xs font-medium text-amber-300 mb-1.5">
                  Encryption Password
                </label>
                <input
                  type="password"
                  value={password}
                  onChange={(e) => setPassword(e.target.value)}
                  className="w-full px-3 py-2 bg-gray-800 border border-gray-700 rounded text-sm text-gray-300 focus:border-amber-600 focus:outline-none transition-colors"
                  placeholder="Enter password (min. 8 characters)"
                />
              </div>
              <div>
                <label className="block text-xs font-medium text-amber-300 mb-1.5">
                  Confirm Password
                </label>
                <input
                  type="password"
                  value={confirmPassword}
                  onChange={(e) => setConfirmPassword(e.target.value)}
                  className="w-full px-3 py-2 bg-gray-800 border border-gray-700 rounded text-sm text-gray-300 focus:border-amber-600 focus:outline-none transition-colors"
                  placeholder="Re-enter password"
                />
              </div>
              <div className="flex items-start gap-2 text-xs text-amber-300/80">
                <svg className="w-4 h-4 flex-shrink-0 mt-0.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                </svg>
                <span>Password is never saved. You'll need to enter it for each encrypted backup.</span>
              </div>
            </div>
          )}

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
                onBlur={(e) => setCustomCron(formatCronExpression(e.target.value))}
                placeholder="Type: 0014 (min=00, hour=14) or 0 14 * * *"
                className="w-full px-3 py-2 bg-gray-800 border border-gray-700 rounded text-sm text-gray-300 placeholder-gray-500 focus:border-emerald-600 focus:outline-none transition-colors"
              />
              <div className="mt-2 p-2.5 bg-gray-900/50 border border-gray-700 rounded text-xs space-y-1.5">
                <div className="text-gray-400">
                  <strong>Format:</strong> minute hour day month weekday
                </div>
                <pre className="text-gray-300 font-mono text-xs leading-snug overflow-x-auto">
{`  0    14   *   *    *
  │    │    │   │    └─ week (0-6, 0=Sun, *=any)
  │    │    │   └────── month (1-12, *=any)
  │    │    └────────── day (1-31, *=any)
  │    └─────────────── hour (0-23, 14=2PM)
  └──────────────────── minute (0-59)`}
                </pre>
                <div className="text-yellow-400 bg-yellow-900/20 border border-yellow-800/50 rounded px-2 py-1">
                  <strong>*</strong> = "any" or "every" value
                </div>
                <div className="text-gray-400 space-y-0.5">
                  <div className="font-semibold">Examples:</div>
                  <div className="font-mono text-emerald-400">"0 14 * * *" <span className="text-gray-500">→ Daily 2PM</span></div>
                  <div className="font-mono text-emerald-400">"30 9 * * 1" <span className="text-gray-500">→ Mon 9:30AM</span></div>
                  <div className="font-mono text-emerald-400">"0 0 1 * *" <span className="text-gray-500">→ 1st/month midnight</span></div>
                </div>
              </div>
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
