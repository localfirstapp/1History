import './style.css'
import flatpickr from 'flatpickr'
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

const fp = flatpickr('#date-range', {
  mode: 'range',
  dateFormat: 'Y-m-d',
  defaultDate: [d.startYmd, d.endYmd],
  minDate: new Date(d.minTime),
  maxDate: new Date(d.maxTime),
})

document.getElementById('keyword').value = d.keyword

function doSearch() {
  const kw = document.getElementById('keyword').value
  const dates = fp.selectedDates
  const fmt = (dt) => dt.toISOString().slice(0, 10)
  const start = dates.length >= 1 ? fmt(dates[0]) : d.startYmd
  const end   = dates.length >= 2 ? fmt(dates[1]) : d.endYmd
  window.location = `/?start=${start}&end=${end}&keyword=${encodeURIComponent(kw)}`
}

document.getElementById('submit').addEventListener('click', doSearch)
document.getElementById('keyword').addEventListener('keypress', (e) => {
  if (e.key === 'Enter') doSearch()
})

const listBtn = document.getElementById('view-list')
const listParams = `start=${d.startYmd}&end=${d.endYmd}${d.keyword ? '&keyword=' + encodeURIComponent(d.keyword) : ''}`
listBtn.href = `/search?${listParams}`
document.getElementById('view-list-count').textContent = d.visitCount.toLocaleString()

initDailyChart(
  document.getElementById('daily-chart'),
  d.dailyCounts,
  (ymd) => window.open(`/details/${ymd}?keyword=${encodeURIComponent(d.keyword)}`, '_blank')
)
initPieChart(document.getElementById('title-pie'), d.titleTop100, 'TOP 10 by Title', null)
initPieChart(
  document.getElementById('domain-pie'),
  d.domainTop100,
  'TOP 10 by Domain',
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
