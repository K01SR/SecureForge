import { useState, useEffect } from 'react';
import { AlertTriangle, ShieldAlert, X } from 'lucide-react';

interface Props {
  isOpen: boolean;
  title: string;
  message: string;
  confirmWord: string;
  targetDetails?: {
    target: string;
    method: string;
    isSystemDrive?: boolean;
  };
  onConfirm: () => void;
  onCancel: () => void;
}

export function ConfirmDialog({
  isOpen,
  title,
  message,
  confirmWord,
  targetDetails,
  onConfirm,
  onCancel,
}: Props) {
  const [input, setInput] = useState('');

  useEffect(() => {
    if (isOpen) {
      setInput('');
    }
  }, [isOpen]);

  if (!isOpen) return null;

  const isMatched = input.trim() === confirmWord;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/80 backdrop-blur-md animate-fadeIn">
      <div className="glass-panel w-full max-w-lg rounded-2xl border border-rose-500/30 glow-border-danger p-6 shadow-2xl relative">
        {/* Close Button */}
        <button
          onClick={onCancel}
          className="absolute top-4 right-4 p-1.5 rounded-lg text-slate-400 hover:text-white hover:bg-white/5 transition-colors"
        >
          <X className="w-5 h-5" />
        </button>

        {/* Header with Danger Icon */}
        <div className="flex items-start gap-4">
          <div className="p-3 rounded-xl bg-rose-950/80 border border-rose-800/80 text-rose-400">
            <ShieldAlert className="w-7 h-7 animate-pulse" />
          </div>
          <div>
            <h3 className="text-lg font-bold text-white tracking-wide">{title}</h3>
            <p className="text-xs text-rose-300/80 mt-1 font-mono uppercase tracking-wider font-semibold">
              Irreversible Cryptographic Destruction
            </p>
          </div>
        </div>

        {/* Target Details Box */}
        {targetDetails && (
          <div className="mt-4 p-3.5 rounded-xl bg-surface-950/90 border border-white/10 space-y-2 text-xs font-mono">
            <div className="flex justify-between">
              <span className="text-slate-500 font-sans">Target:</span>
              <span className="text-white font-bold">{targetDetails.target}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-slate-500 font-sans">Erase Method:</span>
              <span className="text-cyber-400 font-semibold">{targetDetails.method}</span>
            </div>
            {targetDetails.isSystemDrive && (
              <div className="mt-2 p-2 rounded bg-rose-900/40 border border-rose-700/60 text-rose-300 font-sans text-xs flex items-center gap-2">
                <AlertTriangle className="w-4 h-4 text-rose-400 shrink-0" />
                <span>WARNING: Target is a detected boot/system drive. System will be inoperable!</span>
              </div>
            )}
          </div>
        )}

        {/* Message */}
        <p className="mt-4 text-xs text-slate-300 leading-relaxed font-sans">{message}</p>

        {/* Confirmation Phrase Input */}
        <div className="mt-5">
          <label className="block text-xs font-semibold text-slate-400 mb-2">
            To proceed, type <span className="font-mono text-rose-400 font-bold bg-rose-950/80 px-1.5 py-0.5 rounded border border-rose-800/50">{confirmWord}</span> below:
          </label>
          <input
            type="text"
            value={input}
            onChange={(e) => setInput(e.target.value)}
            placeholder={`Type ${confirmWord}`}
            autoFocus
            className="w-full bg-surface-950 border border-rose-500/40 rounded-xl px-4 py-2.5 text-sm font-mono text-rose-200 placeholder-slate-600 focus:outline-none focus:border-rose-400 focus:ring-1 focus:ring-rose-400"
          />
        </div>

        {/* Action Buttons */}
        <div className="mt-6 flex items-center justify-end gap-3">
          <button
            onClick={onCancel}
            className="px-4 py-2 rounded-xl text-xs font-semibold text-slate-300 hover:text-white hover:bg-surface-800 border border-white/5 transition-all"
          >
            Cancel Abort
          </button>
          <button
            disabled={!isMatched}
            onClick={onConfirm}
            className={`px-5 py-2 rounded-xl text-xs font-bold uppercase tracking-wider transition-all flex items-center gap-2 ${
              isMatched
                ? 'bg-rose-600 hover:bg-rose-500 text-white shadow-lg shadow-rose-600/30 cursor-pointer'
                : 'bg-surface-800 text-slate-500 cursor-not-allowed border border-white/5'
            }`}
          >
            <ShieldAlert className="w-4 h-4" /> Confirm & Execute
          </button>
        </div>
      </div>
    </div>
  );
}
