/**
 * Shared TypeScript type definitions.
 *
 * Mirrors the Rust serde structs from sih149-core for
 * type-safe data interchange between frontend and backend.
 */

export interface DriveInfo {
  name: string;
  model: string;
  serial: string;
  busType: "NVMe" | "SATA" | "USB" | "SD" | "Unknown";
  capacityBytes: number;
  smartStatus: "Healthy" | "Warning" | "Critical" | "Unknown";
}

export interface RecoveredFile {
  id: number;
  filename: string;
  fileType: string;
  mimeType: string;
  category: "Documents" | "Media" | "Archives" | "Databases" | "System" | "Unknown";
  fileSize: number;
  sectorOffset: number;
  confidenceScore: number;
  sha256: string;
}

export interface AuditEntry {
  id: number;
  entryHash: string;
  previousHash: string | null;
  timestamp: string;
  payload: Record<string, unknown>;
}
