import { useEffect, useState } from 'react';
import { listen } from '@tauri-apps/api/event';

interface ProgressData {
  stage: string;
  message: string;
  percentage?: number;
  files_processed?: number;
  total_files?: number;
}

export default function ScheduledBackupProgress() {
  const [progress, setProgress] = useState<ProgressData>({
    stage: 'initializing',
    message: 'Iniciando backup agendado...',
  });

  useEffect(() => {
    // Listen to backup progress events from Rust
    const unlisten = listen<ProgressData>('backup-progress', (event) => {
      setProgress(event.payload);
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  const getProgressPercentage = () => {
    if (progress.percentage !== undefined) {
      return progress.percentage;
    }
    if (progress.files_processed && progress.total_files) {
      return Math.round((progress.files_processed / progress.total_files) * 100);
    }
    return 0;
  };

  const percentage = getProgressPercentage();

  return (
    <div className="min-h-screen bg-gradient-to-br from-blue-50 to-indigo-100 flex items-center justify-center p-8">
      <div className="bg-white rounded-2xl shadow-2xl p-8 max-w-2xl w-full">
        {/* Header */}
        <div className="text-center mb-8">
          <div className="w-20 h-20 bg-blue-500 rounded-full flex items-center justify-center mx-auto mb-4">
            <svg
              className="w-10 h-10 text-white animate-spin"
              fill="none"
              viewBox="0 0 24 24"
            >
              <circle
                className="opacity-25"
                cx="12"
                cy="12"
                r="10"
                stroke="currentColor"
                strokeWidth="4"
              />
              <path
                className="opacity-75"
                fill="currentColor"
                d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
              />
            </svg>
          </div>
          <h1 className="text-3xl font-bold text-gray-800 mb-2">
            Backup Agendado
          </h1>
          <p className="text-gray-600">
            InLocker está executando seu backup automático
          </p>
        </div>

        {/* Progress Bar */}
        <div className="mb-6">
          <div className="flex justify-between text-sm text-gray-600 mb-2">
            <span className="font-medium capitalize">{progress.stage}</span>
            <span className="font-bold">{percentage}%</span>
          </div>
          <div className="w-full bg-gray-200 rounded-full h-4 overflow-hidden">
            <div
              className="bg-gradient-to-r from-blue-500 to-indigo-600 h-full rounded-full transition-all duration-300 ease-out"
              style={{ width: `${percentage}%` }}
            />
          </div>
        </div>

        {/* Status Message */}
        <div className="bg-blue-50 border border-blue-200 rounded-lg p-4 mb-6">
          <p className="text-gray-700 text-center">{progress.message}</p>
        </div>

        {/* File Counter */}
        {progress.files_processed !== undefined && progress.total_files !== undefined && (
          <div className="text-center text-sm text-gray-600">
            <span className="font-semibold">{progress.files_processed.toLocaleString()}</span>
            {' de '}
            <span className="font-semibold">{progress.total_files.toLocaleString()}</span>
            {' arquivos processados'}
          </div>
        )}

        {/* Info */}
        <div className="mt-8 pt-6 border-t border-gray-200">
          <p className="text-xs text-gray-500 text-center">
            Esta janela fechará automaticamente quando o backup for concluído.
            <br />
            Você receberá uma notificação com o resultado.
          </p>
        </div>
      </div>
    </div>
  );
}
