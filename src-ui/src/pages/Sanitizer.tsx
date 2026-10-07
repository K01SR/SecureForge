import React, { useState } from 'react';
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
