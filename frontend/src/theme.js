const STORAGE_KEY = 'oh-theme'

function safeGet() {
  try { return localStorage.getItem(STORAGE_KEY) } catch { return null }
}

function safeSave(value) {
  try { localStorage.setItem(STORAGE_KEY, value) } catch { /* ignore */ }
}

function getPreferred() {
  const stored = safeGet()
  if (stored === 'light' || stored === 'dark') return stored
  return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light'
}

export function initTheme() {
  const theme = getPreferred()
  document.documentElement.setAttribute('data-theme', theme)
  return theme
}

export function toggleTheme() {
  const current = document.documentElement.getAttribute('data-theme') ?? getPreferred()
  const next = current === 'dark' ? 'light' : 'dark'
  document.documentElement.setAttribute('data-theme', next)
  safeSave(next)
  return next
}
