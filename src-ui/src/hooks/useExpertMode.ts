import { useState } from 'react';
import { AuthAPI } from '../lib/api';

export function useExpertMode() {
  const [isExpert, setIsExpert] = useState(false);

  const checkConfigured = async () => await AuthAPI.isConfigured();

  const setup = async (pass: string) => {
    try {
      await AuthAPI.setup(pass);
      setIsExpert(true);
      return true;
    } catch {
      return false;
    }
  };

  const verify = async (pass: string) => {
    try {
      const ok = await AuthAPI.verify(pass);
      if (ok) setIsExpert(true);
      return ok;
    } catch {
      return false;
    }
  };

  const lock = () => setIsExpert(false);

  return { isExpert, checkConfigured, setup, verify, lock };
}
