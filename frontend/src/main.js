import './style.css'
import { initTheme, toggleTheme } from './theme.js'
import { initDailyChart, initPieChart } from './charts.js'

const d = window.__SERVER_DATA__

const theme = initTheme()
const themeBtn = document.getElementById('theme-toggle')
themeBtn.textContent = theme === 'dark' ? '☀️' : '🌙'
themeBtn.addEventListener('click', () => {
  const next = toggleTheme()
  themeBtn.textContent = next === 'dark' ? '☀️' : '🌙'
})

document.getElementById('kpi-total').textContent = d.stats.total_visits.toLocaleString()
document.getElementById('kpi-domains').textContent = d.stats.unique_domains.toLocaleString()
document.getElementById('kpi-days').textContent = d.stats.active_days.toLocaleString()
document.getElementById('kpi-today').textContent = d.stats.today_visits.toLocaleString()

const startInput = document.getElementById('date-start')
const endInput = document.getElementById('date-end')
startInput.value = d.startYmd
endInput.value = d.endYmd
startInput.min = endInput.min = new Date(d.minTime).toISOString().slice(0, 10)
startInput.max = endInput.max = new Date(d.maxTime).toISOString().slice(0, 10)

document.getElementById('keyword').value = d.keyword
document.getElementById('footer-version').textContent = d.version
function doSearch() {
  const kw = document.getElementById('keyword').value
  const start = startInput.value || d.startYmd
  const end = endInput.value || d.endYmd
  window.location = `/?start=${start}&end=${end}&keyword=${encodeURIComponent(kw)}`
}

document.getElementById('submit').addEventListener('click', doSearch)
document.getElementById('keyword').addEventListener('keypress', (e) => {
  if (e.key === 'Enter') doSearch()
})

const listBtn = document.getElementById('view-list')
listBtn.href = `/search?start=${d.startYmd}&end=${d.endYmd}${d.keyword ? '&keyword=' + encodeURIComponent(d.keyword) : ''}`
document.getElementById('view-list-count').textContent = d.visitCount.toLocaleString()

initDailyChart(
  document.getElementById('daily-chart'),
  d.dailyCounts,
  (ymd) => window.open(`/details/${ymd}?keyword=${encodeURIComponent(d.keyword)}`, '_blank')
)
initPieChart(document.getElementById('title-pie'), d.titleTop100, 'Top 10 Pages', null)
initPieChart(
  document.getElementById('domain-pie'),
  d.domainTop100,
  'Top 10 Domains',
  (domain) => {
    document.getElementById('keyword').value = domain
    doSearch()
  }
)

function fillTable(tbodyId, rows) {
  const tbody = document.getElementById(tbodyId)
  tbody.innerHTML = rows.map(([name, cnt]) =>
    `<tr><td>${cnt}</td><td>${name}</td></tr>`
  ).join('')
}
fillTable('title-table', d.titleTop100)
fillTable('domain-table', d.domainTop100)
