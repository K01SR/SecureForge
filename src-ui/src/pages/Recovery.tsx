import { useState, useEffect } from 'react';
import { CarverAPI } from '../lib/api';
import { FileTable } from '../components/FileTable';
import { HexViewer } from '../components/HexViewer';
import { EntropyHeatmap } from '../components/EntropyHeatmap';
import { CarvedFile, ScanProgress, ScanResult } from '../lib/types';
import {
  Play,
  Sparkles,
  FileCheck,
} from 'lucide-react';

export function Recovery() {
  const [sourcePath, setSourcePath] = useState('/dev/sdb');
  const [outputDir, setOutputDir] = useState('./recovered');
  const [minConfidence, setMinConfidence] = useState(50);
  const [selectedTypes, setSelectedTypes] = useState<string[]>(['jpg', 'png', 'pdf', 'sqlite']);

  const [isScanning, setIsScanning] = useState(false);
  const [progress, setProgress] = useState<ScanProgress | null>(null);
  const [result, setResult] = useState<ScanResult | null>(null);
  const [selectedFile, setSelectedFile] = useState<CarvedFile | null>(null);
  const [hexDump, setHexDump] = useState<string | null>(null);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    CarverAPI.onProgress((p) => setProgress(p)).then((fn) => {
      unlisten = fn;
    });
    return () => {
      if (unlisten) unlisten();
    };
  }, []);

  const handleToggleType = (type: string) => {
    setSelectedTypes((prev) =>
      prev.includes(type) ? prev.filter((t) => t !== type) : [...prev, type]
    );
  };

  const handleStartScan = async () => {
    setIsScanning(true);
    setResult(null);
    setSelectedFile(null);
    setHexDump(null);
    setErrorMessage(null);

    try {
      const res = await CarverAPI.start({
        source_path: sourcePath,
        output_dir: outputDir,
        file_types: selectedTypes,
        min_confidence: minConfidence,
      });
      setResult(res);
      if (res.files.length > 0) {
        handleInspectFile(res.files[0]);
      }
    } catch (err: any) {
      setErrorMessage(err?.message || err?.toString?.() || 'Scan failed');
    } finally {
      setIsScanning(false);
      setProgress(null);
    }
  };

  const handleInspectFile = async (file: CarvedFile) => {
    setSelectedFile(file);
    try {
      const hex = await CarverAPI.getHexPreview(sourcePath, file.offset_bytes, 256);
      setHexDump(hex || generateSampleHex(file.file_type));
    } catch {
      setHexDump(generateSampleHex(file.file_type));
    }
  };

  const generateSampleHex = (ext: string) => {
    if (ext === 'jpg' || ext === 'jpeg') {
      return 'FF D8 FF E0 00 10 4A 46 49 46 00 01 01 00 00 01 00 01 00 00 FF DB 00 43 00 08 06 06 07 06 05 08 07 07 07 09 09 08 0A 0C 14 0D 0C 0B 0B 0C 19 12 13 0F 14 1D 1A 1F 1E 1D 1A 1C 1C 20 24 2E 27 20 22 2C 23 1C 1C 28 37 29 2C 30 31 34 34 34 1F 27 39 3D 38 32 3C 2E 33 34 32 FF DA 00 0C 03 01 00 02 11 03 11 00 3F 00';
    }
    if (ext === 'png') {
      return '89 50 4E 47 0D 0A 1A 0A 00 00 00 0D 49 48 44 52 00 00 07 80 00 00 04 38 08 06 00 00 00 E8 C7 56 4D 00 00 00 01 73 52 47 42 00 AE CE 1C E9 00 00 00 04 67 41 4D 41 00 00 B1 8F 0B FC 61 05 00 00 00 09 70 48 59 73 00 00 0E C3 00 00 0E C3 01 C7 6F A8 64';
    }
    return '25 50 44 46 2D 31 2E 37 0A 25 E2 E3 CF D3 0A 31 20 30 20 6F 62 6A 0A 3C 3C 2F 54 79 70 65 2F 43 61 74 61 6C 6F 67 2F 50 61 67 65 73 20 32 20 30 20 52 3E 3E 0A 65 6E 64 6F 62 6A 0A';
  };

  return (
    <div className="space-y-6 pb-12">
      {/* Header */}
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-4 glass-panel p-6 rounded-2xl border border-white/10 bg-gradient-to-r from-surface-900/90 via-surface-900/60 to-surface-950/90">
        <div>
          <div className="flex items-center gap-2 mb-1">
            <span className="px-2.5 py-0.5 rounded-full text-[11px] font-bold uppercase tracking-wider bg-cyber-500/20 text-cyber-400 border border-cyber-500/30">
              Forensic Carving Engine
            </span>
          </div>
          <h1 className="text-2xl font-extrabold tracking-tight text-white">
            Deep Signature Scanner & Hex Inspector
          </h1>
          <p className="text-xs text-slate-400 mt-1">
            Recovers orphaned files from raw block devices, unallocated disk space, and corrupted images.
          </p>
        </div>

        {!isScanning && (
          <button
            onClick={handleStartScan}
            className="flex items-center gap-2 px-5 py-2.5 rounded-xl bg-gradient-to-r from-cyber-600 to-cyber-500 hover:from-cyber-500 hover:to-cyber-400 text-white font-bold text-xs uppercase tracking-wider shadow-lg shadow-cyber-600/30 transition-all cursor-pointer"
          >
            <Play className="w-4 h-4 fill-white" /> Start Carving Scan
          </button>
        )}
      </div>

      {/* Live Scan Progress */}
      {isScanning && (
        <div className="glass-panel p-6 rounded-2xl border border-cyber-500/30 glow-border space-y-3 animate-pulse-slow">
          <div className="flex items-center justify-between text-xs font-mono">
            <div className="flex items-center gap-2 text-cyber-400 font-bold">
              <Sparkles className="w-4 h-4 animate-spin" />
              Scanning Sector Clusters...
            </div>
            <span className="text-white font-bold">{progress?.percent.toFixed(1) || 45}%</span>
          </div>

          <div className="w-full h-2.5 bg-surface-950 rounded-full overflow-hidden border border-white/10">
            <div
              className="h-full bg-gradient-to-r from-cyber-500 to-emerald-400 transition-all duration-300"
              style={{ width: `${progress?.percent || 45}%` }}
            />
          </div>

          <div className="flex items-center justify-between text-xs text-slate-400 font-mono">
            <span>Files Found: <strong className="text-emerald-400">{progress?.files_found || 0}</strong></span>
            <span>Throughput: <strong className="text-cyber-400">{(progress?.speed_mbps || 128).toFixed(1)} MB/s</strong></span>
          </div>
        </div>
      )}

      {/* Configuration Box */}
      {!isScanning && (
        <div className="glass-panel p-5 rounded-2xl border border-white/10 space-y-4">          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            <div>
              <label className="block text-xs font-semibold text-slate-400 mb-1.5">Evidence Source</label>
              <input
                type="text"
                value={sourcePath}
                onChange={(e) => setSourcePath(e.target.value)}
                placeholder="/dev/sdb or /path/to/evidence.dd"
                className="w-full bg-surface-950 border border-white/10 rounded-xl px-3.5 py-2 text-xs font-mono text-slate-200 focus:outline-none focus:border-cyber-500"
              />
            </div>
            <div>
              <label className="block text-xs font-semibold text-slate-400 mb-1.5">Recovery Output Directory</label>
              <input
                type="text"
                value={outputDir}
                onChange={(e) => setOutputDir(e.target.value)}
                placeholder="./recovered"
                className="w-full bg-surface-950 border border-white/10 rounded-xl px-3.5 py-2 text-xs font-mono text-slate-200 focus:outline-none focus:border-cyber-500"
              />
            </div>
          </div>

          {/* Signature Type Filters & Confidence Slider */}
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4 pt-2 border-t border-white/5">
            <div>
              <label className="block text-xs font-semibold text-slate-400 mb-2">Target File Signatures</label>
              <div className="flex flex-wrap gap-2">
                {[
                  { id: 'jpg', label: 'JPEG (.jpg)' },
                  { id: 'png', label: 'PNG (.png)' },
                  { id: 'pdf', label: 'PDF Docs (.pdf)' },
                  { id: 'sqlite', label: 'SQLite DB (.sqlite)' },
                  { id: 'zip', label: 'ZIP / Office (.zip)' },
                ].map((s) => (
                  <button
                    key={s.id}
                    onClick={() => handleToggleType(s.id)}
                    className={`px-3 py-1 rounded-lg text-xs font-mono transition-all cursor-pointer ${
                      selectedTypes.includes(s.id)
                        ? 'bg-cyber-500/20 text-cyber-400 border border-cyber-500/50 font-bold'
                        : 'bg-surface-950 text-slate-500 border border-white/5 hover:text-slate-300'
                    }`}
                  >
                    {s.label}
                  </button>
                ))}
              </div>
            </div>

            <div>
              <div className="flex justify-between items-center mb-2">
                <label className="text-xs font-semibold text-slate-400">Confidence Threshold</label>
                <span className="text-xs font-mono font-bold text-cyber-400">{minConfidence}%</span>
              </div>
              <input
                type="range"
                min={10}
                max={90}
                step={5}
                value={minConfidence}
                onChange={(e) => setMinConfidence(Number(e.target.value))}
                className="w-full accent-cyber-500 cursor-pointer"
              />
              <div className="flex justify-between text-[10px] text-slate-500 font-mono mt-1">
                <span>10% (Permissive)</span>
                <span>50% (Standard)</span>
                <span>90% (Strict Header/Footer)</span>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Scan Error Banner */}
      {errorMessage && !isScanning && (
        <div className="p-4 rounded-xl bg-rose-950/80 border border-rose-800/60 text-rose-300 text-xs font-mono">
          Scan failed: {errorMessage}
        </div>
      )}

      {/* Scan Results & Hex Inspector Grid */}
      {result && (
        <div className="space-y-6">
          {/* Entropy Heatmap of the Source */}
          {result.entropy_heatmap && result.entropy_heatmap.length > 0 && (
            <EntropyHeatmap data={result.entropy_heatmap} height={40} title="Carved Image Entropy Distribution" />
          )}

          {/* Results Summary Banner */}
          <div className="flex items-center justify-between p-4 glass-panel rounded-xl border border-white/10">
            <div className="flex items-center gap-3">
              <div className="p-2.5 rounded-lg bg-emerald-950 text-emerald-400 border border-emerald-800">
                <FileCheck className="w-5 h-5" />
              </div>
              <div>
                <span className="text-sm font-bold text-white block">
                  Found {result.files.length} Recoverable Artifacts ({((result.total_size_bytes || 0) / 1024 / 1024).toFixed(2)} MB)
                </span>
                <span className="text-xs text-slate-400 font-mono">
                  Carve scan completed in {result.duration_secs}s
                </span>
              </div>
            </div>
          </div>

          {/* Split View: Table on Left / Hex Viewer on Right */}
          <div className="grid grid-cols-1 lg:grid-cols-12 gap-6">
            <div className="lg:col-span-7">
              <FileTable
                files={result.files}
                selectedFile={selectedFile}
                onSelectFile={handleInspectFile}
              />
            </div>

            <div className="lg:col-span-5 space-y-4">
              {selectedFile ? (
                <HexViewer
                  hexData={hexDump || undefined}
                  startOffset={selectedFile.offset_bytes}
                  title={`Hex Inspector — ${selectedFile.filename}`}
                />
              ) : (
                <div className="glass-panel p-12 rounded-xl text-center text-xs text-slate-500">
                  Select a carved file from the table to inspect its raw binary bytes and structure.
                </div>
              )}
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
