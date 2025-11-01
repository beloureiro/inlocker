import { ReactNode } from 'react';

interface LayoutProps {
  children: ReactNode;
}

export function Layout({ children }: LayoutProps) {
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
          </div>
        </div>
      </header>

      {/* Main Content */}
      <main className="container mx-auto px-6 py-8">
        {children}
      </main>
    </div>
  );
}
