/** @type {import('tailwindcss').Config} */
export default {
  content: [
    './src/**/*.{ts,tsx}',
    './index.html'
  ],
  theme: {
    extend: {
      colors: {
        bg: '#0b0b0f',
        card: '#11131a',
        border: '#232637',
        text: '#e6e6e9',
        muted: '#9aa0a6'
      },
      borderRadius: {
        xl: '14px',
        '2xl': '18px'
      },
      boxShadow: {
        soft: '0 6px 20px rgba(0,0,0,.25)'
      }
    }
  },
  plugins: []
};
