import { useState, useCallback } from 'react'
import { useQuery } from '@tanstack/react-query'
import { searchMedia } from '@/services/unifiedSearch'
import { useSearchStore } from '@/stores/searchStore'
import { MediaCard } from '@/components/MediaCard'
import { usePlayerStore } from '@/stores/playerStore'

export function Home() {
  const [input, setInput] = useState('')
  const { setSearchResponse, setSearching, setError, results, artists, isSearching } = useSearchStore()
  const { currentItem, isPlaying, currentTime, duration } = usePlayerStore()

  const { refetch, isFetching } = useQuery({
    queryKey: ['search', input],
    queryFn: async () => {
      setSearching(true)
      setError(null)
      const r = await searchMedia(input)
      setSearchResponse(r.artists, r.items)
      return r
    },
    enabled: false,
  })

  const handleSearch = useCallback(() => {
    refetch()
  }, [refetch])

  const handleArtistClick = useCallback(
    async (name: string) => {
      // Muestra el nombre del artista en el buscador y filtra inmediatamente
      setInput(name)
      setSearching(true)
      setError(null)
      try {
        const r = await searchMedia(name)
        const onlyArtistSongs = r.items.filter((item) => item.artist === name)
        setSearchResponse(r.artists, onlyArtistSongs)
      } catch (e) {
        setError(e instanceof Error ? e.message : String(e))
      } finally {
        setSearching(false)
      }
    },
    [setError, setSearchResponse, setSearching]
  )

  const searching = isSearching || isFetching
  const progress = duration > 0 ? (currentTime / duration) * 100 : 0

  return (
    <div className="p-6 max-w-6xl mx-auto">
      <section className="mb-8">
        <h1 className="font-display text-3xl font-bold mb-2 bg-gradient-to-r from-jukebox-primary to-jukebox-secondary bg-clip-text text-transparent">
          Buscar música y videos
        </h1>
        <p className="text-gray-400 mb-4">
          Busca en YouTube (vía Rockola). Escribe y pulsa Buscar; agrega a la cola.
        </p>
        <div className="flex gap-2 flex-wrap">
          <input
            type="search"
            value={input}
            onChange={(e) => setInput(e.target.value)}
            onKeyDown={(e) => e.key === 'Enter' && handleSearch()}
            placeholder="Artista, canción, álbum..."
            className="flex-1 min-w-[200px] px-4 py-3 rounded-xl glass-panel border border-white/10 focus:border-jukebox-primary/50 focus:ring-2 focus:ring-jukebox-primary/20 outline-none transition-all"
          />
          <button
            type="button"
            onClick={handleSearch}
            disabled={searching}
            className="px-6 py-3 rounded-xl bg-jukebox-primary hover:bg-jukebox-primary/90 disabled:opacity-50 font-medium transition-colors"
          >
            {searching ? 'Buscando...' : 'Buscar'}
          </button>
        </div>
      </section>

      {currentItem && (
        <section className="mb-8">
          <h2 className="font-display text-xl font-semibold mb-3">Reproduciendo ahora</h2>
          <div className="glass-panel p-4 flex items-center gap-4 rounded-2xl">
            {currentItem.thumbnailUrl && (
              <img
                src={currentItem.thumbnailUrl}
                alt=""
                className="w-16 h-16 rounded-xl object-cover flex-shrink-0"
              />
            )}
            <div className="flex-1 min-w-0">
              <p className="font-medium truncate">{currentItem.title}</p>
              <p className="text-sm text-gray-400 truncate">
                {currentItem.artist ?? '—'} · {isPlaying ? 'Reproduciendo' : 'Pausado'}
              </p>
              <div className="mt-2 h-1.5 rounded-full bg-white/10 overflow-hidden">
                <div
                  className="h-full bg-jukebox-primary rounded-full transition-all duration-300"
                  style={{ width: `${progress}%` }}
                />
              </div>
            </div>
          </div>
        </section>
      )}

      {artists.length > 0 && (
        <section className="mb-8">
          <h2 className="font-display text-xl font-semibold mb-4">Artistas</h2>
          <div className="flex gap-4 overflow-x-auto pb-2 -mx-1 px-1">
            {artists.map((artist) => (
              <button
                key={artist.id}
                type="button"
                onClick={() => handleArtistClick(artist.name)}
                className="flex-shrink-0 flex flex-col items-center gap-2 w-24 group focus:outline-none focus:ring-2 focus:ring-jukebox-primary rounded-xl"
              >
                <div className="w-20 h-20 rounded-full overflow-hidden bg-jukebox-card border-2 border-white/10 group-hover:border-jukebox-primary/50 transition-colors">
                  {artist.thumbnailUrl ? (
                    <img
                      src={artist.thumbnailUrl}
                      alt=""
                      className="w-full h-full object-cover"
                    />
                  ) : (
                    <div className="w-full h-full flex items-center justify-center text-2xl text-gray-500">
                      🎤
                    </div>
                  )}
                </div>
                <span className="text-sm text-gray-300 truncate w-full text-center group-hover:text-jukebox-primary transition-colors">
                  {artist.name}
                </span>
              </button>
            ))}
          </div>
        </section>
      )}

      <section>
        <h2 className="font-display text-xl font-semibold mb-4">Canciones</h2>
        {results.length === 0 && !searching && (
          <p className="text-gray-500 py-8 text-center">
            Escribe algo y pulsa Buscar, o prueba &quot;Queen&quot;, &quot;Eagles&quot; o &quot;Weeknd&quot;.
          </p>
        )}
        <div className="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 gap-4">
          {results.map((item) => (
            <MediaCard key={`${item.source}-${item.id}`} item={item} />
          ))}
        </div>
      </section>
    </div>
  )
}
