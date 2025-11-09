/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    './pages/**/*.{js,ts,jsx,tsx,mdx}',
    './components/**/*.{js,ts,jsx,tsx,mdx}',
    './app/**/*.{js,ts,jsx,tsx,mdx}',
  ],
  theme: {
    extend: {
      colors: {
        'praxis-bg-primary': '#0a0a0a',
        'praxis-bg-secondary': '#141414',
        'praxis-bg-tertiary': '#1e1e1e',
        'praxis-bg-hover': '#252525',
        'praxis-border': '#2a2a2a',
        'praxis-text-primary': '#e8e8e8',
        'praxis-text-secondary': '#a0a0a0',
        'praxis-text-muted': '#666666',
        'praxis-accent': '#6366f1',
        'praxis-accent-hover': '#5457e0',
        'praxis-success': '#10b981',
        'praxis-warning': '#f59e0b',
        'praxis-error': '#ef4444',
      },
    },
  },
  plugins: [],
}

