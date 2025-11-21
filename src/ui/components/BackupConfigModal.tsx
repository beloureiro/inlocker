import { useState } from 'react';
import { BackupConfig, ScheduleConfig } from '../../store/useBackupStore';

interface BackupConfigModalProps {
  config: BackupConfig;
  onSave: (updatedConfig: BackupConfig) => void;
  onClose: () => void;
}

export function BackupConfigModal({ config, onSave, onClose }: BackupConfigModalProps) {
  // Parse existing cron expression to initialize time/day values
  const parseCronExpression = (cron: string) => {
    const parts = cron.trim().split(/\s+/);
    if (parts.length >= 5) {
      return {
        minute: parseInt(parts[0]) || 0,
        hour: parseInt(parts[1]) || 14,
        day: parts[2] !== '*' ? parseInt(parts[2]) : 1,
        weekday: parts[4] !== '*' ? parseInt(parts[4]) : 0,
      };
    }
    return { minute: 0, hour: 14, day: 1, weekday: 0 };
  };

  const existingCron = config.schedule?.cron_expression || '';
  const parsedCron = parseCronExpression(existingCron);

  // Log parsed values for debugging
  console.log('📅 Configure Backup Modal Opened');
  console.log('   Existing cron:', existingCron || '(none)');
  console.log('   Parsed values:', parsedCron);
  console.log('   Preset:', config.schedule?.preset || 'none');

  const [backupName, setBackupName] = useState<string>(config.name);
  const [backupType, setBackupType] = useState<'full' | 'incremental'>(config.backup_type);
  const [backupMode, setBackupMode] = useState<'copy' | 'compressed' | 'encrypted'>(
    config.mode || 'compressed'
  );
  const [schedulePreset, setSchedulePreset] = useState<string>(
    config.schedule?.preset || 'none'
  );

  // Simple time/day selectors initialized from existing config
  const [hour, setHour] = useState<number>(parsedCron.hour);
  const [minute, setMinute] = useState<number>(parsedCron.minute);
  const [weekday, setWeekday] = useState<number>(parsedCron.weekday); // 0 = Sunday
  const [monthday, setMonthday] = useState<number>(parsedCron.day); // 1-31

  const handleSave = () => {
    let cronExpression = '';

    // Generate cron expression internally based on simple selections
    switch (schedulePreset) {
      case 'none':
        cronExpression = '';
        break;
      case 'hourly':
        cronExpression = '0 * * * *'; // Every hour at minute 0
        break;
      case 'daily':
        cronExpression = `${minute} ${hour} * * *`; // Daily at specified time
        break;
      case 'weekly':
        cronExpression = `${minute} ${hour} * * ${weekday}`; // Weekly on specified day/time
        break;
      case 'monthly':
        cronExpression = `${minute} ${hour} ${monthday} * *`; // Monthly on specified day/time
        break;
      default:
        cronExpression = '';
    }

    console.log('💾 SAVING CONFIGURATION');
    console.log('   Preset:', schedulePreset);
    console.log('   Hour:', hour, '| Minute:', minute);
    console.log('   Weekday:', weekday, '| Monthday:', monthday);
    console.log('   Generated cron:', cronExpression || '(no schedule)');

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
      name: backupName.trim() || config.name,
      backup_type: backupType,
      mode: backupMode,
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

          {/* Encryption Mode Info */}
          {backupMode === 'encrypted' && (
            <div className="bg-amber-900/10 border border-amber-800/30 rounded p-3">
              <div className="flex items-start gap-2 text-xs text-amber-300">
                <svg className="w-4 h-4 flex-shrink-0 mt-0.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                </svg>
                <div>
                  <div className="font-semibold mb-1">Encrypted Backup</div>
                  <div className="opacity-90">
                    You'll be prompted for a password each time you run this backup.
                    Passwords are never saved for security reasons.
                  </div>
                </div>
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
              <option value="none">Manual Only</option>
              <option value="hourly">Every Hour</option>
              <option value="daily">Daily</option>
              <option value="weekly">Weekly</option>
              <option value="monthly">Monthly</option>
            </select>
          </div>

          {/* Time Picker for Daily/Weekly/Monthly */}
          {(schedulePreset === 'daily' || schedulePreset === 'weekly' || schedulePreset === 'monthly') && (
            <div>
              <label className="block text-sm font-medium text-gray-300 mb-2">
                Time
              </label>
              <div className="grid grid-cols-2 gap-3">
                <div>
                  <label className="block text-xs text-gray-400 mb-1">Hour (0-23)</label>
                  <input
                    type="number"
                    min="0"
                    max="23"
                    value={hour}
                    onChange={(e) => setHour(Math.max(0, Math.min(23, parseInt(e.target.value) || 0)))}
                    className="w-full px-3 py-2 bg-gray-800 border border-gray-700 rounded text-sm text-gray-300 focus:border-emerald-600 focus:outline-none transition-colors"
                  />
                </div>
                <div>
                  <label className="block text-xs text-gray-400 mb-1">Minute (0-59)</label>
                  <input
                    type="number"
                    min="0"
                    max="59"
                    value={minute}
                    onChange={(e) => setMinute(Math.max(0, Math.min(59, parseInt(e.target.value) || 0)))}
                    className="w-full px-3 py-2 bg-gray-800 border border-gray-700 rounded text-sm text-gray-300 focus:border-emerald-600 focus:outline-none transition-colors"
                  />
                </div>
              </div>
            </div>
          )}

          {/* Day Picker for Weekly */}
          {schedulePreset === 'weekly' && (
            <div>
              <label className="block text-sm font-medium text-gray-300 mb-2">
                Day of Week
              </label>
              <select
                value={weekday}
                onChange={(e) => setWeekday(parseInt(e.target.value))}
                className="w-full px-3 py-2 bg-gray-800 border border-gray-700 rounded text-sm text-gray-300 focus:border-emerald-600 focus:outline-none transition-colors"
              >
                <option value="0">Sunday</option>
                <option value="1">Monday</option>
                <option value="2">Tuesday</option>
                <option value="3">Wednesday</option>
                <option value="4">Thursday</option>
                <option value="5">Friday</option>
                <option value="6">Saturday</option>
              </select>
            </div>
          )}

          {/* Day Picker for Monthly */}
          {schedulePreset === 'monthly' && (
            <div>
              <label className="block text-sm font-medium text-gray-300 mb-2">
                Day of Month
              </label>
              <input
                type="number"
                min="1"
                max="31"
                value={monthday}
                onChange={(e) => setMonthday(Math.max(1, Math.min(31, parseInt(e.target.value) || 1)))}
                className="w-full px-3 py-2 bg-gray-800 border border-gray-700 rounded text-sm text-gray-300 focus:border-emerald-600 focus:outline-none transition-colors"
                placeholder="1-31"
              />
            </div>
          )}

          {/* Schedule Summary */}
          {schedulePreset !== 'none' && (() => {
            const now = new Date();
            const currentHour = now.getHours();
            const currentMinute = now.getMinutes();

            let nextRunMessage = '';
            let isTimePassed = false;

            if (schedulePreset === 'hourly') {
              nextRunMessage = 'Next run: Top of the next hour';
            } else if (schedulePreset === 'daily') {
              // Check if time has passed today
              if (currentHour > hour || (currentHour === hour && currentMinute >= minute)) {
                isTimePassed = true;
                nextRunMessage = `Next run: Tomorrow at ${hour.toString().padStart(2, '0')}:${minute.toString().padStart(2, '0')}`;
              } else {
                const minutesUntil = (hour - currentHour) * 60 + (minute - currentMinute);
                const hoursUntil = Math.floor(minutesUntil / 60);
                const minsUntil = minutesUntil % 60;
                nextRunMessage = `Next run: Today at ${hour.toString().padStart(2, '0')}:${minute.toString().padStart(2, '0')} (in ${hoursUntil}h ${minsUntil}m)`;
              }
            } else if (schedulePreset === 'weekly' || schedulePreset === 'monthly') {
              nextRunMessage = `Next run: Check schedule logs`;
            }

            return (
              <div className={`${isTimePassed ? 'bg-yellow-900/20 border-yellow-800' : 'bg-blue-900/20 border-blue-800'} border rounded p-3`}>
                <div className="flex items-start gap-2">
                  <svg className={`w-5 h-5 ${isTimePassed ? 'text-yellow-400' : 'text-blue-400'} flex-shrink-0 mt-0.5`} fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" />
                  </svg>
                  <div className={`text-sm ${isTimePassed ? 'text-yellow-300' : 'text-blue-300'}`}>
                    <div><strong>Summary:</strong>{' '}
                      {schedulePreset === 'hourly' && 'Runs every hour'}
                      {schedulePreset === 'daily' && `Runs daily at ${hour.toString().padStart(2, '0')}:${minute.toString().padStart(2, '0')}`}
                      {schedulePreset === 'weekly' && `Runs every ${['Sunday', 'Monday', 'Tuesday', 'Wednesday', 'Thursday', 'Friday', 'Saturday'][weekday]} at ${hour.toString().padStart(2, '0')}:${minute.toString().padStart(2, '0')}`}
                      {schedulePreset === 'monthly' && `Runs on day ${monthday} of each month at ${hour.toString().padStart(2, '0')}:${minute.toString().padStart(2, '0')}`}
                    </div>
                    <div className="mt-1 text-xs opacity-90">{nextRunMessage}</div>
                    {isTimePassed && (
                      <div className="mt-2 text-xs opacity-75">
                        Note: Time has passed today. Use "Test Now" button for immediate execution.
                      </div>
                    )}
                  </div>
                </div>
              </div>
            );
          })()}
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
