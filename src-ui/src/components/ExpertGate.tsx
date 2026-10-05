import React, { useState, useEffect } from 'react';
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
