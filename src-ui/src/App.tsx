import React, { useState } from 'react';
import { Dashboard } from './pages/Dashboard';
import { Sanitizer } from './pages/Sanitizer';
import { Recovery } from './pages/Recovery';
import { Reports } from './pages/Reports';

type Page = 'dash' | 'wipe' | 'carve' | 'reports';

export default function App() {
  const [page, setPage] = useState<Page>('dash');

  return (
    <div className="flex h-screen bg-gray-950 text-gray-200 overflow-hidden font-sans">
      <aside className="w-64 bg-gray-900 border-r border-gray-800 flex flex-col">
        <div className="p-6 border-b border-gray-800">
          <h1 className="text-2xl font-black bg-clip-text text-transparent bg-gradient-to-r from-blue-500 to-red-500">
            SecureForge
          </h1>
          <p className="text-xs text-gray-500 font-mono mt-1">SIH149 Edition</p>
        </div>
        
        <nav className="flex-1 p-4 space-y-2">
          <button 
            onClick={() => setPage('dash')} 
            className={`w-full text-left px-4 py-3 rounded-lg font-medium transition-colors ${page === 'dash' ? 'bg-blue-900/30 text-blue-400 border border-blue-900/50' : 'hover:bg-gray-800 text-gray-400'}`}
          >
            Dashboard
          </button>
          <button 
            onClick={() => setPage('wipe')} 
            className={`w-full text-left px-4 py-3 rounded-lg font-medium transition-colors ${page === 'wipe' ? 'bg-red-900/30 text-red-400 border border-red-900/50' : 'hover:bg-gray-800 text-gray-400'}`}
          >
            Sanitizer (Wipe)
          </button>
          <button 
            onClick={() => setPage('carve')} 
            className={`w-full text-left px-4 py-3 rounded-lg font-medium transition-colors ${page === 'carve' ? 'bg-indigo-900/30 text-indigo-400 border border-indigo-900/50' : 'hover:bg-gray-800 text-gray-400'}`}
          >
            Recovery (Carve)
          </button>
          <button 
            onClick={() => setPage('reports')} 
            className={`w-full text-left px-4 py-3 rounded-lg font-medium transition-colors ${page === 'reports' ? 'bg-green-900/30 text-green-400 border border-green-900/50' : 'hover:bg-gray-800 text-gray-400'}`}
          >
            Audit Reports
          </button>
        </nav>
      </aside>

      <main className="flex-1 overflow-auto bg-gray-950 relative">
        {page === 'dash' && <Dashboard />}
        {page === 'wipe' && <Sanitizer />}
        {page === 'carve' && <Recovery />}
        {page === 'reports' && <Reports />}
      </main>
    </div>
  );
}
