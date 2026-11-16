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
} from './types';

// Safely detect if running inside Tauri desktop runtime
const isTauri = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;

async function tauriInvoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  if (isTauri) {
    const { invoke } = await import('@tauri-apps/api/core');
    return invoke<T>(cmd, args);
  }
  // Standalone Browser Mock Fallbacks
  return mockInvoke<T>(cmd, args);
}

async function tauriListen<T>(event: string, handler: (payload: T) => void): Promise<() => void> {
  if (isTauri) {
    const { listen } = await import('@tauri-apps/api/event');
    const unlisten = await listen<T>(event, (e) => handler(e.payload));
    return unlisten;
  }
  return () => {};
}

// Fallback mocks for preview in regular web browser
function mockInvoke<T>(cmd: string, _args?: Record<string, unknown>): Promise<T> {
  switch (cmd) {
    case 'list_drives':
      return Promise.resolve([
        {
          name: 'nvme0n1',
          path: '/dev/nvme0n1',
          size_bytes: 512110190592,
          model: 'Samsung SSD 980 PRO 500GB',
          serial: 'S5GXNF0T123456',
          drive_type: 'NVMe',
          is_mounted: true,
          mount_points: ['/', '/boot/efi'],
          is_system_drive: true,
          smart_status: 'Healthy',
        },
        {
          name: 'sda',
          path: '/dev/sda',
          size_bytes: 2000398934016,
          model: 'Crucial MX500 2TB',
          serial: '2142E5C12345',
          drive_type: 'SSD',
          is_mounted: false,
          mount_points: [],
          is_system_drive: false,
          smart_status: 'Healthy',
        },
        {
          name: 'sdb',
          path: '/dev/sdb',
          size_bytes: 4000787030016,
          model: 'WDC WD40EZAZ-00SF3B0',
          serial: 'WD-WX32D1234567',
          drive_type: 'HDD',
          is_mounted: false,
          mount_points: [],
          is_system_drive: false,
          smart_status: 'Warning',
        },
      ] as unknown as T);

    case 'start_wipe':
      return new Promise((resolve) => {
        setTimeout(() => {
          resolve({
            success: true,
            sectors_wiped: 3907029168,
            bad_sectors: 0,
            duration_secs: 42,
            method_used: 'dod3',
            verified: true,
          } as unknown as T);
        }, 1500);
      });

    case 'shred_files':
      return Promise.resolve({
        total_files: 3,
        total_bytes: 14258900,
        failed_files: 0,
        results: [
          { path: '/tmp/evidence/doc1.pdf', bytes_wiped: 4258900, passes_completed: 3, success: true },
          { path: '/tmp/evidence/db.sqlite', bytes_wiped: 10000000, passes_completed: 3, success: true },
        ],
      } as unknown as T);

    case 'start_scan':
      return Promise.resolve({
        total_files: 4,
        total_size_bytes: 24589000,
        duration_secs: 12,
        entropy_heatmap: Array.from({ length: 100 }, () => Math.random() * 8),
        files: [
          { id: '1', filename: 'carved_00000200.jpg', file_type: 'jpg', size_bytes: 4250000, confidence: 95, offset_bytes: 512, category: 'Media' },
          { id: '2', filename: 'carved_00450000.png', file_type: 'png', size_bytes: 1200000, confidence: 90, offset_bytes: 4521984, category: 'Media' },
          { id: '3', filename: 'carved_00900000.pdf', file_type: 'pdf', size_bytes: 8400000, confidence: 85, offset_bytes: 9437184, category: 'Document' },
          { id: '4', filename: 'carved_01500000.sqlite', file_type: 'sqlite', size_bytes: 10739000, confidence: 92, offset_bytes: 15728640, category: 'Database' },
        ],
      } as unknown as T);

    case 'detect_firmware_capabilities':
      return Promise.resolve({
        is_nvme: true,
        nvme_sanitize_supported: true,
        ata_frozen: false,
        hpa_enabled: false,
        dco_enabled: false,
        recommended_method: 'nvme-crypto',
        warnings: [],
      } as unknown as T);

    case 'list_cases':
      return Promise.resolve([
        { id: 'CASE-2026-001', created_at: '2026-11-10 14:30:00', operation_type: 'Disk Wipe (DoD 3-Pass)', target: '/dev/sdb', status: 'Completed & Certified' },
        { id: 'CASE-2026-002', created_at: '2026-11-12 10:15:00', operation_type: 'Forensic File Carving', target: 'evidence_image.dd', status: 'Completed' },
      ] as unknown as T);

    default:
      return Promise.resolve({} as T);
  }
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
