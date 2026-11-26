import { useState, FormEvent } from 'react';
import { Key, ShieldAlert, CheckCircle2, ArrowRight } from 'lucide-react';
import { saveToken, getSavedToken } from '../lib/api';

interface Props {
  isOpen: boolean;
  onSuccess: () => void;
}

export function TokenModal({ isOpen, onSuccess }: Props) {
  const [token, setToken] = useState(getSavedToken());
  const [error, setError] = useState('');

  if (!isOpen) return null;

  const handleSubmit = (e: FormEvent) => {
    e.preventDefault();
    if (!token.trim()) {
      setError('Please provide the authentication token printed in your terminal.');
      return;
    }
    saveToken(token.trim());
    setError('');
    onSuccess();
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/85 backdrop-blur-md animate-fadeIn">
      <div className="glass-panel w-full max-w-lg rounded-2xl border border-cyber-500/40 glow-border p-6 shadow-2xl relative space-y-4">
        <div className="flex items-center gap-3.5">
          <div className="p-3 rounded-xl bg-cyber-950/80 border border-cyber-500/40 text-cyber-400">
            <Key className="w-6 h-6 animate-pulse" />
          </div>
          <div>
            <h3 className="text-lg font-bold text-white tracking-wide">
              Web Workstation Authentication
            </h3>
            <p className="text-xs text-cyber-400/80 font-mono">
              Bearer Token Required for HTTP Access
            </p>
          </div>
        </div>

        <div className="p-3 rounded-xl bg-surface-950/80 border border-white/5 text-xs text-slate-300 space-y-2">
          <div className="flex items-start gap-2 text-slate-300">
            <ShieldAlert className="w-4 h-4 text-amber-400 shrink-0 mt-0.5" />
            <span>
              The SecureForge server is running in hardened server mode. Copy the <strong>Auth Token</strong> displayed in your terminal output and paste it below:
            </span>
          </div>
          <div className="bg-surface-900/90 p-2 rounded-lg font-mono text-[11px] text-slate-400 border border-white/5 select-all">
            ./target/release/secureforge-desktop --server --port 7878<br />
            <span className="text-emerald-400">Auth Token : [copy 32-character hex token]</span>
          </div>
        </div>

        <form onSubmit={handleSubmit} className="space-y-4">
          <div>
            <label className="block text-xs font-semibold text-slate-400 mb-1.5 font-mono">
              Server Bearer Token
            </label>
            <input
              type="text"
              value={token}
              onChange={(e) => {
                setToken(e.target.value);
                setError('');
              }}
              placeholder="e.g. c994d91d1c300c0a7d1aea9e416842d3"
              className="w-full bg-surface-950 border border-cyber-500/40 rounded-xl px-4 py-2.5 text-sm font-mono text-white placeholder-slate-600 focus:outline-none focus:border-cyber-400 focus:ring-1 focus:ring-cyber-400"
              autoFocus
            />
          </div>

          {error && (
            <div className="p-2.5 rounded-lg bg-rose-950/80 border border-rose-800/60 text-rose-300 text-xs font-mono">
              {error}
            </div>
          )}

          <div className="flex items-center justify-end gap-3 pt-2">
            <button
              type="submit"
              disabled={!token.trim()}
              className="px-5 py-2.5 rounded-xl text-xs font-bold uppercase tracking-wider bg-gradient-to-r from-cyber-600 to-cyber-500 hover:from-cyber-500 hover:to-cyber-400 text-white shadow-lg shadow-cyber-600/30 disabled:opacity-40 disabled:cursor-not-allowed transition-all flex items-center gap-2 cursor-pointer"
            >
              <CheckCircle2 className="w-4 h-4" /> Authenticate Session <ArrowRight className="w-3.5 h-3.5" />
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}

export default TokenModal;
