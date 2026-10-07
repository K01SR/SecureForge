import React from 'react';
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
