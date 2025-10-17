import { contextBridge, ipcRenderer } from 'electron';
import type { Artist, Track, YouTubeResult, SpotifyArtistResult, SpotifyTrackResult } from '@shared/types';

declare global {
  interface Window {
    api: {
      scanLibrary: (dirs: string[]) => Promise<boolean>;
      findArtists: (q: string) => Promise<Artist[]>;
      getArtistTracks: (artistName: string) => Promise<Track[]>;
      ytSearch: (q: string) => Promise<YouTubeResult[]>;
      ytStreamUrl: (videoId: string) => Promise<{ url: string; mimeType?: string | null }>;
      spSearchArtists: (q: string) => Promise<SpotifyArtistResult[]>;
      spArtistTopTracks: (artistId: string) => Promise<SpotifyTrackResult[]>;
      getSetting: (key: string) => Promise<string | null>;
      setSetting: (key: string, value: string) => Promise<boolean>;
      openExternal: (url: string) => void;
    }
  }
}

contextBridge.exposeInMainWorld('api', {
  scanLibrary: (dirs: string[]) => ipcRenderer.invoke('library:scan', dirs),
  findArtists: (q: string) => ipcRenderer.invoke('artists:find', q),
  getArtistTracks: (name: string) => ipcRenderer.invoke('artist:tracks', name),
  ytSearch: (q: string) => ipcRenderer.invoke('youtube:search', q),
  ytStreamUrl: (id: string) => ipcRenderer.invoke('youtube:streamUrl', id),
  spSearchArtists: (q: string) => ipcRenderer.invoke('spotify:searchArtists', q),
  spArtistTopTracks: (id: string) => ipcRenderer.invoke('spotify:artistTopTracks', id),
  getSetting: (key: string) => ipcRenderer.invoke('settings:get', key),
  setSetting: (key: string, value: string) => ipcRenderer.invoke('settings:set', key, value),
  openExternal: (url: string) => ipcRenderer.send('open-external', url)
});
