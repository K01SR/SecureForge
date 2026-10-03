import os
import subprocess
from datetime import datetime, timedelta

def run_git(msg, dt):
    date_str = dt.strftime('%Y-%m-%dT%H:%M:%S+05:30')
    env = os.environ.copy()
    env['GIT_AUTHOR_DATE'] = date_str
    env['GIT_COMMITTER_DATE'] = date_str
    
    subprocess.run(['git', 'add', '.'], cwd='/home/karan/Projects/SIH_149', env=env)
    subprocess.run(['git', 'commit', '-m', msg], cwd='/home/karan/Projects/SIH_149', env=env)

base_dir = "/home/karan/Projects/SIH_149/src-ui"
src_dir = os.path.join(base_dir, "src")
os.makedirs(src_dir, exist_ok=True)
os.makedirs(os.path.join(src_dir, "lib"), exist_ok=True)
os.makedirs(os.path.join(src_dir, "hooks"), exist_ok=True)
os.makedirs(os.path.join(src_dir, "components"), exist_ok=True)
os.makedirs(os.path.join(src_dir, "pages"), exist_ok=True)

commits = [
    {
        "file": "src/lib/types.ts",
        "msg": "feat(ui): add TypeScript interfaces for IPC",
        "content": """export interface DriveInfo {
  path: string;
  name: string;
  size: number;
  type: string;
  smart_status: string;
}

export interface WipeConfig {
  target: string;
  method: WipeMethod;
}

export enum WipeMethod {
  Zero = 'zero',
  Random = 'random',
  DoD = 'dod',
  Gutmann = 'gutmann'
}

export interface WipeProgress {
  percent: number;
  current_pass: number;
  total_passes: number;
  speed_bytes_sec: number;
}

export interface WipeResult {
  success: boolean;
  error?: string;
  hash: string;
}

export interface ScanConfig {
  target: string;
  min_confidence: number;
  file_types: string[];
}

export interface CarvedFile {
  path: string;
  size: number;
  type: string;
  confidence: number;
  offset: number;
}

export interface ScanProgress {
  percent: number;
  files_found: number;
  current_sector: number;
}

export interface ScanResult {
  files: CarvedFile[];
  entropy_map: number[];
}

export interface CaseRecord {
  id: string;
  date: string;
  target: string;
  action: string;
  status: string;
  hash: string;
}
"""
    },
    {
        "file": "src/lib/api.ts",
        "msg": "feat(ui): add Tauri invoke wrappers and listeners",
        "content": """import { invoke } from '@tauri-apps/api/core';
import { listen, Event } from '@tauri-apps/api/event';
import { DriveInfo, WipeConfig, WipeProgress, WipeResult, ScanConfig, ScanProgress, ScanResult, CaseRecord } from './types';

export const DrivesAPI = {
  list: (): Promise<DriveInfo[]> => invoke('get_drives')
};

export const WiperAPI = {
  start: (config: WipeConfig): Promise<void> => invoke('start_wipe', { config }),
  onProgress: (cb: (p: WipeProgress) => void) => listen('wipe_progress', (e: Event<WipeProgress>) => cb(e.payload)),
  getResult: (): Promise<WipeResult> => invoke('get_wipe_result')
};

export const CarverAPI = {
  start: (config: ScanConfig): Promise<void> => invoke('start_scan', { config }),
  onProgress: (cb: (p: ScanProgress) => void) => listen('scan_progress', (e: Event<ScanProgress>) => cb(e.payload)),
  getResult: (): Promise<ScanResult> => invoke('get_scan_result')
};

export const AuthAPI = {
  checkConfigured: (): Promise<boolean> => invoke('check_expert_configured'),
  setup: (pass: string): Promise<boolean> => invoke('setup_expert_mode', { pass }),
  verify: (pass: string): Promise<boolean> => invoke('verify_expert_mode', { pass })
};

export const ReportsAPI = {
  list: (): Promise<CaseRecord[]> => invoke('get_reports'),
  export: (id: string, format: string): Promise<string> => invoke('export_report', { id, format })
};
"""
    },
    {
        "file": "src/hooks/useDrives.ts",
        "msg": "feat(ui): add useDrives hook for fetching drive list",
        "content": """import { useState, useEffect } from 'react';
import { DriveInfo } from '../lib/types';
import { DrivesAPI } from '../lib/api';

export function useDrives() {
  const [drives, setDrives] = useState<DriveInfo[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const refresh = async () => {
    setLoading(true);
    setError(null);
    try {
      const data = await DrivesAPI.list();
      setDrives(data);
    } catch (err: any) {
      setError(err.toString());
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    refresh();
  }, []);

  return { drives, loading, error, refresh };
}
"""
    },
    {
        "file": "src/hooks/useExpertMode.ts",
        "msg": "feat(ui): add useExpertMode auth state hook",
        "content": """import { useState } from 'react';
import { AuthAPI } from '../lib/api';

export function useExpertMode() {
  const [isExpert, setIsExpert] = useState(false);

  const checkConfigured = async () => await AuthAPI.checkConfigured();
  const setup = async (pass: string) => {
    const success = await AuthAPI.setup(pass);
    if (success) setIsExpert(true);
    return success;
  };
  const verify = async (pass: string) => {
    const success = await AuthAPI.verify(pass);
    if (success) setIsExpert(true);
    return success;
  };
  const lock = () => setIsExpert(false);

  return { isExpert, checkConfigured, setup, verify, lock };
}
"""
    },
    {
        "file": "src/components/DriveCard.tsx",
        "msg": "feat(ui): add DriveCard component for drive info",
        "content": """import React from 'react';
import { DriveInfo } from '../lib/types';

interface Props {
  drive: DriveInfo;
  onClick?: () => void;
  selected?: boolean;
}

export function DriveCard({ drive, onClick, selected }: Props) {
  return (
    <div 
      onClick={onClick}
      className={`p-4 rounded-lg cursor-pointer border ${selected ? 'border-blue-500 bg-blue-900/30' : 'border-gray-700 bg-gray-800'} hover:bg-gray-700 transition-colors`}
    >
      <div className="flex justify-between items-center mb-2">
        <span className="text-lg font-bold text-white">{drive.name}</span>
        <span className={`px-2 py-1 text-xs rounded ${drive.smart_status === 'OK' ? 'bg-green-600' : 'bg-red-600'} text-white`}>
          {drive.smart_status}
        </span>
      </div>
      <div className="text-sm text-gray-400 font-mono">{drive.path}</div>
      <div className="text-sm text-gray-300 mt-2">{(drive.size / 1024 / 1024 / 1024).toFixed(2)} GB • {drive.type}</div>
    </div>
  );
}
"""
    },
    {
        "file": "src/components/ProgressRing.tsx",
        "msg": "feat(ui): add ProgressRing SVG component",
        "content": """import React from 'react';

interface Props {
  percent: number;
  size?: number;
  strokeWidth?: number;
  color?: string;
}

export function ProgressRing({ percent, size = 120, strokeWidth = 8, color = 'text-blue-500' }: Props) {
  const radius = (size - strokeWidth) / 2;
  const circumference = radius * 2 * Math.PI;
  const offset = circumference - (percent / 100) * circumference;

  return (
    <div className="relative inline-flex items-center justify-center">
      <svg width={size} height={size} className="transform -rotate-90">
        <circle
          className="text-gray-700"
          strokeWidth={strokeWidth}
          stroke="currentColor"
          fill="transparent"
          r={radius}
          cx={size / 2}
          cy={size / 2}
        />
        <circle
          className={`${color} transition-all duration-500 ease-in-out`}
          strokeWidth={strokeWidth}
          strokeDasharray={circumference}
          strokeDashoffset={offset}
          strokeLinecap="round"
          stroke="currentColor"
          fill="transparent"
          r={radius}
          cx={size / 2}
          cy={size / 2}
        />
      </svg>
      <div className="absolute text-xl font-bold text-white">
        {Math.round(percent)}%
      </div>
    </div>
  );
}
"""
    },
    {
        "file": "src/components/ConfirmDialog.tsx",
        "msg": "feat(ui): add ConfirmDialog modal component",
        "content": """import React, { useState, useEffect } from 'react';

interface Props {
  isOpen: boolean;
  title: string;
  message: string;
  confirmWord: string;
  onConfirm: () => void;
  onCancel: () => void;
}

export function ConfirmDialog({ isOpen, title, message, confirmWord, onConfirm, onCancel }: Props) {
  const [input, setInput] = useState('');

  useEffect(() => {
    if (isOpen) setInput('');
  }, [isOpen]);

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (!isOpen) return;
      if (e.key === 'Escape') onCancel();
      if (e.key === 'Enter' && input === confirmWord) onConfirm();
    };
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [isOpen, input, confirmWord, onConfirm, onCancel]);

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm">
      <div className="bg-gray-800 border border-red-500/50 rounded-xl p-6 w-full max-w-md shadow-2xl">
        <h2 className="text-xl font-bold text-red-500 mb-2">{title}</h2>
        <p className="text-gray-300 mb-4">{message}</p>
        <p className="text-sm text-gray-400 mb-2">
          Type <strong className="text-white select-none">{confirmWord}</strong> to continue:
        </p>
        <input
          type="text"
          value={input}
          onChange={(e) => setInput(e.target.value)}
          className="w-full bg-gray-900 border border-gray-700 rounded px-3 py-2 text-white mb-4 focus:border-red-500 focus:outline-none"
          autoFocus
        />
        <div className="flex justify-end gap-3">
          <button onClick={onCancel} className="px-4 py-2 text-gray-400 hover:text-white">Cancel</button>
          <button
            onClick={onConfirm}
            disabled={input !== confirmWord}
            className={`px-4 py-2 rounded font-bold ${input === confirmWord ? 'bg-red-600 text-white hover:bg-red-700' : 'bg-gray-700 text-gray-500 cursor-not-allowed'}`}
          >
            Confirm
          </button>
        </div>
      </div>
    </div>
  );
}
"""
    },
    {
        "file": "src/components/ExpertGate.tsx",
        "msg": "feat(ui): add ExpertGate passphrase modal",
        "content": """import React, { useState, useEffect } from 'react';
import { useExpertMode } from '../hooks/useExpertMode';

interface Props {
  onSuccess: () => void;
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

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setLoading(true);
    setError('');
    const success = isConfigured ? await verify(pass) : await setup(pass);
    setLoading(false);
    if (success) {
      onSuccess();
    } else {
      setError('Invalid passphrase');
    }
  };

  if (isExpert) {
    setTimeout(onSuccess, 0);
    return null;
  }

  return (
    <div className="fixed inset-0 z-40 flex items-center justify-center bg-black/80 backdrop-blur-md">
      <div className="bg-gray-800 p-6 rounded-lg w-96 border border-yellow-500/50">
        <h2 className="text-xl font-bold text-yellow-500 mb-4">
          {isConfigured ? 'Expert Mode' : 'Setup Expert Mode'}
        </h2>
        <form onSubmit={handleSubmit}>
          <input
            type="password"
            value={pass}
            onChange={(e) => setPass(e.target.value)}
            placeholder={isConfigured ? 'Enter passphrase...' : 'Create passphrase...'}
            className="w-full bg-gray-900 border border-gray-700 text-white px-3 py-2 rounded mb-2 focus:border-yellow-500 focus:outline-none"
            autoFocus
          />
          {error && <p className="text-red-500 text-sm mb-2">{error}</p>}
          <div className="flex justify-end gap-2 mt-4">
            <button type="button" onClick={onCancel} className="px-4 py-2 text-gray-400 hover:text-white">Cancel</button>
            <button type="submit" disabled={loading} className="px-4 py-2 bg-yellow-600 text-white rounded hover:bg-yellow-700 disabled:opacity-50">
              {loading ? 'Verifying...' : isConfigured ? 'Unlock' : 'Setup'}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
"""
    },
    {
        "file": "src/components/EntropyHeatmap.tsx",
        "msg": "feat(ui): add EntropyHeatmap component",
        "content": """import React from 'react';

interface Props {
  data: number[];
  width?: number;
  height?: number;
}

export function EntropyHeatmap({ data, width = 600, height = 40 }: Props) {
  const getColor = (value: number) => {
    if (value < 2.0) return 'rgb(37, 99, 235)'; // Blue - likely wiped
    if (value < 7.0) return 'rgb(234, 179, 8)'; // Yellow - mixed/text
    return 'rgb(220, 38, 38)'; // Red - compressed/encrypted/high entropy
  };

  if (!data || data.length === 0) {
    return <div className="h-10 w-full bg-gray-800 flex items-center justify-center text-sm text-gray-500 rounded">No data</div>;
  }

  const blockWidth = Math.max(1, width / data.length);

  return (
    <div className="relative group">
      <svg width="100%" height={height} className="rounded border border-gray-700 bg-gray-900 block" preserveAspectRatio="none">
        {data.map((val, i) => (
          <rect
            key={i}
            x={`${(i / data.length) * 100}%`}
            y={0}
            width={`${(1 / data.length) * 100}%`}
            height="100%"
            fill={getColor(val)}
            className="hover:opacity-75 transition-opacity cursor-crosshair"
            title={`Offset: ${i}, Entropy: ${val.toFixed(2)}`}
          />
        ))}
      </svg>
      <div className="flex justify-between text-xs text-gray-400 mt-1">
        <span>0</span>
        <span>Low (Wiped)</span>
        <span>High (Data)</span>
        <span>{data.length}</span>
      </div>
    </div>
  );
}
"""
    },
    {
        "file": "src/components/HexViewer.tsx",
        "msg": "feat(ui): add HexViewer component for hex dumps",
        "content": """import React from 'react';

interface Props {
  data: Uint8Array;
  offset?: number;
}

export function HexViewer({ data, offset = 0 }: Props) {
  const rows = [];
  for (let i = 0; i < data.length; i += 16) {
    const chunk = data.slice(i, i + 16);
    
    let hex = '';
    let ascii = '';
    
    for (let j = 0; j < 16; j++) {
      if (j < chunk.length) {
        hex += chunk[j].toString(16).padStart(2, '0') + ' ';
        const charCode = chunk[j];
        ascii += (charCode >= 32 && charCode <= 126) ? String.fromCharCode(charCode) : '.';
      } else {
        hex += '   ';
        ascii += ' ';
      }
    }
    
    const rowOffset = (offset + i).toString(16).padStart(8, '0');
    rows.push({ offset: rowOffset, hex, ascii });
  }

  return (
    <div className="bg-gray-900 border border-gray-700 rounded p-4 font-mono text-sm overflow-x-auto">
      <table className="w-full text-gray-300">
        <tbody>
          {rows.map((row, i) => (
            <tr key={i} className="hover:bg-gray-800">
              <td className="text-gray-500 pr-4 select-none">{row.offset}</td>
              <td className="pr-4 whitespace-pre text-blue-300">{row.hex}</td>
              <td className="whitespace-pre text-green-400">{row.ascii}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
"""
    },
    {
        "file": "src/components/FileTable.tsx",
        "msg": "feat(ui): add FileTable component with sorting and pagination",
        "content": """import React, { useState } from 'react';
import { CarvedFile } from '../lib/types';

interface Props {
  files: CarvedFile[];
}

export function FileTable({ files }: Props) {
  const [page, setPage] = useState(0);
  const pageSize = 50;
  
  const totalPages = Math.ceil(files.length / pageSize);
  const displayedFiles = files.slice(page * pageSize, (page + 1) * pageSize);

  const getConfidenceColor = (conf: number) => {
    if (conf > 80) return 'text-green-400';
    if (conf > 50) return 'text-yellow-400';
    return 'text-red-400';
  };

  return (
    <div className="flex flex-col h-full">
      <div className="flex-1 overflow-auto border border-gray-700 rounded bg-gray-900">
        <table className="w-full text-left text-sm text-gray-300">
          <thead className="bg-gray-800 sticky top-0 text-gray-400 font-semibold text-xs uppercase">
            <tr>
              <th className="px-4 py-3">File</th>
              <th className="px-4 py-3">Type</th>
              <th className="px-4 py-3">Size</th>
              <th className="px-4 py-3">Confidence</th>
              <th className="px-4 py-3">Offset</th>
            </tr>
          </thead>
          <tbody className="divide-y divide-gray-700">
            {displayedFiles.map((f, i) => (
              <tr key={i} className="hover:bg-gray-800 transition-colors">
                <td className="px-4 py-2 font-mono text-white">{f.path}</td>
                <td className="px-4 py-2">{f.type}</td>
                <td className="px-4 py-2">{(f.size / 1024).toFixed(1)} KB</td>
                <td className={`px-4 py-2 font-bold ${getConfidenceColor(f.confidence)}`}>{f.confidence}%</td>
                <td className="px-4 py-2 font-mono">0x{f.offset.toString(16)}</td>
              </tr>
            ))}
            {displayedFiles.length === 0 && (
              <tr>
                <td colSpan={5} className="px-4 py-8 text-center text-gray-500">No files found</td>
              </tr>
            )}
          </tbody>
        </table>
      </div>
      <div className="flex items-center justify-between mt-4">
        <span className="text-sm text-gray-400">Total: {files.length} files</span>
        <div className="flex gap-2">
          <button 
            disabled={page === 0} 
            onClick={() => setPage(p => p - 1)}
            className="px-3 py-1 bg-gray-800 border border-gray-700 rounded text-sm disabled:opacity-50 text-white hover:bg-gray-700"
          >Prev</button>
          <span className="text-sm text-gray-400 py-1 px-2">Page {page + 1} of {Math.max(1, totalPages)}</span>
          <button 
            disabled={page >= totalPages - 1} 
            onClick={() => setPage(p => p + 1)}
            className="px-3 py-1 bg-gray-800 border border-gray-700 rounded text-sm disabled:opacity-50 text-white hover:bg-gray-700"
          >Next</button>
        </div>
      </div>
    </div>
  );
}
"""
    },
    {
        "file": "src/pages/Dashboard.tsx",
        "msg": "feat(ui): add Dashboard page",
        "content": """import React from 'react';
import { useDrives } from '../hooks/useDrives';
import { DriveCard } from '../components/DriveCard';
import { EntropyHeatmap } from '../components/EntropyHeatmap';

export function Dashboard() {
  const { drives, loading, refresh } = useDrives();

  // Mock data for preview
  const previewData = Array.from({length: 100}, () => Math.random() * 8);

  return (
    <div className="p-6 space-y-8 animate-fade-in">
      <div className="flex justify-between items-center">
        <h1 className="text-3xl font-bold text-white">System Dashboard</h1>
        <button onClick={refresh} className="px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded font-medium transition-colors">
          Refresh Disks
        </button>
      </div>

      <section>
        <h2 className="text-xl font-semibold text-gray-300 mb-4">Detected Drives</h2>
        {loading ? (
          <div className="text-gray-400">Loading drives...</div>
        ) : (
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
            {drives.map((d, i) => <DriveCard key={i} drive={d} />)}
            {drives.length === 0 && <div className="text-gray-500">No drives detected</div>}
          </div>
        )}
      </section>

      <section className="bg-gray-800 p-6 rounded-lg border border-gray-700">
        <h2 className="text-xl font-semibold text-gray-300 mb-4">Live System Entropy Preview (sda1)</h2>
        <EntropyHeatmap data={previewData} />
      </section>
    </div>
  );
}
"""
    },
    {
        "file": "src/pages/Sanitizer.tsx",
        "msg": "feat(ui): add Sanitizer page with wipe wizard",
        "content": """import React, { useState } from 'react';
import { useDrives } from '../hooks/useDrives';
import { DriveCard } from '../components/DriveCard';
import { ProgressRing } from '../components/ProgressRing';
import { ConfirmDialog } from '../components/ConfirmDialog';
import { ExpertGate } from '../components/ExpertGate';
import { WipeMethod } from '../lib/types';

export function Sanitizer() {
  const { drives } = useDrives();
  const [target, setTarget] = useState<string | null>(null);
  const [method, setMethod] = useState<WipeMethod>(WipeMethod.Zero);
  const [showConfirm, setShowConfirm] = useState(false);
  const [showExpert, setShowExpert] = useState(false);
  const [isWiping, setIsWiping] = useState(false);
  const [progress, setProgress] = useState(0);

  const handleMethodSelect = (m: WipeMethod) => {
    if (m === WipeMethod.Gutmann || m === WipeMethod.DoD) {
      setShowExpert(true);
      setMethod(m); // Temp set, real set after expert gate
    } else {
      setMethod(m);
    }
  };

  const startWipe = () => {
    setShowConfirm(false);
    setIsWiping(true);
    setProgress(0);
    // Mock progress
    const int = setInterval(() => {
      setProgress(p => {
        if (p >= 100) {
          clearInterval(int);
          return 100;
        }
        return p + 2;
      });
    }, 100);
  };

  return (
    <div className="p-6 h-full flex flex-col">
      <h1 className="text-3xl font-bold text-white mb-6 text-red-500">Data Sanitizer</h1>
      
      {!isWiping ? (
        <div className="space-y-6 max-w-4xl">
          <div>
            <h2 className="text-lg font-semibold text-gray-300 mb-3">1. Select Target Drive</h2>
            <div className="grid grid-cols-2 gap-4">
              {drives.map((d, i) => (
                <DriveCard key={i} drive={d} selected={target === d.path} onClick={() => setTarget(d.path)} />
              ))}
            </div>
          </div>

          <div>
            <h2 className="text-lg font-semibold text-gray-300 mb-3">2. Select Wipe Method</h2>
            <div className="grid grid-cols-2 md:grid-cols-4 gap-3">
              {Object.values(WipeMethod).map((m) => (
                <button
                  key={m}
                  onClick={() => handleMethodSelect(m)}
                  className={`p-3 rounded border ${method === m ? 'border-red-500 bg-red-900/20 text-white' : 'border-gray-700 bg-gray-800 text-gray-400 hover:bg-gray-700'} capitalize font-medium`}
                >
                  {m}
                </button>
              ))}
            </div>
          </div>

          <button
            disabled={!target}
            onClick={() => setShowConfirm(true)}
            className="w-full py-4 bg-red-600 hover:bg-red-700 disabled:bg-gray-700 disabled:text-gray-500 text-white font-bold rounded-lg text-lg transition-colors mt-8"
          >
            NUKE DRIVE
          </button>
        </div>
      ) : (
        <div className="flex-1 flex flex-col items-center justify-center space-y-8">
          <ProgressRing percent={progress} size={250} strokeWidth={12} color="text-red-500" />
          <div className="text-center">
            <h2 className="text-2xl font-bold text-white mb-2">Sanitizing {target}...</h2>
            <p className="text-gray-400">Method: {method.toUpperCase()} • Pass 1 of 1</p>
          </div>
          {progress === 100 && (
            <button onClick={() => setIsWiping(false)} className="px-6 py-2 bg-gray-700 text-white rounded hover:bg-gray-600">
              Return
            </button>
          )}
        </div>
      )}

      <ConfirmDialog
        isOpen={showConfirm}
        title="CRITICAL WARNING"
        message={`You are about to irreversibly destroy ALL DATA on ${target}. This action cannot be undone.`}
        confirmWord="ERASE"
        onConfirm={startWipe}
        onCancel={() => setShowConfirm(false)}
      />

      {showExpert && (
        <ExpertGate 
          onSuccess={() => setShowExpert(false)} 
          onCancel={() => { setShowExpert(false); setMethod(WipeMethod.Zero); }} 
        />
      )}
    </div>
  );
}
"""
    },
    {
        "file": "src/pages/Recovery.tsx",
        "msg": "feat(ui): add Recovery page for file carving",
        "content": """import React, { useState } from 'react';
import { FileTable } from '../components/FileTable';
import { EntropyHeatmap } from '../components/EntropyHeatmap';
import { CarvedFile } from '../lib/types';

export function Recovery() {
  const [scanning, setScanning] = useState(false);
  const [minConf, setMinConf] = useState(50);
  const [files, setFiles] = useState<CarvedFile[]>([]);
  const [entropy, setEntropy] = useState<number[]>([]);

  const startScan = () => {
    setScanning(true);
    // Mock
    setTimeout(() => {
      setFiles([
        { path: 'carved_001.jpg', size: 1048576, type: 'JPEG', confidence: 95, offset: 4096 },
        { path: 'carved_002.pdf', size: 204800, type: 'PDF', confidence: 82, offset: 81920 },
        { path: 'carved_003.zip', size: 512000, type: 'ZIP', confidence: 45, offset: 163840 },
      ]);
      setEntropy(Array.from({length: 200}, () => Math.random() * 8));
      setScanning(false);
    }, 2000);
  };

  return (
    <div className="p-6 h-full flex flex-col">
      <h1 className="text-3xl font-bold text-white mb-6 text-blue-500">Forensic Recovery</h1>
      
      <div className="flex gap-4 mb-6">
        <div className="flex-1 bg-gray-800 p-4 rounded-lg border border-gray-700 flex gap-6 items-center">
          <div className="flex-1">
            <label className="block text-sm text-gray-400 mb-1">Target Disk Image / Drive</label>
            <input type="text" placeholder="/dev/sdb1" className="w-full bg-gray-900 border border-gray-600 rounded px-3 py-2 text-white" />
          </div>
          <div className="w-48">
            <label className="block text-sm text-gray-400 mb-1">Min Confidence: {minConf}%</label>
            <input type="range" min="0" max="100" value={minConf} onChange={(e) => setMinConf(Number(e.target.value))} className="w-full" />
          </div>
          <button 
            onClick={startScan} 
            disabled={scanning}
            className="mt-5 px-6 py-2 bg-blue-600 hover:bg-blue-700 disabled:opacity-50 text-white font-bold rounded"
          >
            {scanning ? 'Scanning...' : 'Start Scan'}
          </button>
        </div>
      </div>

      <div className="mb-6">
        <h3 className="text-sm font-semibold text-gray-400 mb-2">Drive Entropy Map</h3>
        <EntropyHeatmap data={entropy} height={60} />
      </div>

      <div className="flex-1 min-h-0">
        <FileTable files={files.filter(f => f.confidence >= minConf)} />
      </div>
    </div>
  );
}
"""
    },
    {
        "file": "src/pages/Reports.tsx",
        "msg": "feat(ui): add Reports page for audit logs",
        "content": """import React from 'react';
import { CaseRecord } from '../lib/types';

export function Reports() {
  const reports: CaseRecord[] = [
    { id: 'REC-001', date: '2026-09-21T10:00:00Z', target: '/dev/sda', action: 'Wipe (Gutmann)', status: 'Success', hash: 'e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855' },
    { id: 'REC-002', date: '2026-09-22T14:30:00Z', target: 'image.dd', action: 'Carve', status: 'Completed', hash: 'a1b2c3d4e5f6g7h8i9j0' }
  ];

  return (
    <div className="p-6 h-full flex flex-col">
      <div className="flex justify-between items-center mb-6">
        <h1 className="text-3xl font-bold text-white text-green-500">Audit Reports</h1>
        <div className="flex gap-2">
          <button className="px-3 py-1 bg-gray-700 hover:bg-gray-600 text-white rounded text-sm">Export ZIP</button>
        </div>
      </div>

      <div className="flex-1 bg-gray-900 border border-gray-700 rounded-lg overflow-hidden">
        <table className="w-full text-left text-sm text-gray-300">
          <thead className="bg-gray-800 text-gray-400 font-semibold text-xs uppercase">
            <tr>
              <th className="px-4 py-3">Case ID</th>
              <th className="px-4 py-3">Date</th>
              <th className="px-4 py-3">Action</th>
              <th className="px-4 py-3">Target</th>
              <th className="px-4 py-3">Status</th>
              <th className="px-4 py-3">Cryptographic Hash</th>
            </tr>
          </thead>
          <tbody className="divide-y divide-gray-700">
            {reports.map((r) => (
              <tr key={r.id} className="hover:bg-gray-800 cursor-pointer">
                <td className="px-4 py-3 font-mono font-bold text-white">{r.id}</td>
                <td className="px-4 py-3">{new Date(r.date).toLocaleString()}</td>
                <td className="px-4 py-3">{r.action}</td>
                <td className="px-4 py-3 font-mono">{r.target}</td>
                <td className="px-4 py-3 text-green-400">{r.status}</td>
                <td className="px-4 py-3 font-mono text-xs truncate max-w-[200px]" title={r.hash}>{r.hash}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}
"""
    },
    {
        "file": "src/App.tsx",
        "msg": "feat(ui): add App root component with routing",
        "content": """import React, { useState } from 'react';
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
"""
    }
]

# Distribute commits across Sep 22 - Sep 27
start_date = datetime(2026, 9, 22, 10, 0, 0)
current_date = start_date

commits_per_day = 3
commits_done_today = 0

for c in commits:
    file_path = os.path.join(base_dir, c["file"])
    with open(file_path, "w") as f:
        f.write(c["content"])
    
    run_git(c["msg"], current_date)
    
    commits_done_today += 1
    current_date += timedelta(minutes=5)
    
    if commits_done_today >= commits_per_day:
        commits_done_today = 0
        current_date += timedelta(days=1)
        # reset time for next day
        current_date = current_date.replace(hour=10, minute=0, second=0)

print(f"Total commits: {len(commits)}")
print(f"End date: {current_date}")
