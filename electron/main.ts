import 'dotenv/config';
import path from 'node:path';
import { app, BrowserWindow, ipcMain, shell } from 'electron';
import isDev from 'electron-is-dev';
import Database from 'better-sqlite3';
import { spawn } from 'node:child_process';
import ytdl from 'ytdl-core';
import ytSearch from 'yt-search';
import type { Artist, Track, YouTubeResult, SpotifyArtistResult, SpotifyTrackResult } from '@shared/types';

let win: BrowserWindow | null = null;

// --- DB Bootstrap ---
const db = new Database(path.join(app.getPath('userData'), 'library.db'));
db.pragma('journal_mode = WAL');

db.exec(`
CREATE TABLE IF NOT EXISTS artists (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  name TEXT UNIQUE NOT NULL,
  coverPath TEXT
);
CREATE TABLE IF NOT EXISTS tracks (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  artistId INTEGER,
  artistName TEXT NOT NULL,
  title TEXT NOT NULL,
  type TEXT NOT NULL,
  pathOrId TEXT NOT NULL,
  durationSec INTEGER,
  FOREIGN KEY (artistId) REFERENCES artists(id)
);
CREATE INDEX IF NOT EXISTS idx_tracks_artistName ON tracks(artistName);
CREATE TABLE IF NOT EXISTS settings (
  key TEXT PRIMARY KEY,
  value TEXT NOT NULL
);
`);

// --- Helpers DB ---
const upsertArtist = db.prepare(`INSERT OR IGNORE INTO artists (name) VALUES (?)`);
const getArtistByName = db.prepare(`SELECT * FROM artists WHERE name = ?`);
const insertTrack = db.prepare(`INSERT INTO tracks (artistId, artistName, title, type, pathOrId, durationSec) VALUES (?,?,?,?,?,?)`);
const findArtists = db.prepare(`SELECT * FROM artists WHERE name LIKE ? ORDER BY name LIMIT 100`);
const getTracksByArtist = db.prepare(`SELECT * FROM tracks WHERE artistName = ? ORDER BY title`);
const setSettingStmt = db.prepare(`INSERT INTO settings(key,value) VALUES(?,?) ON CONFLICT(key) DO UPDATE SET value=excluded.value`);
const getSettingStmt = db.prepare(`SELECT value FROM settings WHERE key = ?`);

// --- Ventana principal ---
async function createWindow() {
  win = new BrowserWindow({
    width: 1280,
    height: 800,
    backgroundColor: '#0b0b0f',
    webPreferences: {
      preload: path.join(__dirname, 'preload.cjs'),
      nodeIntegration: false,
      contextIsolation: true,
      sandbox: false,
      webSecurity: true,
      allowRunningInsecureContent: false
    },
    title: 'Rockola'
  });

  const url = isDev && process.env.VITE_DEV_SERVER_URL
    ? process.env.VITE_DEV_SERVER_URL
    : `file://${path.join(__dirname, '../dist/index.html')}`;

  await win.loadURL(url);

  if (isDev) {
    win.webContents.openDevTools({ mode: 'detach' });
  }
}

app.whenReady().then(createWindow);

app.on('window-all-closed', () => {
  if (process.platform !== 'darwin') app.quit();
});

app.on('activate', () => {
  if (BrowserWindow.getAllWindows().length === 0) createWindow();
});

// --- Librería local: Indexación ---
ipcMain.handle('library:scan', async (_evt, dirs: string[]) => {
  return new Promise((resolve, reject) => {
    const indexerPath = isDev
      ? path.join(process.cwd(), 'node', 'indexer.ts')
      : path.join(process.resourcesPath, 'node', 'indexer.js');

    const child = spawn(process.execPath, [
      isDev ? '-r' : '',
      isDev ? 'tsx' : '',
      indexerPath,
      JSON.stringify(dirs)
    ].filter((segment): segment is string => Boolean(segment)), {
      env: process.env,
      stdio: ['ignore', 'pipe', 'pipe']
    });

    const insertTrackStmt = db.prepare(`
      INSERT INTO tracks (artistId, artistName, title, type, pathOrId, durationSec)
      VALUES (?,?,?,?,?,?)
    `);
    const deleteTracksStmt = db.prepare('DELETE FROM tracks');
    const deleteArtistsStmt = db.prepare('DELETE FROM artists');

    const transaction = db.transaction(() => {
      deleteTracksStmt.run();
      deleteArtistsStmt.run();
    });

    transaction();

    child.stdout.on('data', (chunk) => {
      const lines = chunk.toString().split('\n').filter(Boolean);
      for (const line of lines) {
        try {
          const rec: Track = JSON.parse(line);
          upsertArtist.run(rec.artistName);
          const artist = getArtistByName.get(rec.artistName) as Artist | undefined;
          insertTrackStmt.run(artist?.id ?? null, rec.artistName, rec.title, rec.type, rec.pathOrId, rec.durationSec ?? null);
        } catch (e) {
          console.warn('parse/index error', e);
        }
      }
    });

    child.stderr.on('data', (d) => console.error('[indexer]', d.toString()));
    child.on('close', (code) => {
      if (code === 0) resolve(true);
      else reject(new Error('indexer exit code ' + code));
    });
  });
});

ipcMain.handle('artists:find', async (_e, q: string) => {
  const rows = findArtists.all(`%${q}%`) as Artist[];
  return rows;
});

ipcMain.handle('artist:tracks', async (_e, artistName: string) => {
  const rows = getTracksByArtist.all(artistName) as Track[];
  return rows;
});

ipcMain.handle('settings:get', async (_e, key: string) => {
  const row = getSettingStmt.get(key) as { value: string } | undefined;
  return row?.value ?? null;
});

ipcMain.handle('settings:set', async (_e, key: string, value: string) => {
  setSettingStmt.run(key, value);
  return true;
});

ipcMain.handle('youtube:search', async (_e, query: string): Promise<YouTubeResult[]> => {
  const res = await ytSearch(query);
  const vids = (res.videos || []).slice(0, 15).map(v => ({
    videoId: v.videoId,
    title: v.title,
    author: v.author?.name ?? '',
    durationSec: v.seconds,
    thumbnail: v.thumbnail
  }));
  return vids;
});

ipcMain.handle('youtube:streamUrl', async (_e, videoId: string) => {
  const info = await ytdl.getInfo(videoId);
  const progressive = ytdl.chooseFormat(info.formats, { quality: 'highest', filter: 'audioandvideo' });
  if (progressive && progressive.url) return { url: progressive.url, mimeType: progressive.mimeType };
  const audio = ytdl.chooseFormat(info.formats, { quality: 'highestaudio' });
  return { url: audio.url, mimeType: audio.mimeType };
});

async function getSpotifyToken() {
  const id = process.env.SPOTIFY_CLIENT_ID;
  const secret = process.env.SPOTIFY_CLIENT_SECRET;
  if (!id || !secret) return null;
  const body = new URLSearchParams({ grant_type: 'client_credentials' });
  const basic = Buffer.from(`${id}:${secret}`).toString('base64');
  const resp = await fetch('https://accounts.spotify.com/api/token', {
    method: 'POST',
    headers: { 'Authorization': `Basic ${basic}`, 'Content-Type': 'application/x-www-form-urlencoded' },
    body
  });
  if (!resp.ok) return null;
  const json = await resp.json();
  return json.access_token as string;
}

ipcMain.handle('spotify:searchArtists', async (_e, query: string): Promise<SpotifyArtistResult[]> => {
  const token = await getSpotifyToken();
  if (!token) return [];
  const resp = await fetch(`https://api.spotify.com/v1/search?type=artist&limit=15&q=${encodeURIComponent(query)}`, {
    headers: { 'Authorization': `Bearer ${token}` }
  });
  const json = await resp.json();
  const artists = (json.artists?.items ?? []).map((a: any) => ({
    id: a.id,
    name: a.name,
    genres: a.genres ?? [],
    image: a.images?.[0]?.url
  }));
  return artists;
});

ipcMain.handle('spotify:artistTopTracks', async (_e, artistId: string): Promise<SpotifyTrackResult[]> => {
  const token = await getSpotifyToken();
  if (!token) return [];
  const resp = await fetch(`https://api.spotify.com/v1/artists/${artistId}/top-tracks?market=US`, {
    headers: { 'Authorization': `Bearer ${token}` }
  });
  const json = await resp.json();
  return (json.tracks ?? []).map((t: any) => ({
    id: t.id,
    name: t.name,
    artists: t.artists?.map((x: any) => x.name) ?? [],
    durationMs: t.duration_ms,
    previewUrl: t.preview_url
  }));
});

ipcMain.on('open-external', (_e, url: string) => shell.openExternal(url));
