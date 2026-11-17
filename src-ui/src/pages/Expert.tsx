import { useState, useEffect } from 'react';
import { useDrives } from '../hooks/useDrives';
import { useExpertMode } from '../hooks/useExpertMode';
import { FirmwareAPI } from '../lib/api';
import { ExpertGate } from '../components/ExpertGate';
import { DriveInfo, FirmwareCapabilities } from '../lib/types';
import {
  Lock,
  Unlock,
  Cpu,
  AlertTriangle,
  HardDrive,
} from 'lucide-react';

export function Expert() {
  const { drives } = useDrives();
  const { isExpert, lock } = useExpertMode();
  const [showGate, setShowGate] = useState(!isExpert);
  const [selectedDrive, setSelectedDrive] = useState<DriveInfo | null>(null);
  const [capabilities, setCapabilities] = useState<FirmwareCapabilities | null>(null);
  const [probing, setProbing] = useState(false);

  useEffect(() => {
    if (drives.length > 0 && !selectedDrive) {
      setSelectedDrive(drives[0]);
    }
  }, [drives, selectedDrive]);

  useEffect(() => {
    if (selectedDrive) {
      setProbing(true);
      FirmwareAPI.detect(selectedDrive.path)
        .then((caps) => {
          setCapabilities(caps);
          setProbing(false);
        })
        .catch(() => setProbing(false));
    }
  }, [selectedDrive]);

  return (
    <div className="space-y-6 pb-12">
      {/* Header */}
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-4 glass-panel p-6 rounded-2xl border border-amber-500/30 glow-border bg-gradient-to-r from-surface-900/90 via-surface-900/60 to-surface-950/90">
        <div>
          <div className="flex items-center gap-2 mb-1">
            <span className="px-2.5 py-0.5 rounded-full text-[11px] font-bold uppercase tracking-wider bg-amber-500/20 text-amber-400 border border-amber-500/30 flex items-center gap-1">
              <Lock className="w-3 h-3" /> High Security Hardware Enclave
            </span>
          </div>
          <h1 className="text-2xl font-extrabold tracking-tight text-white">
            Firmware-Level Storage Controller Diagnostic
          </h1>
          <p className="text-xs text-slate-400 mt-1">
            Direct NVMe Sanitize commands, ATA Security Erase primitives, Host Protected Area (HPA) & DCO probing.
          </p>
        </div>

        <div className="flex items-center gap-3">
          {isExpert ? (
            <button
              onClick={lock}
              className="flex items-center gap-2 px-4 py-2 rounded-xl bg-surface-800 hover:bg-surface-700 text-amber-400 border border-amber-500/30 text-xs font-bold uppercase tracking-wider transition-all cursor-pointer"
            >
              <Unlock className="w-4 h-4" /> Lock Enclave
            </button>
          ) : (
            <button
              onClick={() => setShowGate(true)}
              className="flex items-center gap-2 px-4 py-2 rounded-xl bg-amber-600 hover:bg-amber-500 text-white text-xs font-bold uppercase tracking-wider transition-all cursor-pointer shadow-lg shadow-amber-600/30"
            >
              <Lock className="w-4 h-4" /> Authorize Session
            </button>
          )}
        </div>
      </div>

      {/* Main Grid */}
      <div className="grid grid-cols-1 lg:grid-cols-12 gap-6">
        {/* Drive Selector */}
        <div className="lg:col-span-5 space-y-4">
          <h2 className="text-sm font-bold uppercase tracking-wider text-slate-200">
            Select Physical Drive to Probe
          </h2>

          <div className="space-y-3">
            {drives.map((d) => (
              <div
                key={d.path}
                onClick={() => setSelectedDrive(d)}
                className={`p-3.5 rounded-xl glass-panel-interactive cursor-pointer border transition-all ${
                  selectedDrive?.path === d.path
                    ? 'border-amber-500/60 ring-2 ring-amber-500/20 bg-surface-900'
                    : 'border-white/5'
                }`}
              >
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-2.5">
                    <div className="p-2 rounded bg-surface-950 text-amber-400">
                      {d.drive_type === 'NVMe' ? <Cpu className="w-5 h-5" /> : <HardDrive className="w-5 h-5" />}
                    </div>
                    <div>
                      <span className="font-mono font-bold text-white text-sm block">{d.path}</span>
                      <span className="text-xs text-slate-400">{d.model}</span>
                    </div>
                  </div>
                  <span className="text-xs font-mono font-bold px-2 py-0.5 rounded bg-surface-950 text-slate-300 border border-white/5">
                    {d.drive_type}
                  </span>
                </div>
              </div>
            ))}
          </div>
        </div>

        {/* Firmware Capabilities Report */}
        <div className="lg:col-span-7 space-y-4">
          <h2 className="text-sm font-bold uppercase tracking-wider text-slate-200">
            Hardware Controller Telemetry
          </h2>

          {selectedDrive && capabilities ? (
            <div className="glass-panel p-6 rounded-2xl border border-white/10 space-y-5 font-mono text-xs">
              <div className="flex items-center justify-between pb-3 border-b border-white/5 font-sans">
                <div>
                  <span className="text-xs text-slate-500 block">Probing Target</span>
                  <span className="text-base font-bold font-mono text-white">{selectedDrive.path}</span>
                </div>
                <span className="text-xs px-2.5 py-1 rounded font-bold bg-amber-950 text-amber-400 border border-amber-800">
                  Recommended: {capabilities.recommended_method}
                </span>
              </div>

              {/* Status Grid */}
              <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                <div className="p-3.5 rounded-xl bg-surface-950/80 border border-white/5 space-y-1">
                  <span className="text-slate-500 font-sans block">NVMe Sanitize Support</span>
                  <span className={`text-sm font-bold ${capabilities.nvme_sanitize_supported ? 'text-emerald-400' : 'text-slate-500'}`}>
                    {capabilities.nvme_sanitize_supported ? 'SUPPORTED (id-ctrl)' : 'NOT DETECTED'}
                  </span>
                </div>

                <div className="p-3.5 rounded-xl bg-surface-950/80 border border-white/5 space-y-1">
                  <span className="text-slate-500 font-sans block">ATA Security State</span>
                  <span className={`text-sm font-bold ${capabilities.ata_frozen ? 'text-rose-400' : 'text-emerald-400'}`}>
                    {capabilities.ata_frozen ? 'FROZEN (BIOS Locked)' : 'UNLOCKED (Ready)'}
                  </span>
                </div>

                <div className="p-3.5 rounded-xl bg-surface-950/80 border border-white/5 space-y-1">
                  <span className="text-slate-500 font-sans block">Host Protected Area (HPA)</span>
                  <span className={`text-sm font-bold ${capabilities.hpa_enabled ? 'text-amber-400' : 'text-emerald-400'}`}>
                    {capabilities.hpa_enabled ? 'DETECTED (Hidden Sectors)' : 'NONE DETECTED'}
                  </span>
                </div>

                <div className="p-3.5 rounded-xl bg-surface-950/80 border border-white/5 space-y-1">
                  <span className="text-slate-500 font-sans block">Device Config Overlay (DCO)</span>
                  <span className={`text-sm font-bold ${capabilities.dco_enabled ? 'text-amber-400' : 'text-emerald-400'}`}>
                    {capabilities.dco_enabled ? 'ACTIVE (Capacity Masked)' : 'DISABLED'}
                  </span>
                </div>
              </div>

              {/* Warnings List */}
              {capabilities.warnings && capabilities.warnings.length > 0 && (
                <div className="p-4 rounded-xl bg-amber-950/60 border border-amber-800/60 space-y-2 text-amber-300 font-sans text-xs">
                  <div className="flex items-center gap-2 font-bold">
                    <AlertTriangle className="w-4 h-4 text-amber-400" />
                    <span>Hardware Safety Advisories:</span>
                  </div>
                  <ul className="list-disc list-inside space-y-1 text-slate-300 text-[11px]">
                    {capabilities.warnings.map((w, idx) => (
                      <li key={idx}>{w}</li>
                    ))}
                  </ul>
                </div>
              )}
            </div>
          ) : (
            <div className="glass-panel p-12 rounded-xl text-center text-xs text-slate-500">
              {probing ? 'Probing hardware controller via hdparm and nvme-cli...' : 'Select a storage device to probe.'}
            </div>
          )}
        </div>
      </div>

      {showGate && (
        <ExpertGate
          onSuccess={() => setShowGate(false)}
          onCancel={() => setShowGate(false)}
        />
      )}
    </div>
  );
}
