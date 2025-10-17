interface Props {
  nowPlaying: { type: 'local' | 'youtube'; src: string; title: string } | null;
}

export default function Player({ nowPlaying }: Props) {
  if (!nowPlaying) return null;
  const isVideo = nowPlaying.src.match(/\.(mp4|mkv|webm|mov|avi)$/i) || nowPlaying.type === 'youtube';
  return (
    <div className="card fixed left-5 right-5 bottom-5 p-4">
      <div className="mb-2 opacity-90">Reproduciendo: {nowPlaying.title}</div>
      {isVideo ? (
        <video src={nowPlaying.src} controls className="h-auto w-full max-h-[360px] rounded-xl" />
      ) : (
        <audio src={nowPlaying.src} controls className="w-full" />
      )}
    </div>
  );
}
