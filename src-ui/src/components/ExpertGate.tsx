import { useState, useEffect, FormEvent } from 'react';
import { useExpertMode } from '../hooks/useExpertMode';
import { Lock, Unlock, Key, Shield, X, AlertTriangle } from 'lucide-react';

interface Props {
  onSuccess: (passphrase?: string) => void;
  onCancel: () => void;
}

export function ExpertGate({ onSuccess, onCancel }: Props) {
  const { isExpert, checkConfigured, setup, verify } = useExpertMode();
  const [isConfigured, setIsConfigured] = useState(true);
  const [pass, setPass] = useState('');
  const [error, setError] = useState('');
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    checkConfigured().then(setIsConfigured);
  }, [checkConfigured]);

  useEffect(() => {
    if (isExpert) {
      const timer = setTimeout(() => onSuccess(pass), 0);
      return () => clearTimeout(timer);
    }
  }, [isExpert, pass, onSuccess]);

  const handleSubmit = async (e: FormEvent) => {
    e.preventDefault();
    if (!pass) return;
    setLoading(true);
    setError('');
    const success = isConfigured ? await verify(pass) : await setup(pass);
    setLoading(false);
    if (success) {
      onSuccess(pass);
    } else {
      setError(isConfigured ? 'Authentication failed: Invalid cryptographic passphrase' : 'Setup failed');
    }
  };

  if (isExpert) {
    return null;
  }

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/80 backdrop-blur-md animate-fadeIn">
      <div className="glass-panel w-full max-w-md rounded-2xl border border-amber-500/30 glow-border p-6 shadow-2xl relative">
        <button
          onClick={onCancel}
          className="absolute top-4 right-4 p-1.5 rounded-lg text-slate-400 hover:text-white hover:bg-white/5 transition-colors"
        >
          <X className="w-5 h-5" />
        </button>

        <div className="flex items-center gap-3.5 mb-4">
          <div className="p-3 rounded-xl bg-amber-950/80 border border-amber-800/80 text-amber-400">
            {isConfigured ? <Lock className="w-6 h-6 animate-pulse" /> : <Key className="w-6 h-6 text-cyber-400" />}
          </div>
          <div>
            <h3 className="text-lg font-bold text-white tracking-wide">
              {isConfigured ? 'Expert Security Gate' : 'Initialize Expert Passphrase'}
            </h3>
            <p className="text-xs text-amber-300/80 font-mono">
              Argon2id Salted Cryptographic Enclave
            </p>
          </div>
        </div>

        <div className="mb-4 p-3 rounded-xl bg-surface-950/80 border border-white/5 text-xs text-slate-300 flex items-start gap-2.5">
          <Shield className="w-4 h-4 text-amber-400 shrink-0 mt-0.5" />
          <span>
            {isConfigured
              ? 'Enter your passphrase to authorize firmware-level hardware erasures (NVMe Sanitize / ATA Secure Erase) and protected operations.'
              : 'Set up an Argon2id salted master passphrase to protect high-privilege firmware wipe commands on this machine.'}
          </span>
        </div>

        <form onSubmit={handleSubmit} className="space-y-4">
          <div>
            <label className="block text-xs font-semibold text-slate-400 mb-1.5">
              {isConfigured ? 'Master Passphrase' : 'New Master Passphrase'}
            </label>
            <input
              type="password"
              value={pass}
              onChange={(e) => setPass(e.target.value)}
              placeholder={isConfigured ? 'Enter security passphrase...' : 'Choose a strong master passphrase...'}
              className="w-full bg-surface-950 border border-amber-500/40 rounded-xl px-4 py-2.5 text-sm font-mono text-white placeholder-slate-600 focus:outline-none focus:border-amber-400 focus:ring-1 focus:ring-amber-400"
              autoFocus
            />
          </div>

          {error && (
            <div className="p-2.5 rounded-lg bg-rose-950/80 border border-rose-800/60 text-rose-300 text-xs flex items-center gap-2">
              <AlertTriangle className="w-4 h-4 text-rose-400 shrink-0" />
              <span>{error}</span>
            </div>
          )}

          <div className="flex items-center justify-end gap-3 pt-2">
            <button
              type="button"
              onClick={onCancel}
              className="px-4 py-2 rounded-xl text-xs font-semibold text-slate-400 hover:text-white hover:bg-surface-800 transition-colors"
            >
              Cancel
            </button>
            <button
              type="submit"
              disabled={loading || !pass}
              className="px-5 py-2 rounded-xl text-xs font-bold uppercase tracking-wider bg-gradient-to-r from-amber-600 to-amber-500 hover:from-amber-500 hover:to-amber-400 text-white shadow-lg shadow-amber-600/30 disabled:opacity-40 disabled:cursor-not-allowed transition-all flex items-center gap-2"
            >
              {loading ? (
                'Verifying...'
              ) : isConfigured ? (
                <>
                  <Unlock className="w-3.5 h-3.5" /> Unlock Session
                </>
              ) : (
                <>
                  <Key className="w-3.5 h-3.5" /> Set & Authorize
                </>
              )}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
