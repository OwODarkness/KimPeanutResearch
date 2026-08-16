import { open } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";

export interface LocalPdfImportReceipt {
  paperId: string;
  sourceFileName: string;
  byteCount: number;
  sha256: string;
}

/** Opens the desktop PDF picker and delegates all file handling to Rust. */
export async function selectAndImportLocalPdf(): Promise<LocalPdfImportReceipt | null> {
  if (!("__TAURI_INTERNALS__" in window)) {
    throw new Error("Local PDF import is available only in the KimPeanut desktop app, not a web browser.");
  }

  const selectedPath = await open({
    directory: false,
    multiple: false,
    filters: [{ name: "PDF documents", extensions: ["pdf"] }],
  });

  if (selectedPath === null) {
    return null;
  }
  if (Array.isArray(selectedPath)) {
    throw new Error("Select one PDF at a time.");
  }

  return invoke<LocalPdfImportReceipt>("import_local_pdf", { sourcePath: selectedPath });
}
