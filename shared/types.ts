export type MediaType = 'local-video' | 'local-audio' | 'youtube' | 'spotify';

export interface Artist {
  id: number;
  name: string;
  coverPath?: string | null;
}

export interface Track {
  id: number;
  artistId?: number | null;
  artistName: string;
  title: string;
  type: MediaType;
  pathOrId: string;
  durationSec?: number | null;
}

export interface YouTubeResult {
  videoId: string;
  title: string;
  author: string;
  durationSec?: number;
  thumbnail?: string;
}

export interface SpotifyArtistResult {
  id: string;
  name: string;
  genres: string[];
  image?: string;
}

export interface SpotifyTrackResult {
  id: string;
  name: string;
  artists: string[];
  durationMs?: number;
  previewUrl?: string | null;
}
