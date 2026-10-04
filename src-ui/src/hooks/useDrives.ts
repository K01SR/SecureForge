import { useState, useEffect } from 'react';
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
