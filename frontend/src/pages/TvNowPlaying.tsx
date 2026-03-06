import { NowPlaying } from '@/pages/NowPlaying'

export function TvNowPlaying() {
  return (
    <div className="min-h-screen bg-jukebox-dark flex items-center justify-center">
      <div className="w-full max-w-4xl">
        <NowPlaying />
      </div>
    </div>
  )
}

