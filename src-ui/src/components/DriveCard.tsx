import { DriveInfo } from '../lib/types';
import { HardDrive, Cpu, Usb, ShieldAlert, ShieldCheck, Activity, AlertTriangle } from 'lucide-react';

interface Props {
  drive: DriveInfo;
  onClick?: () => void;
  selected?: boolean;
}

export function DriveCard({ drive, onClick, selected }: Props) {
  const formatBytes = (bytes: number) => {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB', 'PB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return `${parseFloat((bytes / Math.pow(k, i)).toFixed(2))} ${sizes[i]}`;
  };

  const getDriveIcon = () => {
    switch (drive.drive_type) {
      case 'NVMe':
        return <Cpu className="w-6 h-6 text-cyber-400" />;
      case 'SSD':
        return <HardDrive className="w-6 h-6 text-emerald-400" />;
      case 'USB':
        return <Usb className="w-6 h-6 text-amber-400" />;
      default:
        return <HardDrive className="w-6 h-6 text-slate-400" />;
    }
  };

  const getSmartBadge = () => {
    switch (drive.smart_status) {
      case 'Healthy':
        return (
          <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-xs font-medium bg-emerald-950/80 text-emerald-400 border border-emerald-800/60">
            <ShieldCheck className="w-3 h-3" /> Healthy
          </span>
        );
      case 'Warning':
        return (
          <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-xs font-medium bg-amber-950/80 text-amber-400 border border-amber-800/60">
            <AlertTriangle className="w-3 h-3" /> Warning
          </span>
        );
      case 'Critical':
        return (
          <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-xs font-medium bg-rose-950/80 text-rose-400 border border-rose-800/60">
            <ShieldAlert className="w-3 h-3" /> Critical
          </span>
        );
      default:
        return (
          <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-xs font-medium bg-slate-800 text-slate-400 border border-slate-700">
            <Activity className="w-3 h-3" /> SMART N/A
          </span>
        );
    }
  };

  return (
    <div
      onClick={onClick}
      className={`glass-panel-interactive p-4 rounded-xl cursor-pointer relative overflow-hidden group ${
        selected ? 'border-cyber-400/80 ring-2 ring-cyber-400/30 glow-border bg-surface-900/90' : ''
      }`}
    >
      {/* Top row: Icon, Name, Type Badge, SMART */}
      <div className="flex items-start justify-between gap-2">
        <div className="flex items-center gap-3">
          <div className="p-2.5 rounded-lg bg-surface-950/80 border border-white/5 group-hover:border-cyber-500/30 transition-colors">
            {getDriveIcon()}
          </div>
          <div>
            <div className="flex items-center gap-2">
              <span className="font-mono text-base font-bold text-white tracking-wide">{drive.name}</span>
              <span className="text-xs px-2 py-0.5 rounded font-mono bg-cyber-950 text-cyber-400 border border-cyber-800/50">
                {drive.drive_type}
              </span>
            </div>
            <div className="text-xs text-slate-400 font-mono mt-0.5 truncate max-w-[200px]" title={drive.path}>
              {drive.path}
            </div>
          </div>
        </div>
        <div>{getSmartBadge()}</div>
      </div>

      {/* Model & Serial */}
      <div className="mt-3 pt-3 border-t border-white/5 space-y-1 text-xs">
        <div className="flex justify-between text-slate-300">
          <span className="text-slate-500">Model:</span>
          <span className="font-medium truncate max-w-[220px]" title={drive.model}>{drive.model || 'Generic Storage'}</span>
        </div>
        <div className="flex justify-between text-slate-300 font-mono">
          <span className="text-slate-500 font-sans">Serial:</span>
          <span className="text-slate-400 truncate max-w-[220px]">{drive.serial || 'N/A'}</span>
        </div>
      </div>

      {/* Capacity & System Status */}
      <div className="mt-3 pt-2.5 border-t border-white/5 flex items-center justify-between">
        <div>
          <span className="text-xs text-slate-500 block">Capacity</span>
          <span className="text-sm font-bold font-mono text-cyber-400">{formatBytes(drive.size_bytes)}</span>
        </div>
        <div>
          {drive.is_system_drive ? (
            <span className="inline-flex items-center gap-1 text-xs font-medium text-rose-400 bg-rose-950/80 px-2.5 py-1 rounded-md border border-rose-800/60">
              <ShieldAlert className="w-3.5 h-3.5" /> Boot / OS Drive
            </span>
          ) : drive.is_mounted ? (
            <span className="inline-flex items-center gap-1 text-xs font-medium text-amber-400 bg-amber-950/80 px-2 py-0.5 rounded-md border border-amber-800/50">
              Mounted ({drive.mount_points.length})
            </span>
          ) : (
            <span className="inline-flex items-center gap-1 text-xs font-medium text-emerald-400 bg-emerald-950/80 px-2 py-0.5 rounded-md border border-emerald-800/50">
              Available
            </span>
          )}
        </div>
      </div>

      {/* Subtle selection glow bar */}
      {selected && (
        <div className="absolute bottom-0 left-0 right-0 h-1 bg-gradient-to-r from-cyber-500 via-emerald-400 to-cyber-500" />
      )}
    </div>
  );
}
