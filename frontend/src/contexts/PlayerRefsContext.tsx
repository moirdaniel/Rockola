import { createContext, useContext, useMemo, useRef, type RefObject } from 'react'
import { usePlayerStore } from '@/stores/playerStore'

export interface PlayerRefs {
  audioRef: RefObject<HTMLAudioElement | null>
  videoRef: RefObject<HTMLVideoElement | null>
}

const PlayerRefsContext = createContext<PlayerRefs | null>(null)

export function usePlayerRefs(): PlayerRefs {
  const ctx = useContext(PlayerRefsContext)
  if (!ctx) throw new Error('usePlayerRefs must be used within PlayerRefsProvider')
  return ctx
}

interface PlayerRefsProviderProps {
  pathname: string
  children: React.ReactNode
}

export function PlayerRefsProvider({ pathname, children }: PlayerRefsProviderProps) {
  const audioRef = useRef<HTMLAudioElement>(null)
  const videoRef = useRef<HTMLVideoElement>(null)
  const currentItem = usePlayerStore((s) => s.currentItem)
  const value = useMemo(() => ({ audioRef, videoRef }), [])
  const showFullscreen = pathname === '/now-playing' || pathname === '/tv'

  return (
    <PlayerRefsContext.Provider value={value}>
      <audio ref={audioRef} className="hidden" />
      {showFullscreen ? (
        <div className="fixed inset-0 z-0 flex items-center justify-center bg-black">
          {currentItem?.type === 'video' ? (
            <video
              ref={videoRef}
              className="w-full h-full object-contain"
              playsInline
            />
          ) : currentItem?.thumbnailUrl ? (
            <img
              src={currentItem.thumbnailUrl}
              alt=""
              className="w-full h-full object-contain"
            />
          ) : (
            <div className="w-full h-full flex items-center justify-center text-white/50">
              {currentItem ? currentItem.title : 'Sin portada'}
            </div>
          )}
        </div>
      ) : (
        <div className="hidden">
          <video ref={videoRef} playsInline />
        </div>
      )}
      {children}
    </PlayerRefsContext.Provider>
  )
}
