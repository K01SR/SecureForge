import { isTauri, invoke } from '@tauri-apps/api/core';
import {
  DriveInfo,
  WipeConfig,
  WipeProgress,
  WipeResult,
  ShredConfig,
  ShredProgress,
  ShredResult,
  ScanConfig,
  ScanProgress,
  ScanResult,
  FirmwareCapabilities,
  FirmwareEraseConfig,
  FirmwareEraseResult,
  CaseRecord,
  PluginItem,
} from './types';

// Runtime detection of Tauri desktop environment
export function checkIsTauri(): boolean {
  try {
    if (typeof window === 'undefined') return false;
    return isTauri() || '__TAURI_INTERNALS__' in window || '__TAURI__' in window;
  } catch {
    return false;
  }
}

export function getSavedToken(): string {
  if (typeof sessionStorage !== 'undefined') {
    return sessionStorage.getItem('secureforge_api_token') || '';
  }
  return '';
}

export function saveToken(token: string) {
  if (typeof sessionStorage !== 'undefined') {
    sessionStorage.setItem('secureforge_api_token', token.trim());
  }
}

export function clearToken() {
  if (typeof sessionStorage !== 'undefined') {
    sessionStorage.removeItem('secureforge_api_token');
  }
}

async function safeFetchJson<T>(url: string, options?: RequestInit): Promise<T> {
  const token = getSavedToken();

  const headers: Record<string, string> = {
    'Content-Type': 'application/json',
    ...(options?.headers as Record<string, string> || {}),
  };
  if (token) {
    headers['Authorization'] = `Bearer ${token}`;
  }

  let res: Response;
  try {
    res = await fetch(url, { ...options, headers });
  } catch (netErr: any) {
    throw new Error(`Failed to connect to SecureForge backend server: ${netErr.message || netErr}`);
  }

  if (res.status === 401) {
    if (typeof window !== 'undefined') {
      window.dispatchEvent(new CustomEvent('secureforge-auth-required'));
    }
    throw new Error('Unauthorized: Missing or invalid Bearer token.');
  }

  const contentType = res.headers.get('content-type') || '';
  if (!contentType.includes('application/json')) {
    if (!res.ok) {
      const txt = await res.text();
      throw new Error(`Server returned HTTP ${res.status}: ${txt.slice(0, 100)}`);
    }
    throw new Error(
      `Unexpected server response (${contentType || 'HTML'}). Ensure SecureForge backend server is running on http://127.0.0.1:7878`
    );
  }

  const json = await res.json();
  if (!res.ok) {
    throw new Error(json.message || `Request failed with HTTP status ${res.status}`);
  }
  return json as T;
}

async function tauriInvoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  // 1. Direct Tauri IPC in desktop application
  if (checkIsTauri()) {
    return invoke<T>(cmd, args);
  }

  // 2. HTTP REST fallback when running in standard browser
  const baseUrl = typeof window !== 'undefined' ? window.location.origin : 'https://127.0.0.1:7878';

  try {
    switch (cmd) {
      case 'list_drives': {
        const json: any = await safeFetchJson(`${baseUrl}/api/drives`);
        if (json.status === 'success') return json.data as T;
        throw new Error(json.message || 'Failed to query system drives');
      }

      case 'start_wipe': {
        return await safeFetchJson<T>(`${baseUrl}/api/wipe`, {
          method: 'POST',
          body: JSON.stringify(args?.config || args),
        });
      }

      case 'start_scan': {
        return await safeFetchJson<T>(`${baseUrl}/api/scan`, {
          method: 'POST',
          body: JSON.stringify(args?.config || args),
        });
      }

      case 'list_cases': {
        const json: any = await safeFetchJson(`${baseUrl}/api/cases`);
        return (json.data || []) as T;
      }

      case 'list_plugins': {
        const json: any = await safeFetchJson(`${baseUrl}/api/plugins`);
        return (json.data || []) as T;
      }

      case 'get_drive_entropy': {
        const json: any = await safeFetchJson(`${baseUrl}/api/entropy`, {
          method: 'POST',
          body: JSON.stringify(args),
        });
        return (json.data || []) as T;
      }

      default:
        throw new Error(`Command '${cmd}' requires active desktop runtime or mapped REST route.`);
    }
  } catch (err: any) {
    console.warn(`[SecureForge API] ${cmd} error:`, err);
    throw err;
  }
}

async function tauriListen<T>(event: string, handler: (payload: T) => void): Promise<() => void> {
  if (checkIsTauri()) {
    const { listen } = await import('@tauri-apps/api/event');
    const unlisten = await listen<T>(event, (e) => handler(e.payload));
    return unlisten;
  }
  return () => {};
}

export const DrivesAPI = {
  list: (): Promise<DriveInfo[]> => tauriInvoke('list_drives'),
  getInfo: (devicePath: string): Promise<DriveInfo> => tauriInvoke('get_drive_info', { devicePath }),
};

export const WiperAPI = {
  start: (config: WipeConfig): Promise<WipeResult> => tauriInvoke('start_wipe', { config }),
  cancel: (): Promise<void> => tauriInvoke('cancel_wipe'),
  estimateTime: (devicePath: string, method: string): Promise<number> =>
    tauriInvoke('estimate_wipe_time', { devicePath, method }),
  onProgress: (cb: (p: WipeProgress) => void) => tauriListen<WipeProgress>('wipe-progress', cb),
};

export const ShredderAPI = {
  shred: (config: ShredConfig): Promise<ShredResult> => tauriInvoke('shred_files', { config }),
  onProgress: (cb: (p: ShredProgress) => void) => tauriListen<ShredProgress>('shred-progress', cb),
};

export const CarverAPI = {
  start: (config: ScanConfig): Promise<ScanResult> => tauriInvoke('start_scan', { config }),
  cancel: (): Promise<void> => tauriInvoke('cancel_scan'),
  getHexPreview: (filePath: string, offset: number, length: number, allowedRoot?: string): Promise<string> =>
    tauriInvoke('get_file_hex_preview', { filePath, offset, length, allowedRoot }),
  onProgress: (cb: (p: ScanProgress) => void) => tauriListen<ScanProgress>('scan-progress', cb),
};

export const FirmwareAPI = {
  detect: (devicePath: string): Promise<FirmwareCapabilities> =>
    tauriInvoke('detect_firmware_capabilities', { devicePath }),
  erase: (config: FirmwareEraseConfig): Promise<FirmwareEraseResult> =>
    tauriInvoke('start_firmware_erase', { config }),
};

export const AuthAPI = {
  isConfigured: (): Promise<boolean> => tauriInvoke('is_expert_configured'),
  setup: (passphrase: string): Promise<void> => tauriInvoke('setup_expert_passphrase', { passphrase }),
  verify: (passphrase: string): Promise<boolean> => tauriInvoke('verify_expert_passphrase', { passphrase }),
};

export const ReportsAPI = {
  list: (): Promise<CaseRecord[]> => tauriInvoke('list_cases'),
  export: (caseId: string, format: string, outputPath: string): Promise<string> =>
    tauriInvoke('export_report', { caseId, format, outputPath }),
  getAuditLog: (caseId: string): Promise<string> => tauriInvoke('get_audit_log', { caseId }),
};

export const PluginsAPI = {
  list: (): Promise<PluginItem[]> => tauriInvoke('list_plugins'),
};

export const EntropyAPI = {
  get: (devicePath: string, chunks?: number): Promise<number[]> =>
    tauriInvoke('get_drive_entropy', { devicePath, chunks }),
};
