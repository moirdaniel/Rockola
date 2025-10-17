import fs from 'node:fs';
import path from 'node:path';
import mm from 'music-metadata';
import type { Track } from '@shared/types';

const SUP_EXT = new Set(['.mp3', '.flac', '.wav', '.m4a', '.aac', '.ogg', '.mp4', '.mkv', '.webm', '.avi', '.mov']);

async function* walk(dir: string): AsyncGenerator<string> {
  const entries = await fs.promises.readdir(dir, { withFileTypes: true });
  for (const e of entries) {
    const p = path.join(dir, e.name);
    if (e.isDirectory()) {
      yield* walk(p);
    } else if (e.isFile()) {
      const ext = path.extname(e.name).toLowerCase();
      if (SUP_EXT.has(ext)) yield p;
    }
  }
}

function guessArtistAndTitle(filePath: string) {
  const base = path.basename(filePath, path.extname(filePath));
  const parts = base.split(' - ');
  if (parts.length >= 2) return { artist: parts[0], title: parts.slice(1).join(' - ') };
  const dir = path.basename(path.dirname(filePath));
  return { artist: dir, title: base };
}

async function processFile(fp: string) {
  let artist = '';
  let title = '';
  let durationSec: number | undefined;
  let type: 'local-audio' | 'local-video' = 'local-audio';
  try {
    const meta = await mm.parseFile(fp, { duration: true });
    artist = meta.common.artist || '';
    title = meta.common.title || '';
    durationSec = meta.format.duration ? Math.round(meta.format.duration) : undefined;
    const v = (meta.format.numberOfVideoTracks ?? 0) > 0;
    type = v ? 'local-video' : 'local-audio';
  } catch {
    const g = guessArtistAndTitle(fp);
    artist = g.artist;
    title = g.title;
    const ext = path.extname(fp).toLowerCase();
    type = ['.mp4', '.mkv', '.webm', '.avi', '.mov'].includes(ext) ? 'local-video' : 'local-audio';
  }

  if (!artist || !title) {
    const g = guessArtistAndTitle(fp);
    artist ||= g.artist;
    title ||= g.title;
  }

  const rec: Track = {
    id: 0,
    artistId: null,
    artistName: artist.trim() || 'Desconocido',
    title: title.trim() || path.basename(fp),
    type,
    pathOrId: fp,
    durationSec
  };
  process.stdout.write(JSON.stringify(rec) + '\n');
}

async function main() {
  const arg = process.argv[2];
  const dirs: string[] = arg ? JSON.parse(arg) : [];
  for (const dir of dirs) {
    try {
      for await (const file of walk(dir)) await processFile(file);
    } catch (e) {
      console.error('walk error', dir, e);
    }
  }
}

main().catch(err => { console.error(err); process.exit(1); });
