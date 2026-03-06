/** @type {import('tailwindcss').Config} */
export default {
  content: ['./index.html', './src/**/*.{js,ts,jsx,tsx}'],
  darkMode: 'class',
  theme: {
    extend: {
      colors: {
        jukebox: {
          primary: '#e11d48',
          secondary: '#7c3aed',
          dark: '#0f0a14',
          panel: '#1a1225',
          card: '#251a35',
        },
      },
      fontFamily: {
        display: ['Orbitron', 'sans-serif'],
        body: ['Outfit', 'sans-serif'],
      },
      animation: {
        'glow-pulse': 'glow-pulse 2s ease-in-out infinite',
        'slide-up': 'slide-up 0.3s ease-out',
      },
      keyframes: {
        'glow-pulse': {
          '0%, 100%': { opacity: '1', boxShadow: '0 0 20px rgba(225, 29, 72, 0.5)' },
          '50%': { opacity: '0.8', boxShadow: '0 0 30px rgba(225, 29, 72, 0.8)' },
        },
        'slide-up': {
          '0%': { transform: 'translateY(10px)', opacity: '0' },
          '100%': { transform: 'translateY(0)', opacity: '1' },
        },
      },
      backdropBlur: {
        xs: '2px',
      },
    },
  },
  plugins: [],
}
