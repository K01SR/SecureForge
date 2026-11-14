import { useState } from 'react';
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
