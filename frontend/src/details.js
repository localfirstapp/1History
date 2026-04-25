import './style.css'
import { initTheme, toggleTheme } from './theme.js'

const theme = initTheme()
const themeBtn = document.getElementById('theme-toggle')
themeBtn.textContent = theme === 'dark' ? '☀️' : '🌙'
themeBtn.addEventListener('click', () => {
  const next = toggleTheme()
  themeBtn.textContent = next === 'dark' ? '☀️' : '🌙'
})

document.getElementById('submit').addEventListener('click', () => {
  const kw = document.getElementById('keyword').value
  const ymd = window.location.pathname.split('/').pop()
  window.location = `/details/${ymd}?keyword=${encodeURIComponent(kw)}`
})
document.getElementById('keyword').addEventListener('keypress', (e) => {
  if (e.key === 'Enter') document.getElementById('submit').click()
})
