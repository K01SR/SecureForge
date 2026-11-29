import { createContext, useContext, useState, useCallback, ReactNode } from 'react';
import { AuthAPI } from '../lib/api';

interface ExpertModeContextType {
  isExpert: boolean;
  checkConfigured: () => Promise<boolean>;
  setup: (pass: string) => Promise<boolean>;
  verify: (pass: string) => Promise<boolean>;
  lock: () => void;
}

const ExpertModeContext = createContext<ExpertModeContextType | null>(null);

export function ExpertModeProvider({ children }: { children: ReactNode }) {
  const [isExpert, setIsExpert] = useState(false);

  const checkConfigured = useCallback(async () => await AuthAPI.isConfigured(), []);

  const setup = useCallback(async (pass: string) => {
    try {
      await AuthAPI.setup(pass);
      setIsExpert(true);
      return true;
    } catch {
      return false;
    }
  }, []);

  const verify = useCallback(async (pass: string) => {
    try {
      const ok = await AuthAPI.verify(pass);
      if (ok) setIsExpert(true);
      return ok;
    } catch {
      return false;
    }
  }, []);

  const lock = useCallback(() => setIsExpert(false), []);

  return (
    <ExpertModeContext.Provider value={{ isExpert, checkConfigured, setup, verify, lock }}>
      {children}
    </ExpertModeContext.Provider>
  );
}

export function useExpertMode(): ExpertModeContextType {
  const ctx = useContext(ExpertModeContext);
  if (!ctx) throw new Error('useExpertMode must be used within ExpertModeProvider');
  return ctx;
}
