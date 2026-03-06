import { Routes, Route } from 'react-router-dom'
import { Layout } from '@/components/Layout'
import { Home } from '@/pages/Home'
import { NowPlaying } from '@/pages/NowPlaying'
import { Queue } from '@/pages/Queue'
import { Credits } from '@/pages/Credits'
import { Admin } from '@/pages/Admin'
import { Mantenedores } from '@/pages/Mantenedores'
import { TvNowPlaying } from '@/pages/TvNowPlaying'
import { DisplayConfig } from '@/pages/DisplayConfig'

export default function App() {
  return (
    <Routes>
      <Route element={<Layout />}>
        <Route path="/" element={<Home />} />
        <Route path="/now-playing" element={<NowPlaying />} />
        <Route path="/queue" element={<Queue />} />
        <Route path="/credits" element={<Credits />} />
        <Route path="/admin" element={<Admin />} />
        <Route path="/mantenedores" element={<Mantenedores />} />
        <Route path="/tv" element={<TvNowPlaying />} />
        <Route path="/pantallas" element={<DisplayConfig />} />
      </Route>
    </Routes>
  )
}
