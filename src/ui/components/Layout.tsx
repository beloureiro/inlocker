import { ReactNode, useState } from 'react';
import { PreferencesModal } from './PreferencesModal';

interface LayoutProps {
  children: ReactNode;
}

export function Layout({ children }: LayoutProps) {
  const [showPreferences, setShowPreferences] = useState(false);

  return (
    <div className="min-h-screen bg-gray-950 text-white">
      {/* Header with drag region for titlebar */}
      <header className="border-b border-gray-800 bg-gray-900" data-tauri-drag-region>
        <div className="container mx-auto px-6 py-4 pt-8">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-3">
              {/* Logo with new InLocker icon */}
              <img
                src="/logo.png"
                alt="InLocker Logo"
                className="w-12 h-12 rounded-xl shadow-lg"
              />
              <div>
                <h1 className="text-xl font-bold">InLocker</h1>
                <p className="text-xs text-gray-400">Automatic, compressed, and secure backups</p>
              </div>
            </div>

            {/* Settings button */}
            <button
              onClick={() => setShowPreferences(true)}
              className="p-2 text-gray-400 hover:text-white hover:bg-gray-800 rounded-lg transition-colors"
              title="Preferences"
            >
              <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
              </svg>
            </button>
          </div>
        </div>
      </header>

      {/* Main Content */}
      <main className="container mx-auto px-6 py-8">
        {children}
      </main>

      {/* Preferences Modal */}
      {showPreferences && (
        <PreferencesModal onClose={() => setShowPreferences(false)} />
      )}
    </div>
  );
}
