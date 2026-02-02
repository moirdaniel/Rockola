export type MediaType = "video" | "audio";

export type Artist = {
  id: number;
  displayName: string;
};

export type ItemRow = {
  id: number;
  title: string;
  fullPath: string;
  mediaType: MediaType;
};

export type QueueItem = {
  id: number;
  title: string;
  fullPath: string;
  mediaType: MediaType;
  artistName: string;
};
