import { invoke } from "@tauri-apps/api/core";

export interface StorageLocation { dataDir: string; layoutVersion: number; usesCustomLocation: boolean; }
export interface StorageMigrationReceipt { previousDataDir: string; dataDir: string; copiedEntries: number; previousLocationRetained: boolean; }

export function getStorageLocation() { return invoke<StorageLocation>("get_storage_location"); }
export function migrateStorageLocation(destination: string) { return invoke<StorageMigrationReceipt>("migrate_storage_location", { destination }); }
