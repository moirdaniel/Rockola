import { invoke } from "@tauri-apps/api/core";
import type { Artist, ItemRow, MediaType } from "../ui/types";


export async function getMediaPort(): Promise<number> {
  return invoke<number>("get_media_port");
}

function logInvoke<T>(cmd: string, payload?: Record<string, unknown>) {
  console.log(`🛰️ [TAURI] invoke("${cmd}")`, payload ?? {});
  return invoke<T>(cmd, payload);
}

type ArtistTuple = [number, string, string]; // [id, displayName, sourcePath]
type ItemTuple = [number, string, string, string]; // [id, title, fullPath, mediaType]

function normalizeMediaType(v: string): MediaType {
  return v === "audio" ? "audio" : "video";
}

export async function addSource(sourcePath: string): Promise<void> {
  const rootPath = (sourcePath || "").trim();
  console.log("➕ [UI->TAURI] addSource()", { rootPath });
  await logInvoke<void>("add_source", { rootPath });
}

export async function startScan(sourcePath: string): Promise<void> {
  const rootPath = (sourcePath || "").trim();
  console.log("🚀 [UI->TAURI] startScan()", { rootPath });
  await logInvoke<void>("start_scan", { rootPath });
}

export async function listArtists(): Promise<Artist[]> {
  console.log("📚 [UI->TAURI] listArtists()");
  const rows = await logInvoke<ArtistTuple[]>("list_artists");
  console.log("📦 [TAURI] list_artists raw:", rows);

  const mapped: Artist[] = rows.map(([id, displayName]) => ({
    id,
    displayName,
  }));

  console.log("✅ [UI] listArtists mapped:", mapped);
  return mapped;
}

export async function listItemsByArtist(artistId: number): Promise<ItemRow[]> {
  console.log("🎵 [UI->TAURI] listItemsByArtist()", { artistId });
  const rows = await logInvoke<ItemTuple[]>("list_items_by_artist", { artistId });
  console.log("📦 [TAURI] list_items_by_artist raw:", rows);

  // ✅ TIPADO EXPLÍCITO PARA EVITAR WIDENING A string
  const mapped: ItemRow[] = rows.map(([id, title, fullPath, mediaType]) => ({
    id,
    title,
    fullPath,
    mediaType: normalizeMediaType(mediaType),
  }));

  console.log("✅ [UI] listItemsByArtist mapped:", mapped);
  return mapped;
}

/**
 * Detecta si la app corre dentro de Tauri (desktop)
 */
export const isTauri = (): boolean => {
  return typeof window !== "undefined" && "__TAURI__" in window;
};