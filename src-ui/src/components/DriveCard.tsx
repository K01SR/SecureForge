import React from 'react';
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
