import { useEffect } from 'react';
import { usePreferencesStore } from '../../store/usePreferencesStore';

interface PreferencesModalProps {
  onClose: () => void;
}

export function PreferencesModal({ onClose }: PreferencesModalProps) {
  const { preferences, loadPreferences, setAutoCloseProgressWindow, isLoading } = usePreferencesStore();

  useEffect(() => {
    loadPreferences();
  }, [loadPreferences]);

  const handleAutoCloseChange = async (e: React.ChangeEvent<HTMLInputElement>) => {
    await setAutoCloseProgressWindow(e.target.checked);
  };

  return (
    <div className="fixed inset-0 bg-black/60 flex items-center justify-center z-50">
      <div className="bg-gray-900 border border-gray-700 rounded-lg w-full max-w-md mx-4 shadow-xl">
        <div className="flex items-center justify-between p-4 border-b border-gray-700">
          <h2 className="text-lg font-semibold">Preferences</h2>
          <button
            onClick={onClose}
            className="text-gray-400 hover:text-white transition-colors"
          >
            <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>

        <div className="p-4 space-y-4">
          <div className="space-y-3">
            <h3 className="text-sm font-medium text-gray-300 uppercase tracking-wide">Scheduled Backup</h3>

            <label className="flex items-start gap-3 cursor-pointer group">
              <input
                type="checkbox"
                checked={preferences.auto_close_progress_window}
                onChange={handleAutoCloseChange}
                disabled={isLoading}
                className="mt-0.5 w-4 h-4 rounded border-gray-600 bg-gray-800 text-emerald-500 focus:ring-emerald-500 focus:ring-offset-0"
              />
              <div className="flex-1">
                <span className="text-sm text-gray-200 group-hover:text-white transition-colors">
                  Auto-close progress window
                </span>
                <p className="text-xs text-gray-500 mt-1">
                  When enabled, the scheduled backup progress window will close automatically
                  after the backup completes. When disabled, you must close it manually.
                </p>
              </div>
            </label>
          </div>
        </div>

        <div className="flex justify-end gap-2 p-4 border-t border-gray-700">
          <button
            onClick={onClose}
            className="px-4 py-2 text-sm font-medium text-gray-300 hover:text-white transition-colors"
          >
            Close
          </button>
        </div>
      </div>
    </div>
  );
}
