module.exports = {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        fusion: {
          DEFAULT: '#00fff7',
          dark: '#0f172a',
          neon: '#00ffe7',
          chrome: '#b8c6db',
        }
      },
      boxShadow: {
        'fusion-glow': '0 0 40px #00fff7, 0 0 80px #00ffe7',
      },
      fontFamily: {
        fusion: ['Orbitron', 'sans-serif'],
      },
    },
  },
  plugins: [],
}
