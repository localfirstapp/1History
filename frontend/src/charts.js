import * as echarts from 'echarts/core'
import { LineChart, PieChart } from 'echarts/charts'
import {
  TitleComponent, TooltipComponent, LegendComponent,
  GridComponent, DataZoomComponent, ToolboxComponent,
} from 'echarts/components'
import { CanvasRenderer } from 'echarts/renderers'

echarts.use([
  LineChart, PieChart,
  TitleComponent, TooltipComponent, LegendComponent,
  GridComponent, DataZoomComponent, ToolboxComponent,
  CanvasRenderer,
])

function getAccentColor() {
  return getComputedStyle(document.documentElement)
    .getPropertyValue('--accent').trim() || '#4A90D9'
}

export function initDailyChart(el, data, onClickDate) {
  const chart = echarts.init(el)
  chart.setOption({
    color: [getAccentColor()],
    title: { text: 'Daily PV', subtext: 'Click any point to view details' },
    tooltip: {
      trigger: 'axis',
      formatter: (params) => {
        const d = new Date(params[0].value[0])
        return `${d.toLocaleDateString()}<br/>PV: ${params[0].value[1]}`
      },
    },
    toolbox: { feature: { saveAsImage: { show: true }, dataView: { show: true, readOnly: true } } },
    dataZoom: [{ type: 'inside' }, { type: 'slider' }],
    xAxis: { type: 'time' },
    yAxis: { name: 'PV', type: 'value' },
    series: [{
      name: 'Page View',
      type: 'line',
      showAllSymbol: 'auto',
      data: data.map(([ts, cnt]) => [new Date(ts), cnt]),
    }],
  })
  chart.on('click', (params) => {
    const d = new Date(params.value[0])
    const ymd = d.toISOString().slice(0, 10)
    onClickDate(ymd)
  })
  const onResize = () => chart.resize()
  window.addEventListener('resize', onResize)
  return chart
}

export function initPieChart(el, data, title, onClickItem) {
  const chart = echarts.init(el)
  const top10 = data.slice(0, 10).map(([name, value]) => ({
    name: name.length > 50 ? name.slice(0, 50) + '…' : name,
    value,
  }))
  chart.setOption({
    title: { text: title, left: 'center' },
    tooltip: {
      trigger: 'item',
      formatter: (params) => `${params.name}<br/>${params.value} (${params.percent}%)`,
      confine: true,
    },
    legend: {
      orient: 'vertical',
      left: '48%',
      top: 'middle',
      formatter: (name) => name.length > 60 ? name.slice(0, 60) + '…' : name,
    },
    series: [{
      name: title,
      type: 'pie',
      radius: ['30%', '65%'],
      center: ['24%', '55%'],
      data: top10,
      label: { show: false },
      labelLine: { show: false },
      emphasis: { itemStyle: { shadowBlur: 6, shadowOffsetX: 0, shadowColor: 'rgba(0,0,0,0.3)' } },
    }],
  })
  if (onClickItem) {
    chart.on('click', (params) => onClickItem(params.name))
  }
  const onResize = () => chart.resize()
  window.addEventListener('resize', onResize)
  return chart
}
