export interface DriveInfo {
  path: string;
  name: string;
  size: number;
  type: string;
  smart_status: string;
}

export interface WipeConfig {
  target: string;
  method: WipeMethod;
}

export enum WipeMethod {
  Zero = 'zero',
  Random = 'random',
  DoD = 'dod',
  Gutmann = 'gutmann'
}

export interface WipeProgress {
  percent: number;
  current_pass: number;
  total_passes: number;
  speed_bytes_sec: number;
}

export interface WipeResult {
  success: boolean;
  error?: string;
  hash: string;
}

export interface ScanConfig {
  target: string;
  min_confidence: number;
  file_types: string[];
}

export interface CarvedFile {
  path: string;
  size: number;
  type: string;
  confidence: number;
  offset: number;
}

export interface ScanProgress {
  percent: number;
  files_found: number;
  current_sector: number;
}

export interface ScanResult {
  files: CarvedFile[];
  entropy_map: number[];
}

export interface CaseRecord {
  id: string;
  date: string;
  target: string;
  action: string;
  status: string;
  hash: string;
}
