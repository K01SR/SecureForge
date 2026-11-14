import { useState } from 'react';
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
