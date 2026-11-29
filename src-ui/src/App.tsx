import { useState, useEffect } from 'react';
import { Dashboard } from './pages/Dashboard';
import { Sanitizer } from './pages/Sanitizer';
import { Recovery } from './pages/Recovery';
import { Reports } from './pages/Reports';
import { Expert } from './pages/Expert';
import { Plugins } from './pages/Plugins';
import { useExpertMode } from './hooks/useExpertMode';
import {
  ShieldCheck,
  Zap,
  FileSearch,
  FileText,
  Cpu,
  Blocks,
  LayoutDashboard,
  Lock,
  Unlock,
  Terminal,
  Clock,
} from 'lucide-react';

import { TokenModal } from './components/TokenModal';
import { checkIsTauri, getSavedToken } from './lib/api';

type Page = 'dash' | 'wipe' | 'shred' | 'carve' | 'reports' | 'expert' | 'plugins';

export default function App() {
  const [page, setPage] = useState<Page>('dash');
  const { isExpert } = useExpertMode();
  const [currentTime, setCurrentTime] = useState<string>('');
  const [showTokenModal, setShowTokenModal] = useState<boolean>(false);
  const [sidebarOpen, setSidebarOpen] = useState(false);

  useEffect(() => {
    // Only prompt for token in web browser mode when no token is saved
    if (!checkIsTauri() && !getSavedToken()) {
      setShowTokenModal(true);
    }

    const handleAuthRequired = () => {
      if (!checkIsTauri()) {
        setShowTokenModal(true);
      }
    };

    window.addEventListener('secureforge-auth-required', handleAuthRequired);
    return () => window.removeEventListener('secureforge-auth-required', handleAuthRequired);
  }, []);

  useEffect(() => {
    const updateTime = () => {
      const now = new Date();
      // toTimeString() returns LOCAL time; using toUTCString() components
      // gives true UTC so the " UTC" suffix is accurate.
      const utc = now.toUTCString().split(' ')[4] || '';
      const trimmed = utc && utc.includes(':') ? utc.slice(0, -3) : '';
      setCurrentTime(trimmed ? trimmed + ' UTC' : new Date().toTimeString().split(' ')[0]);
    };
    updateTime();
    const timer = setInterval(updateTime, 1000);
    return () => clearInterval(timer);
  }, []);

  const navItems = [
    {
      id: 'dash' as Page,
      label: 'Command Center',
      icon: <LayoutDashboard className="w-4 h-4" />,
      color: 'text-cyber-400',
      activeBg: 'bg-cyber-500/10 text-cyber-400 border-cyber-500/30',
    },
    {
      id: 'wipe' as Page,
      label: 'Sanitization Studio',
      icon: <Zap className="w-4 h-4" />,
      color: 'text-rose-400',
      activeBg: 'bg-rose-500/10 text-rose-400 border-rose-500/30',
    },
    {
      id: 'carve' as Page,
      label: 'Forensic Carver',
      icon: <FileSearch className="w-4 h-4" />,
      color: 'text-emerald-400',
      activeBg: 'bg-emerald-500/10 text-emerald-400 border-emerald-500/30',
    },
    {
      id: 'reports' as Page,
      label: 'Audit Case Vault',
      icon: <FileText className="w-4 h-4" />,
      color: 'text-purple-400',
      activeBg: 'bg-purple-500/10 text-purple-400 border-purple-500/30',
    },
    {
      id: 'expert' as Page,
      label: 'Firmware Enclave',
      icon: <Cpu className="w-4 h-4" />,
      color: 'text-amber-400',
      activeBg: 'bg-amber-500/10 text-amber-400 border-amber-500/30',
    },
    {
      id: 'plugins' as Page,
      label: 'Signatures & Plugins',
      icon: <Blocks className="w-4 h-4" />,
      color: 'text-blue-400',
      activeBg: 'bg-blue-500/10 text-blue-400 border-blue-500/30',
    },
  ];

  return (
    <div className="flex h-screen bg-surface-950 text-slate-100 overflow-hidden font-sans selection:bg-cyber-500/30">
      {/* Mobile overlay */}
      {sidebarOpen && (
        <div
          className="fixed inset-0 bg-black/70 z-10 lg:hidden"
          onClick={() => setSidebarOpen(false)}
        />
      )}

      {/* Cyber Sidebar — collapsible on small screens */}
      <aside className={`w-64 bg-surface-900/95 border-r border-white/5 flex flex-col justify-between shrink-0 relative z-20 transition-transform duration-200 fixed lg:static inset-y-0 left-0 ${sidebarOpen ? 'translate-x-0' : '-translate-x-full lg:translate-x-0'}`}>
        <div>
          {/* Logo & Brand */}
          <div className="p-5 border-b border-white/5 flex items-center gap-3">
            <div className="p-2.5 rounded-xl bg-gradient-to-br from-cyber-500 to-rose-600 text-white shadow-lg shadow-cyber-500/20">
              <ShieldCheck className="w-6 h-6" />
            </div>
            <div>
              <div className="flex items-center gap-1.5">
                <span className="font-extrabold text-base tracking-tight text-white font-mono">
                  Secure<span className="text-cyber-400">Forge</span>
                </span>
                <span className="px-1.5 py-0.2 rounded text-[9px] font-mono font-bold bg-white/10 text-cyber-300">
                  v0.1.0
                </span>
              </div>
              <span className="text-[10px] text-slate-400 font-mono tracking-wider uppercase block">
                Sanitize • Recover • Certify
              </span>
            </div>
          </div>

          {/* Navigation Links */}
          <nav className="p-3 space-y-1.5">
            {navItems.map((item) => {
              const isActive = page === item.id || (page === 'shred' && item.id === 'wipe');
              return (
                <button
                  key={item.id}
                  onClick={() => {
                    setPage(item.id);
                    setSidebarOpen(false);
                  }}
                  className={`w-full flex items-center justify-between px-3.5 py-2.5 rounded-xl text-xs font-semibold transition-all cursor-pointer border ${
                    isActive
                      ? `${item.activeBg} font-bold shadow-md shadow-black/40`
                      : 'text-slate-400 hover:text-slate-200 hover:bg-white/[0.03] border-transparent'
                  }`}
                >
                  <div className="flex items-center gap-2.5">
                    <span className={isActive ? item.color : 'text-slate-400'}>{item.icon}</span>
                    <span>{item.label}</span>
                  </div>
                  {isActive && <div className="w-1.5 h-1.5 rounded-full bg-current animate-pulse" />}
                </button>
              );
            })}
          </nav>
        </div>

        {/* Sidebar Footer System Status */}
        <div className="p-4 border-t border-white/5 bg-surface-950/60 space-y-2 text-[11px] font-mono">
          <div className="flex items-center justify-between text-slate-400">
            <span className="flex items-center gap-1.5">
              <Terminal className="w-3.5 h-3.5 text-cyber-400" /> Kernel Engine
            </span>
            <span className="text-emerald-400 font-bold">ONLINE</span>
          </div>

          <div className="flex items-center justify-between text-slate-400">
            <span className="flex items-center gap-1.5">
              {isExpert ? <Unlock className="w-3.5 h-3.5 text-amber-400" /> : <Lock className="w-3.5 h-3.5 text-slate-500" />}
              Expert Enclave
            </span>
            <span className={isExpert ? 'text-amber-400 font-bold' : 'text-slate-500'}>
              {isExpert ? 'UNLOCKED' : 'LOCKED'}
            </span>
          </div>
        </div>
      </aside>

      {/* Main App Container */}
      <div className="flex-1 flex flex-col overflow-hidden relative">
        {/* Top Operational Header */}
        <header className="h-14 bg-surface-900/80 backdrop-blur-md border-b border-white/5 px-4 sm:px-6 flex items-center justify-between shrink-0 z-10">
          <div className="flex items-center gap-3">
            <button
              onClick={() => setSidebarOpen((o) => !o)}
              className="lg:hidden p-2 rounded-lg text-slate-300 hover:text-white hover:bg-white/5 border border-white/5 transition-colors cursor-pointer"
              aria-label="Toggle navigation"
            >
              <span className="block w-5 h-0.5 bg-current mb-1" />
              <span className="block w-5 h-0.5 bg-current mb-1" />
              <span className="block w-5 h-0.5 bg-current" />
            </button>
            <span className="text-xs font-bold text-slate-400 uppercase tracking-wider font-mono">
              Module:
            </span>
            <span className="text-xs font-mono font-bold text-white px-2.5 py-1 rounded-md bg-surface-950 border border-white/10">
              {page === 'dash'
                ? 'SYSTEM_TELEMETRY'
                : page === 'wipe' || page === 'shred'
                ? 'DATA_SANITIZATION'
                : page === 'carve'
                ? 'FORENSIC_CARVER'
                : page === 'reports'
                ? 'AUDIT_CASE_VAULT'
                : page === 'expert'
                ? 'FIRMWARE_SECURITY'
                : 'PLUGIN_MANAGER'}
            </span>
          </div>

          <div className="flex items-center gap-4 text-xs font-mono">
            <div className="hidden sm:flex items-center gap-1.5 text-slate-400 bg-surface-950 px-3 py-1 rounded-lg border border-white/5">
              <Clock className="w-3.5 h-3.5 text-cyber-400" />
              <span>{currentTime}</span>
            </div>

            <div className="flex items-center gap-2 px-3 py-1 rounded-lg bg-emerald-950/80 border border-emerald-800/60 text-emerald-400 text-xs font-semibold">
              <span className="w-2 h-2 rounded-full bg-emerald-400 animate-ping" />
              <span>NIST SP 800-88 R1</span>
            </div>
          </div>
        </header>

        {/* Scrollable Page Body */}
        <main className="flex-1 overflow-y-auto p-6 md:p-8 bg-surface-950/80">
          {page === 'dash' && <Dashboard onNavigate={(p) => setPage(p)} />}
          {(page === 'wipe' || page === 'shred') && <Sanitizer />}
          {page === 'carve' && <Recovery />}
          {page === 'reports' && <Reports />}
          {page === 'expert' && <Expert />}
          {page === 'plugins' && <Plugins />}
        </main>
      </div>

      {/* Web Mode Bearer Authentication Modal */}
      <TokenModal
        isOpen={showTokenModal}
        onSuccess={() => {
          setShowTokenModal(false);
          window.location.reload();
        }}
      />
    </div>
  );
}
