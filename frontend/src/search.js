import './style.css'
import { Virtualizer, observeElementRect, observeElementOffset, elementScroll } from '@tanstack/virtual-core'
import { initTheme, toggleTheme } from './theme.js'

const d = window.__SERVER_DATA__

const theme = initTheme()
const themeBtn = document.getElementById('theme-toggle')
themeBtn.textContent = theme === 'dark' ? '☀️' : '🌙'
themeBtn.addEventListener('click', () => {
  const next = toggleTheme()
  themeBtn.textContent = next === 'dark' ? '☀️' : '🌙'
})

document.getElementById('filter-summary').textContent =
  `${d.startYmd} – ${d.endYmd}${d.keyword ? ' · "' + d.keyword + '"' : ''}`

let allVisits = d.visits
let filtered = allVisits

const filterInput = document.getElementById('filter-input')
const countEl = document.getElementById('result-count')

function updateCount() {
  countEl.textContent = `${filtered.length.toLocaleString()} records`
}
updateCount()

filterInput.addEventListener('input', () => {
  const q = filterInput.value.toLowerCase()
  filtered = q
    ? allVisits.filter(v => v.url.toLowerCase().includes(q) || (v.title || '').toLowerCase().includes(q))
    : allVisits
  updateCount()
  virtualizer.setOptions({ count: filtered.length })
  renderRows()
})

const ROW_HEIGHT = 36
const container = document.getElementById('virtual-container')
const totalHeightEl = document.getElementById('virtual-total-height')
const rowsEl = document.getElementById('virtual-rows')

const virtualizer = new Virtualizer({
  count: filtered.length,
  getScrollElement: () => container,
  estimateSize: () => ROW_HEIGHT,
  overscan: 10,
  scrollMargin: 0,
  observeElementRect,
  observeElementOffset,
  scrollToFn: elementScroll,
  onChange: () => renderRows(),
})

function formatDatetime(ms) {
  return new Date(ms * 1000).toLocaleString(undefined, {
    year: 'numeric', month: '2-digit', day: '2-digit',
    hour: '2-digit', minute: '2-digit', second: '2-digit',
  })
}

function esc(str) {
  return String(str).replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;')
}

function renderRows() {
  const items = virtualizer.getVirtualItems()
  totalHeightEl.style.height = virtualizer.getTotalSize() + 'px'
  rowsEl.style.transform = items.length ? `translateY(${items[0].start}px)` : 'translateY(0)'
  rowsEl.innerHTML = items.map(item => {
    const v = filtered[item.index]
    const title = v.title || v.url
    return `<div style="display:grid;grid-template-columns:160px 1fr;padding:8px 16px;border-bottom:1px solid var(--border);height:${ROW_HEIGHT}px;align-items:center;box-sizing:border-box">
      <span style="font-size:12px;color:var(--text-muted)">${formatDatetime(v.visit_time)}</span>
      <a href="${esc(v.url)}" style="font-size:13px;overflow:hidden;text-overflow:ellipsis;white-space:nowrap" title="${esc(v.title)}">${esc(title)}</a>
    </div>`
  }).join('')
}

virtualizer._didMount()
virtualizer._willUpdate()
renderRows()

container.addEventListener('scroll', () => {
  virtualizer._willUpdate()
  renderRows()
})
