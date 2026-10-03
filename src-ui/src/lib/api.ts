import { invoke } from '@tauri-apps/api/core';
import { listen, Event } from '@tauri-apps/api/event';
import { DriveInfo, WipeConfig, WipeProgress, WipeResult, ScanConfig, ScanProgress, ScanResult, CaseRecord } from './types';

export const DrivesAPI = {
  list: (): Promise<DriveInfo[]> => invoke('get_drives')
};

export const WiperAPI = {
  start: (config: WipeConfig): Promise<void> => invoke('start_wipe', { config }),
  onProgress: (cb: (p: WipeProgress) => void) => listen('wipe_progress', (e: Event<WipeProgress>) => cb(e.payload)),
  getResult: (): Promise<WipeResult> => invoke('get_wipe_result')
};

export const CarverAPI = {
  start: (config: ScanConfig): Promise<void> => invoke('start_scan', { config }),
  onProgress: (cb: (p: ScanProgress) => void) => listen('scan_progress', (e: Event<ScanProgress>) => cb(e.payload)),
  getResult: (): Promise<ScanResult> => invoke('get_scan_result')
};

export const AuthAPI = {
  checkConfigured: (): Promise<boolean> => invoke('check_expert_configured'),
  setup: (pass: string): Promise<boolean> => invoke('setup_expert_mode', { pass }),
  verify: (pass: string): Promise<boolean> => invoke('verify_expert_mode', { pass })
};

export const ReportsAPI = {
  list: (): Promise<CaseRecord[]> => invoke('get_reports'),
  export: (id: string, format: string): Promise<string> => invoke('export_report', { id, format })
};
