import { useState, useEffect } from 'react';
import { useDrives } from '../hooks/useDrives';
import { DriveCard } from '../components/DriveCard';
import { EntropyHeatmap } from '../components/EntropyHeatmap';
import { DriveInfo } from '../lib/types';
import {
  HardDrive,
  ShieldCheck,
  Zap,
  RefreshCw,
  Cpu,
  Trash2,
  FileSearch,
  FileText,
  Activity,
  Lock,
} from 'lucide-react';

import { EntropyAPI } from '../lib/api';

interface Props {
  onNavigate: (page: 'dash' | 'wipe' | 'shred' | 'carve' | 'reports' | 'expert') => void;
}

export function Dashboard({ onNavigate }: Props) {
  const { drives, loading, error, refresh } = useDrives();
  const [selectedDrive, setSelectedDrive] = useState<DriveInfo | null>(null);
  const [entropyData, setEntropyData] = useState<number[]>([]);

  useEffect(() => {
    const target = selectedDrive || (drives.length > 0 ? drives[0] : null);
    if (target) {
      EntropyAPI.get(target.path, 120)
        .then((data) => {
          if (data && data.length > 0) setEntropyData(data);
        })
        .catch(() => {
          // If device read requires root or image path
        });
    }
  }, [selectedDrive, drives]);

  const formatBytes = (bytes: number) => {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB', 'PB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return `${parseFloat((bytes / Math.pow(k, i)).toFixed(2))} ${sizes[i]}`;
  };

  const totalCapacity = drives.reduce((acc, d) => acc + (d.size_bytes || 0), 0);
  const healthyCount = drives.filter((d) => d.smart_status === 'Healthy').length;

  return (
    <div className="space-y-6 pb-12">
      {/* Top Welcome & Telemetry Banner */}
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-4 glass-panel p-6 rounded-2xl border border-white/10 bg-gradient-to-r from-surface-900/90 via-surface-900/60 to-surface-950/90 relative overflow-hidden">
        <div className="relative z-10">
          <div className="flex items-center gap-2.5 mb-1.5">
            <span className="px-2.5 py-0.5 rounded-full text-[11px] font-bold uppercase tracking-wider bg-cyber-500/20 text-cyber-400 border border-cyber-500/30">
              NIST SP 800-88 R1 Certified
            </span>
            <span className="px-2.5 py-0.5 rounded-full text-[11px] font-mono text-emerald-400 bg-emerald-950/80 border border-emerald-800/50 flex items-center gap-1">
              <span className="w-1.5 h-1.5 rounded-full bg-emerald-400 animate-pulse" /> Live Engine Active
            </span>
          </div>
          <h1 className="text-2xl font-extrabold tracking-tight text-white">
            Forensic Sanitization & Carving Station
          </h1>
          <p className="text-xs text-slate-400 mt-1 max-w-xl leading-relaxed">
            Hardware controller-level NVMe/ATA sanitization, multi-pass in-place overwriting, and deep sector signature recovery with SHA-256 tamper-evident hash chaining.
          </p>
        </div>

        {/* Quick Action Buttons */}
        <div className="flex items-center gap-2.5 relative z-10">
          <button
            onClick={() => onNavigate('wipe')}
            className="flex items-center gap-2 px-4 py-2.5 rounded-xl bg-gradient-to-r from-rose-600 to-rose-500 hover:from-rose-500 hover:to-rose-400 text-white font-bold text-xs uppercase tracking-wider shadow-lg shadow-rose-600/30 transition-all cursor-pointer"
          >
            <Zap className="w-4 h-4" /> Sanitize Drive
          </button>
          <button
            onClick={() => onNavigate('carve')}
            className="flex items-center gap-2 px-4 py-2.5 rounded-xl bg-gradient-to-r from-cyber-600 to-cyber-500 hover:from-cyber-500 hover:to-cyber-400 text-white font-bold text-xs uppercase tracking-wider shadow-lg shadow-cyber-600/30 transition-all cursor-pointer"
          >
            <FileSearch className="w-4 h-4" /> Recover Files
          </button>
        </div>
      </div>

      {/* Metrics Row */}
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
        <div className="glass-panel p-4 rounded-xl border border-white/5 flex items-center gap-3.5">
          <div className="p-3 rounded-xl bg-cyber-950/80 border border-cyber-800/60 text-cyber-400">
            <HardDrive className="w-6 h-6" />
          </div>
          <div>
            <span className="text-xs text-slate-400 block font-medium">Discovered Storage</span>
            <span className="text-lg font-bold font-mono text-white">{drives.length} Block Devices</span>
          </div>
        </div>

        <div className="glass-panel p-4 rounded-xl border border-white/5 flex items-center gap-3.5">
          <div className="p-3 rounded-xl bg-emerald-950/80 border border-emerald-800/60 text-emerald-400">
            <Cpu className="w-6 h-6" />
          </div>
          <div>
            <span className="text-xs text-slate-400 block font-medium">Total Online Capacity</span>
            <span className="text-lg font-bold font-mono text-emerald-400">{formatBytes(totalCapacity)}</span>
          </div>
        </div>

        <div className="glass-panel p-4 rounded-xl border border-white/5 flex items-center gap-3.5">
          <div className="p-3 rounded-xl bg-amber-950/80 border border-amber-800/60 text-amber-400">
            <ShieldCheck className="w-6 h-6" />
          </div>
          <div>
            <span className="text-xs text-slate-400 block font-medium">SMART Health Status</span>
            <span className="text-lg font-bold font-mono text-white">
              {healthyCount} / {drives.length} Verified
            </span>
          </div>
        </div>

        <div className="glass-panel p-4 rounded-xl border border-white/5 flex items-center gap-3.5">
          <div className="p-3 rounded-xl bg-purple-950/80 border border-purple-800/60 text-purple-400">
            <Activity className="w-6 h-6" />
          </div>
          <div>
            <span className="text-xs text-slate-400 block font-medium">Audit Hash Chain</span>
            <span className="text-lg font-bold font-mono text-purple-400">Tamper-Evident</span>
          </div>
        </div>
      </div>

      {/* Main Grid: Storage Devices & Telemetry */}
      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        {/* Drive Explorer (2 Cols) */}
        <div className="lg:col-span-2 space-y-4">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <HardDrive className="w-4 h-4 text-cyber-400" />
              <h2 className="text-sm font-bold uppercase tracking-wider text-slate-200">
                Connected Block Devices
              </h2>
            </div>
            <button
              onClick={refresh}
              disabled={loading}
              className="flex items-center gap-1.5 px-3 py-1 rounded-lg bg-surface-800 hover:bg-surface-700 text-slate-300 text-xs font-medium transition-colors border border-white/5 cursor-pointer disabled:opacity-50"
            >
              <RefreshCw className={`w-3.5 h-3.5 ${loading ? 'animate-spin' : ''}`} />
              Scan Busses
            </button>
          </div>

          {error && (
            <div className="p-4 rounded-xl bg-rose-950/80 border border-rose-800/60 text-rose-300 text-xs">
              Error detecting drives: {error}
            </div>
          )}

          {drives.length === 0 && !loading ? (
            <div className="glass-panel p-12 rounded-xl text-center text-slate-500">
              No physical block devices detected.
            </div>
          ) : (
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              {drives.map((drive) => (
                <DriveCard
                  key={drive.path}
                  drive={drive}
                  selected={selectedDrive?.path === drive.path}
                  onClick={() => setSelectedDrive(drive)}
                />
              ))}
            </div>
          )}
        </div>

        {/* Selected Drive Inspector / Quick Tools (1 Col) */}
        <div className="space-y-4">
          <div className="flex items-center gap-2">
            <Activity className="w-4 h-4 text-cyber-400" />
            <h2 className="text-sm font-bold uppercase tracking-wider text-slate-200">
              Target Device Operations
            </h2>
          </div>

          {selectedDrive ? (
            <div className="glass-panel p-5 rounded-xl border border-cyber-500/30 glow-border space-y-4">
              <div className="flex items-center justify-between pb-3 border-b border-white/5">
                <div>
                  <span className="text-xs text-slate-500 block">Selected Target</span>
                  <span className="text-base font-bold font-mono text-white">{selectedDrive.path}</span>
                </div>
                <span className="px-2.5 py-1 rounded text-xs font-mono font-bold bg-cyber-950 text-cyber-400 border border-cyber-800/60">
                  {selectedDrive.drive_type}
                </span>
              </div>

              <div className="space-y-2 text-xs font-mono">
                <div className="flex justify-between text-slate-300">
                  <span className="text-slate-500 font-sans">Model:</span>
                  <span className="truncate max-w-[180px]">{selectedDrive.model}</span>
                </div>
                <div className="flex justify-between text-slate-300">
                  <span className="text-slate-500 font-sans">Size:</span>
                  <span className="text-cyber-400 font-bold">{formatBytes(selectedDrive.size_bytes)}</span>
                </div>
                <div className="flex justify-between text-slate-300">
                  <span className="text-slate-500 font-sans">Mounts:</span>
                  <span>{selectedDrive.mount_points.length > 0 ? selectedDrive.mount_points.join(', ') : 'Unmounted'}</span>
                </div>
              </div>

              {/* Action Trigger Buttons */}
              <div className="pt-3 border-t border-white/5 space-y-2">
                <button
                  onClick={() => onNavigate('wipe')}
                  className="w-full py-2 px-3 rounded-lg bg-rose-600 hover:bg-rose-500 text-white font-bold text-xs uppercase tracking-wider flex items-center justify-center gap-2 shadow-md shadow-rose-600/20 transition-all cursor-pointer"
                >
                  <Zap className="w-3.5 h-3.5" /> Sanitize / Wipe Target
                </button>
                <button
                  onClick={() => onNavigate('carve')}
                  className="w-full py-2 px-3 rounded-lg bg-surface-800 hover:bg-surface-700 text-slate-200 font-medium text-xs flex items-center justify-center gap-2 border border-white/5 transition-all cursor-pointer"
                >
                  <FileSearch className="w-3.5 h-3.5 text-cyber-400" /> Carve Forensic Artifacts
                </button>
              </div>
            </div>
          ) : (
            <div className="glass-panel p-6 rounded-xl text-center text-xs text-slate-400 space-y-2">
              <HardDrive className="w-8 h-8 mx-auto text-slate-600" />
              <p>Select any storage device from the left panel to inspect partition maps and launch sanitization.</p>
            </div>
          )}

          {/* Quick Hub Navigation Cards */}
          <div className="glass-panel p-4 rounded-xl space-y-2.5">
            <span className="text-xs font-semibold text-slate-400 uppercase tracking-wider block mb-2">
              Forensic Workstation Modules
            </span>

            <div
              onClick={() => onNavigate('shred')}
              className="p-2.5 rounded-lg bg-surface-950/80 hover:bg-surface-900 border border-white/5 hover:border-amber-500/30 cursor-pointer flex items-center gap-3 transition-colors"
            >
              <div className="p-2 rounded bg-amber-950/80 text-amber-400">
                <Trash2 className="w-4 h-4" />
              </div>
              <div className="flex-1">
                <span className="text-xs font-bold text-white block">File & Folder Shredder</span>
                <span className="text-[11px] text-slate-400">Rename storm & symlink defense</span>
              </div>
            </div>

            <div
              onClick={() => onNavigate('reports')}
              className="p-2.5 rounded-lg bg-surface-950/80 hover:bg-surface-900 border border-white/5 hover:border-cyber-500/30 cursor-pointer flex items-center gap-3 transition-colors"
            >
              <div className="p-2 rounded bg-cyber-950/80 text-cyber-400">
                <FileText className="w-4 h-4" />
              </div>
              <div className="flex-1">
                <span className="text-xs font-bold text-white block">Audit Case Vault</span>
                <span className="text-[11px] text-slate-400">PDF certificates & RFC 3161 tokens</span>
              </div>
            </div>

            <div
              onClick={() => onNavigate('expert')}
              className="p-2.5 rounded-lg bg-surface-950/80 hover:bg-surface-900 border border-white/5 hover:border-purple-500/30 cursor-pointer flex items-center gap-3 transition-colors"
            >
              <div className="p-2 rounded bg-purple-950/80 text-purple-400">
                <Lock className="w-4 h-4" />
              </div>
              <div className="flex-1">
                <span className="text-xs font-bold text-white block">Firmware Security Enclave</span>
                <span className="text-[11px] text-slate-400">NVMe Sanitize / ATA Secure Erase</span>
              </div>
            </div>
          </div>
        </div>
      </div>

      {/* Real-time Sector Entropy Radar */}
      <div>
        <EntropyHeatmap data={entropyData} height={44} title="Live Forensic Entropy Telemetry Monitor" />
      </div>
    </div>
  );
}
