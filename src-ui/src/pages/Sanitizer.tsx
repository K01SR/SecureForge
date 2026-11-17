import { useState, useEffect } from 'react';
import { useDrives } from '../hooks/useDrives';
import { WiperAPI, ShredderAPI, FirmwareAPI } from '../lib/api';
import { DriveCard } from '../components/DriveCard';
import { ProgressRing } from '../components/ProgressRing';
import { ConfirmDialog } from '../components/ConfirmDialog';
import { ExpertGate } from '../components/ExpertGate';
import { DriveInfo, WipeProgress, WipeResult, ShredProgress, ShredResult, FirmwareCapabilities, FirmwareEraseResult } from '../lib/types';
import {
  Zap,
  Trash2,
  Cpu,
  CheckCircle,
  AlertTriangle,
  RotateCcw,
  Flame,
  ArrowRight,
} from 'lucide-react';

export function Sanitizer() {
  const { drives } = useDrives();
  const [activeTab, setActiveTab] = useState<'drive' | 'firmware' | 'shred'>('drive');

  // Drive Wipe States
  const [selectedDrive, setSelectedDrive] = useState<DriveInfo | null>(null);
  const [method, setMethod] = useState('dod3');
  const [verifyPostWipe, setVerifyPostWipe] = useState(true);

  // Shredder States
  const [shredPaths, setShredPaths] = useState('/tmp/sensitive_data');
  const [shredPasses, setShredPasses] = useState(3);
  const [shredRenames, setShredRenames] = useState(8);

  // Firmware Erase States
  const [firmwareCaps, setFirmwareCaps] = useState<FirmwareCapabilities | null>(null);
  const [firmwareMethod, setFirmwareMethod] = useState('auto');
  const [showExpertGate, setShowExpertGate] = useState(false);

  // Execution States
  const [isWiping, setIsWiping] = useState(false);
  const [wipeProgress, setWipeProgress] = useState<WipeProgress | null>(null);
  const [wipeResult, setWipeResult] = useState<WipeResult | null>(null);
  const [shredProgress, setShredProgress] = useState<ShredProgress | null>(null);
  const [shredResult, setShredResult] = useState<ShredResult | null>(null);
  const [firmwareResult, setFirmwareResult] = useState<FirmwareEraseResult | null>(null);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);

  // Confirmation Modal
  const [showConfirm, setShowConfirm] = useState(false);

  // Progress Listeners
  useEffect(() => {
    let unlistenWipe: (() => void) | undefined;
    let unlistenShred: (() => void) | undefined;

    WiperAPI.onProgress((p) => setWipeProgress(p)).then((fn) => {
      unlistenWipe = fn;
    });

    ShredderAPI.onProgress((p) => setShredProgress(p)).then((fn) => {
      unlistenShred = fn;
    });

    return () => {
      if (unlistenWipe) unlistenWipe();
      if (unlistenShred) unlistenShred();
    };
  }, []);

  // Detect Firmware Capabilities when selecting drive in Firmware tab
  useEffect(() => {
    if (selectedDrive && activeTab === 'firmware') {
      FirmwareAPI.detect(selectedDrive.path)
        .then(setFirmwareCaps)
        .catch((e) => console.error('Firmware probe error:', e));
    }
  }, [selectedDrive, activeTab]);

  const handleStartOperation = () => {
    if (activeTab === 'drive' && !selectedDrive) return;
    if (activeTab === 'firmware' && !selectedDrive) return;
    if (activeTab === 'firmware') {
      setShowExpertGate(true);
      return;
    }
    setShowConfirm(true);
  };

  const handleConfirmExecution = async () => {
    setShowConfirm(false);
    setIsWiping(true);
    setErrorMessage(null);
    setWipeResult(null);
    setShredResult(null);
    setFirmwareResult(null);

    try {
      if (activeTab === 'drive' && selectedDrive) {
        setWipeProgress({
          sector_current: 0,
          sector_total: Math.floor(selectedDrive.size_bytes / 512),
          percent: 0,
          speed_mbps: 0,
          eta_seconds: 60,
          phase: 'Initializing Multi-Pass Overwrite...',
        });

        const res = await WiperAPI.start({
          device_path: selectedDrive.path,
          method,
          verify: verifyPostWipe,
        });
        setWipeResult(res);
      } else if (activeTab === 'shred') {
        const paths = shredPaths
          .split('\n')
          .map((p) => p.trim())
          .filter(Boolean);

        const res = await ShredderAPI.shred({
          paths,
          passes: shredPasses,
          renames: shredRenames,
          scrub_slack: false,
        });
        setShredResult(res);
      } else if (activeTab === 'firmware' && selectedDrive) {
        const res = await FirmwareAPI.erase({
          device_path: selectedDrive.path,
          method: firmwareMethod,
        });
        setFirmwareResult(res);
      }
    } catch (err: any) {
      setErrorMessage(err.toString());
    } finally {
      setIsWiping(false);
      setWipeProgress(null);
      setShredProgress(null);
    }
  };

  const resetState = () => {
    setWipeResult(null);
    setShredResult(null);
    setFirmwareResult(null);
    setErrorMessage(null);
    setWipeProgress(null);
    setShredProgress(null);
  };

  return (
    <div className="space-y-6 pb-12">
      {/* Tab Switcher */}
      <div className="flex items-center gap-2 p-1.5 glass-panel rounded-xl w-fit border border-white/10">
        <button
          onClick={() => {
            setActiveTab('drive');
            resetState();
          }}
          className={`flex items-center gap-2 px-4 py-2 rounded-lg text-xs font-bold transition-all cursor-pointer ${
            activeTab === 'drive'
              ? 'bg-gradient-to-r from-rose-600 to-rose-500 text-white shadow-lg shadow-rose-600/30'
              : 'text-slate-400 hover:text-white'
          }`}
        >
          <Zap className="w-4 h-4" /> Block Device Sanitizer
        </button>
        <button
          onClick={() => {
            setActiveTab('firmware');
            resetState();
          }}
          className={`flex items-center gap-2 px-4 py-2 rounded-lg text-xs font-bold transition-all cursor-pointer ${
            activeTab === 'firmware'
              ? 'bg-gradient-to-r from-purple-600 to-purple-500 text-white shadow-lg shadow-purple-600/30'
              : 'text-slate-400 hover:text-white'
          }`}
        >
          <Cpu className="w-4 h-4" /> Firmware Erase (NVMe / ATA)
        </button>
        <button
          onClick={() => {
            setActiveTab('shred');
            resetState();
          }}
          className={`flex items-center gap-2 px-4 py-2 rounded-lg text-xs font-bold transition-all cursor-pointer ${
            activeTab === 'shred'
              ? 'bg-gradient-to-r from-amber-600 to-amber-500 text-white shadow-lg shadow-amber-600/30'
              : 'text-slate-400 hover:text-white'
          }`}
        >
          <Trash2 className="w-4 h-4" /> File & Folder Shredder
        </button>
      </div>

      {/* Execution / Progress Modal Banner */}
      {isWiping && (
        <div className="glass-panel p-8 rounded-2xl border border-rose-500/40 glow-border-danger bg-surface-950/95 text-center space-y-6 animate-pulse-slow">
          <div className="flex items-center justify-center gap-2 text-rose-400 font-mono text-xs uppercase tracking-widest font-bold">
            <Flame className="w-4 h-4 animate-bounce" />
            Active Sanitization in Progress — Do Not Disconnect Power
          </div>

          <ProgressRing
            percent={wipeProgress?.percent || shredProgress?.percent || 50}
            size={190}
            strokeWidth={12}
            color="#ef4444"
            speedMbps={wipeProgress?.speed_mbps}
            etaSeconds={wipeProgress?.eta_seconds}
            phase={wipeProgress?.phase || shredProgress?.current_file || 'Processing Overwrite Passes...'}
          />

          <p className="text-xs text-slate-400 font-mono">
            {wipeProgress
              ? `Sectors: ${wipeProgress.sector_current.toLocaleString()} / ${wipeProgress.sector_total.toLocaleString()}`
              : `Files Shredded: ${shredProgress?.files_done} / ${shredProgress?.files_total}`}
          </p>
        </div>
      )}

      {/* Results View */}
      {(wipeResult || shredResult || firmwareResult || errorMessage) && !isWiping && (
        <div className="glass-panel p-6 rounded-2xl border border-white/10 space-y-4">
          <div className="flex items-center justify-between pb-4 border-b border-white/5">
            <div className="flex items-center gap-3">
              {errorMessage ? (
                <div className="p-3 rounded-xl bg-rose-950/80 border border-rose-800 text-rose-400">
                  <AlertTriangle className="w-6 h-6" />
                </div>
              ) : (
                <div className="p-3 rounded-xl bg-emerald-950/80 border border-emerald-800 text-emerald-400">
                  <CheckCircle className="w-6 h-6" />
                </div>
              )}
              <div>
                <h3 className="text-lg font-bold text-white">
                  {errorMessage ? 'Sanitization Aborted with Errors' : 'Sanitization Completed & Verified'}
                </h3>
                <p className="text-xs text-slate-400 font-mono">
                  {errorMessage ? errorMessage : 'NIST 800-88 Purge Criteria Satisfied — Certificate Ready'}
                </p>
              </div>
            </div>
            <button
              onClick={resetState}
              className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-surface-800 hover:bg-surface-700 text-slate-300 text-xs font-medium border border-white/5 transition-colors cursor-pointer"
            >
              <RotateCcw className="w-3.5 h-3.5" /> Start New Task
            </button>
          </div>

          {wipeResult && (
            <div className="grid grid-cols-2 md:grid-cols-4 gap-4 text-xs font-mono">
              <div className="p-3 rounded-xl bg-surface-950/80 border border-white/5">
                <span className="text-slate-500 font-sans block">Sectors Processed</span>
                <span className="text-sm font-bold text-white">{wipeResult.sectors_wiped.toLocaleString()}</span>
              </div>
              <div className="p-3 rounded-xl bg-surface-950/80 border border-white/5">
                <span className="text-slate-500 font-sans block">Verification Status</span>
                <span className="text-sm font-bold text-emerald-400">
                  {wipeResult.verified ? 'PASSED (Entropy Check)' : 'Unverified'}
                </span>
              </div>
              <div className="p-3 rounded-xl bg-surface-950/80 border border-white/5">
                <span className="text-slate-500 font-sans block">Method Used</span>
                <span className="text-sm font-bold text-cyber-400">{wipeResult.method_used.toUpperCase()}</span>
              </div>
              <div className="p-3 rounded-xl bg-surface-950/80 border border-white/5">
                <span className="text-slate-500 font-sans block">Elapsed Duration</span>
                <span className="text-sm font-bold text-white">{wipeResult.duration_secs}s</span>
              </div>
            </div>
          )}

          {shredResult && (
            <div className="grid grid-cols-3 gap-4 text-xs font-mono">
              <div className="p-3 rounded-xl bg-surface-950/80 border border-white/5">
                <span className="text-slate-500 font-sans block">Files Shredded</span>
                <span className="text-sm font-bold text-emerald-400">{shredResult.total_files}</span>
              </div>
              <div className="p-3 rounded-xl bg-surface-950/80 border border-white/5">
                <span className="text-slate-500 font-sans block">Bytes Overwritten</span>
                <span className="text-sm font-bold text-white">{shredResult.total_bytes.toLocaleString()} B</span>
              </div>
              <div className="p-3 rounded-xl bg-surface-950/80 border border-white/5">
                <span className="text-slate-500 font-sans block">Rename Storm Count</span>
                <span className="text-sm font-bold text-amber-400">{shredRenames} renames/file</span>
              </div>
            </div>
          )}

          {firmwareResult && (
            <div className="p-3 rounded-xl bg-surface-950/80 border border-white/5 space-y-2 text-xs font-mono">
              <div className="flex justify-between">
                <span className="text-slate-500 font-sans">Firmware Method:</span>
                <span className="text-purple-400 font-bold">{firmwareResult.method_used}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-slate-500 font-sans">Controller Duration:</span>
                <span className="text-white">{firmwareResult.duration_secs}s</span>
              </div>
              <div className="mt-2 pt-2 border-t border-white/5">
                <span className="text-slate-500 block mb-1">Controller Output Log:</span>
                <pre className="p-2 rounded bg-black/60 text-slate-300 overflow-x-auto text-[11px]">
                  {firmwareResult.command_output}
                </pre>
              </div>
            </div>
          )}
        </div>
      )}

      {/* Main Configuration Panels */}
      {!isWiping && (
        <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
          {/* Target Selector (2 Cols for drive/firmware, or text area for shredder) */}
          <div className="lg:col-span-2 space-y-4">
            {activeTab !== 'shred' ? (
              <>
                <div className="flex items-center justify-between">
                  <h2 className="text-sm font-bold uppercase tracking-wider text-slate-200">
                    Step 1: Select Target Device
                  </h2>
                  <span className="text-xs text-slate-500 font-mono">
                    {drives.length} drives available
                  </span>
                </div>

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
              </>
            ) : (
              <div className="glass-panel p-5 rounded-2xl border border-white/10 space-y-4">
                <h2 className="text-sm font-bold uppercase tracking-wider text-slate-200">
                  Step 1: Select Files or Folders to Shred
                </h2>
                <p className="text-xs text-slate-400">
                  Enter absolute paths of confidential files or directories to in-place overwrite, rename storm, and delete:
                </p>
                <textarea
                  rows={6}
                  value={shredPaths}
                  onChange={(e) => setShredPaths(e.target.value)}
                  placeholder="/tmp/confidential_case/evidence.pdf"
                  className="w-full bg-surface-950 border border-white/10 rounded-xl p-3.5 text-xs font-mono text-slate-200 placeholder-slate-600 focus:outline-none focus:border-amber-500 leading-relaxed"
                />

                <div className="grid grid-cols-2 gap-4 pt-2 text-xs">
                  <div>
                    <label className="block text-slate-400 font-semibold mb-1">Overwrite Passes</label>
                    <select
                      value={shredPasses}
                      onChange={(e) => setShredPasses(Number(e.target.value))}
                      className="w-full bg-surface-950 border border-white/10 rounded-lg p-2 text-xs text-slate-200 font-mono"
                    >
                      <option value={1}>1-Pass (Zero Overwrite)</option>
                      <option value={3}>3-Pass (DoD 5220.22-M)</option>
                      <option value={7}>7-Pass (DoD ECE)</option>
                      <option value={35}>35-Pass (Peter Gutmann)</option>
                    </select>
                  </div>
                  <div>
                    <label className="block text-slate-400 font-semibold mb-1">Random Rename Storms</label>
                    <select
                      value={shredRenames}
                      onChange={(e) => setShredRenames(Number(e.target.value))}
                      className="w-full bg-surface-950 border border-white/10 rounded-lg p-2 text-xs text-slate-200 font-mono"
                    >
                      <option value={3}>3 Random Renames</option>
                      <option value={8}>8 Random Renames (Standard)</option>
                      <option value={16}>16 Random Renames (High Defense)</option>
                    </select>
                  </div>
                </div>
              </div>
            )}
          </div>

          {/* Sanitization Options & Execution Trigger (1 Col) */}
          <div className="space-y-4">
            <h2 className="text-sm font-bold uppercase tracking-wider text-slate-200">
              Step 2: Sanitization Method
            </h2>

            <div className="glass-panel p-5 rounded-2xl border border-white/10 space-y-4">
              {activeTab === 'drive' && (
                <div className="space-y-3 text-xs">
                  <label className="block text-slate-400 font-semibold">Standard Wipe Algorithms</label>

                  <div className="space-y-2">
                    {[
                      { id: 'dod3', name: 'DoD 5220.22-M (3-Pass)', desc: 'Zeros, Ones, CSPRNG Random + Verify' },
                      { id: 'zero', name: 'Zero-Fill (1-Pass Clear)', desc: 'NIST 800-88 Clear level overwrite' },
                      { id: 'random', name: 'CSPRNG Random (1-Pass)', desc: 'Cryptographically secure pseudorandom fill' },
                      { id: 'dod7', name: 'DoD 5220.22-M ECE (7-Pass)', desc: 'Comprehensive multi-pass military grade' },
                      { id: 'gutmann', name: 'Peter Gutmann (35-Pass)', desc: 'Maximal theoretical magnetic erasure' },
                    ].map((m) => (
                      <label
                        key={m.id}
                        className={`p-3 rounded-xl border flex items-start gap-3 cursor-pointer transition-all ${
                          method === m.id
                            ? 'bg-rose-950/60 border-rose-500/60 text-white'
                            : 'bg-surface-950/60 border-white/5 text-slate-400 hover:border-white/20'
                        }`}
                      >
                        <input
                          type="radio"
                          name="wipe_method"
                          value={m.id}
                          checked={method === m.id}
                          onChange={(e) => setMethod(e.target.value)}
                          className="mt-0.5 accent-rose-500"
                        />
                        <div>
                          <span className="font-bold text-slate-200 block">{m.name}</span>
                          <span className="text-[11px] text-slate-500 leading-tight block mt-0.5">{m.desc}</span>
                        </div>
                      </label>
                    ))}
                  </div>

                  <div className="pt-3 border-t border-white/5 flex items-center justify-between">
                    <span className="text-slate-300 font-medium">Post-Wipe Verification</span>
                    <input
                      type="checkbox"
                      checked={verifyPostWipe}
                      onChange={(e) => setVerifyPostWipe(e.target.checked)}
                      className="w-4 h-4 accent-emerald-500 rounded"
                    />
                  </div>
                </div>
              )}

              {activeTab === 'firmware' && (
                <div className="space-y-3 text-xs">
                  <div className="p-3 rounded-xl bg-purple-950/80 border border-purple-800/60 text-purple-300">
                    <span className="font-bold block mb-1">Hardware-Level Controller Erase</span>
                    <p className="text-[11px] text-slate-400 leading-relaxed">
                      Issues direct NVMe sanitize or ATA Security Erase to the drive firmware, destroying reallocated and hidden sectors.
                    </p>
                  </div>

                  {firmwareCaps && (
                    <div className="p-3 rounded-xl bg-surface-950/80 border border-white/5 space-y-1.5 font-mono text-[11px]">
                      <div className="flex justify-between">
                        <span className="text-slate-500">NVMe Crypto Sanitize:</span>
                        <span className={firmwareCaps.nvme_sanitize_supported ? 'text-emerald-400 font-bold' : 'text-slate-500'}>
                          {firmwareCaps.nvme_sanitize_supported ? 'SUPPORTED' : 'NOT DETECTED'}
                        </span>
                      </div>
                      <div className="flex justify-between">
                        <span className="text-slate-500">ATA Frozen State:</span>
                        <span className={firmwareCaps.ata_frozen ? 'text-rose-400 font-bold' : 'text-emerald-400'}>
                          {firmwareCaps.ata_frozen ? 'FROZEN (Power Cycle Required)' : 'UNLOCKED'}
                        </span>
                      </div>
                    </div>
                  )}

                  <div className="space-y-2">
                    {[
                      { id: 'auto', name: 'Auto-Select Best Method', desc: 'Queries drive capabilities and selects strongest erase' },
                      { id: 'nvme-crypto', name: 'NVMe Cryptographic Erase', desc: 'Instant destruction of drive encryption master key' },
                      { id: 'nvme-block', name: 'NVMe Block Erase', desc: 'Low-level flash cell block erasure' },
                      { id: 'ata-enhanced', name: 'ATA Enhanced Security Erase', desc: 'Covers reallocated and wear-leveled sectors' },
                    ].map((m) => (
                      <label
                        key={m.id}
                        className={`p-3 rounded-xl border flex items-start gap-3 cursor-pointer transition-all ${
                          firmwareMethod === m.id
                            ? 'bg-purple-950/60 border-purple-500/60 text-white'
                            : 'bg-surface-950/60 border-white/5 text-slate-400 hover:border-white/20'
                        }`}
                      >
                        <input
                          type="radio"
                          name="fw_method"
                          value={m.id}
                          checked={firmwareMethod === m.id}
                          onChange={(e) => setFirmwareMethod(e.target.value)}
                          className="mt-0.5 accent-purple-500"
                        />
                        <div>
                          <span className="font-bold text-slate-200 block">{m.name}</span>
                          <span className="text-[11px] text-slate-500 leading-tight block mt-0.5">{m.desc}</span>
                        </div>
                      </label>
                    ))}
                  </div>
                </div>
              )}

              {/* Launch Button */}
              <button
                disabled={activeTab !== 'shred' && !selectedDrive}
                onClick={handleStartOperation}
                className={`w-full py-3 px-4 rounded-xl font-bold text-xs uppercase tracking-wider flex items-center justify-center gap-2 shadow-xl transition-all cursor-pointer ${
                  activeTab === 'shred'
                    ? 'bg-gradient-to-r from-amber-600 to-amber-500 hover:from-amber-500 hover:to-amber-400 text-white shadow-amber-600/30'
                    : activeTab === 'firmware'
                    ? 'bg-gradient-to-r from-purple-600 to-purple-500 hover:from-purple-500 hover:to-purple-400 text-white shadow-purple-600/30'
                    : 'bg-gradient-to-r from-rose-600 to-rose-500 hover:from-rose-500 hover:to-rose-400 text-white shadow-rose-600/30'
                } disabled:opacity-40 disabled:cursor-not-allowed`}
              >
                {activeTab === 'shred' ? (
                  <>
                    <Trash2 className="w-4 h-4" /> Shred Files Permanently
                  </>
                ) : activeTab === 'firmware' ? (
                  <>
                    <Cpu className="w-4 h-4" /> Authorize Firmware Erase
                  </>
                ) : (
                  <>
                    <Zap className="w-4 h-4" /> Execute Multi-Pass Wipe
                  </>
                )}
                <ArrowRight className="w-4 h-4" />
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Confirmation Dialog Modal */}
      <ConfirmDialog
        isOpen={showConfirm}
        title={activeTab === 'shred' ? 'Confirm Permanent File Shredding' : 'Confirm Permanent Drive Erasure'}
        message={
          activeTab === 'shred'
            ? 'All specified files and directory contents will be overwritten with multiple random passes, renamed repeatedly to scrub file system journal traces, and destroyed.'
            : `All sectors on ${selectedDrive?.path} (${selectedDrive?.model}) will be permanently overwritten and wiped according to the selected standard. This action is 100% irreversible.`
        }
        confirmWord="ERASE"
        targetDetails={{
          target: activeTab === 'shred' ? 'Multiple Files' : selectedDrive?.path || '',
          method: activeTab === 'shred' ? `${shredPasses}-Pass Shred` : method.toUpperCase(),
          isSystemDrive: selectedDrive?.is_system_drive,
        }}
        onConfirm={handleConfirmExecution}
        onCancel={() => setShowConfirm(false)}
      />

      {/* Expert Gate for Firmware Methods */}
      {showExpertGate && (
        <ExpertGate
          onSuccess={() => {
            setShowExpertGate(false);
            setShowConfirm(true);
          }}
          onCancel={() => setShowExpertGate(false)}
        />
      )}
    </div>
  );
}
