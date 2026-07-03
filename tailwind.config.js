/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      spacing: {
        '4.5': '1.125rem',
        '18': '4.5rem',
      },
      borderRadius: {
        '3xl': '1.25rem',
        '4xl': '1.5rem',
        '5xl': '2rem',
      },
      colors: {
        surface: {
          DEFAULT: '#0c0b0a',
          card: '#0d0d0f',
          sidebar: '#0c0c0e',
          input: '#08080a',
          elevated: '#141210',
        },
        accent: {
          amber: '#f59e0b',
          blue: '#3b82f6',
          emerald: '#10b981',
          purple: '#a855f7',
          rose: '#f43f5e',
          cyan: '#06b6d4',
        },
      },
      boxShadow: {
        'card': '0 4px 24px -1px rgba(0,0,0,0.2)',
        'btn-glow': '0 0 40px rgba(245,158,11,0.25)',
        'input-glow': '0 0 20px rgba(245,158,11,0.05)',
        'blue-glow': '0 0 15px rgba(59,130,246,0.8)',
        'amber-glow': '0 0 15px rgba(245,158,11,0.1)',
      },
    },
  },
  plugins: [],
}