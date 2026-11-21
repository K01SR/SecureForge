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

// Detect if running inside Tauri desktop runtime
const isTauri = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;

async function tauriInvoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  if (isTauri) {
    const { invoke } = await import('@tauri-apps/api/core');
    return invoke<T>(cmd, args);
  }

  // Real REST API execution over HTTP server when accessed in a browser
  const baseUrl = typeof window !== 'undefined' ? window.location.origin : 'http://127.0.0.1:7878';
  const token = typeof localStorage !== 'undefined' ? localStorage.getItem('secureforge_api_token') : '';

  const headers: Record<string, string> = {
    'Content-Type': 'application/json',
  };
  if (token) {
    headers['Authorization'] = `Bearer ${token}`;
  }

  try {
    switch (cmd) {
      case 'list_drives': {
        const res = await fetch(`${baseUrl}/api/drives`, { headers });
        const json = await res.json();
        if (json.status === 'success') return json.data as T;
        throw new Error(json.message || 'Failed to query system drives');
      }

      case 'start_wipe': {
        const res = await fetch(`${baseUrl}/api/wipe`, {
          method: 'POST',
          headers,
          body: JSON.stringify(args?.config || args),
        });
        return (await res.json()) as T;
      }

      case 'start_scan': {
        const res = await fetch(`${baseUrl}/api/scan`, {
          method: 'POST',
          headers,
          body: JSON.stringify(args?.config || args),
        });
        return (await res.json()) as T;
      }

      case 'list_cases': {
        const res = await fetch(`${baseUrl}/api/cases`, { headers });
        const json = await res.json();
        return (json.data || []) as T;
      }

      case 'list_plugins': {
        const res = await fetch(`${baseUrl}/api/plugins`, { headers });
        const json = await res.json();
        return (json.data || []) as T;
      }

      case 'get_drive_entropy': {
        const res = await fetch(`${baseUrl}/api/entropy`, {
          method: 'POST',
          headers,
          body: JSON.stringify(args),
        });
        const json = await res.json();
        return (json.data || []) as T;
      }

      default:
        throw new Error(`Command '${cmd}' requires direct Tauri runtime or mapped REST route.`);
    }
  } catch (err: any) {
    console.warn(`[SecureForge API] ${cmd} error:`, err);
    throw err;
  }
}

async function tauriListen<T>(event: string, handler: (payload: T) => void): Promise<() => void> {
  if (isTauri) {
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
  getHexPreview: (filePath: string, offset: number, length: number): Promise<string> =>
    tauriInvoke('get_file_hex_preview', { filePath, offset, length }),
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
