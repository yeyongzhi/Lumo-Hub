import type { HomeWidget, HomeWidgetKind, HomeWidgetSize } from './types'

export const widgetNames: Record<HomeWidgetKind, string> = { clock: '日期时间', countdowns: '倒数日', sticky: '便签', note: '笔记', recent: '最近打开', weather: '天气', hitokoto: '随机一言', news: '每日 60s 新闻', poetry: '今日诗词' }
export const widgetSizes: HomeWidgetSize[] = ['1x1', '2x1', '1x2', '2x2', '4x1']
export const widgetClasses: Record<HomeWidgetSize, string> = {
  '1x1': 'col-span-1 row-span-1', '2x1': 'col-span-2 row-span-1',
  '1x2': 'col-span-1 row-span-2', '2x2': 'col-span-2 row-span-2',
  '4x1': 'col-span-2 row-span-1 @min-[64rem]:col-span-4',
}
export function defaultHomeLayout(): HomeWidget[] {
  return [
    { id: 'clock', kind: 'clock', size: '2x1' }, { id: 'countdowns', kind: 'countdowns', size: '2x1' },
    { id: 'sticky', kind: 'sticky', size: '1x1' }, { id: 'note', kind: 'note', size: '1x1' },
    { id: 'recent', kind: 'recent', size: '2x1' },

  ]
}
export function homeTemplate(name:'simple'|'work'|'life'):HomeWidget[] {
  const kinds:HomeWidgetKind[]=name==='simple'?['clock','recent','sticky']:name==='work'?['clock','recent','note','sticky','countdowns']:['clock','countdowns','weather','hitokoto','poetry','news']
  return kinds.map(kind=>({id:crypto.randomUUID(),kind,size:kind==='news'?'2x2':['clock','recent','countdowns','weather','hitokoto','poetry'].includes(kind)?'2x1':'1x1'}))
}
export function moveWidget(layout: HomeWidget[], from: number, to: number): HomeWidget[] {
  if (from < 0 || to < 0 || from >= layout.length || to >= layout.length) return layout
  const next = [...layout]
  next.splice(to, 0, next.splice(from, 1)[0])
  return next
}

export function previewWidgetMove(layout: HomeWidget[], draggedId: string, targetId: string, after: boolean): HomeWidget[] {
  const from = layout.findIndex(widget => widget.id === draggedId)
  const target = layout.findIndex(widget => widget.id === targetId)
  if (from < 0 || target < 0 || from === target) return layout
  let to = target + (after ? 1 : 0)
  if (from < to) to--
  return from === to ? layout : moveWidget(layout, from, to)
}
