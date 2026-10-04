import { useState } from 'react';
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
