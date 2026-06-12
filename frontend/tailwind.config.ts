import type { Config } from 'tailwindcss'
const config: Config = {
  content: ['./index.html','./src/**/*.{ts,tsx}'],
  darkMode: 'class',
  theme: {
    extend: {
      fontFamily: { mono: ['JetBrains Mono','Fira Code','Menlo','monospace'] },
      colors: {
        sf: {
          bg:      '#0d1117', surface: '#161b22', border:  '#30363d',
          muted:   '#8b949e', accent:  '#7c3aed', accent2: '#a78bfa',
        },
      },
    },
  },
  plugins: [],
}
export default config
