import type { HomeWidget, HomeWidgetKind, HomeWidgetSize } from './types';

export const widgetNames: Record<HomeWidgetKind, string> = {
  clock: '日期时间',
  countdowns: '倒数日',
  sticky: '便签',
  note: '笔记',
  recent: '最近打开',
  weather: '天气',
  hitokoto: '随机一言',
  news: '每日 60s 新闻',
  poetry: '今日诗词',
  library: '应用库',
  quick_launch: '快捷启动',
  quick_tools: '快捷工具',
  calendar: '日历',
  time: '时间',
};
export const fixedWidgetSize = (kind: HomeWidgetKind) => kind === 'poetry' || kind === 'hitokoto';
export const normalizeHomeWidget = (widget: HomeWidget): HomeWidget => ({
  ...widget,
  size: fixedWidgetSize(widget.kind) ? '1x1' : widget.size,
  selected_ids: [...(widget.selected_ids ?? [])],
});
export const widgetSizes: HomeWidgetSize[] = ['1x1', '2x1', '1x2', '2x2', '4x1'];
// Each profile defines a square cell and the gap between cells in logical pixels.
export const homeGridProfiles = [
  { minWindowWidth: 0, cell: 100, gap: 8 },
  { minWindowWidth: 960, cell: 140, gap: 10 },
  { minWindowWidth: 1080, cell: 170, gap: 12 },
  { minWindowWidth: 1200, cell: 200, gap: 16 },
] as const;
export type HomeGridProfile = (typeof homeGridProfiles)[number];
export const homeGridTracksPerCell = 6;
export function homeGridProfileForWidth(width: number): HomeGridProfile {
  return (
    [...homeGridProfiles].reverse().find((profile) => width >= profile.minWindowWidth) ??
    homeGridProfiles[0]
  );
}
export function homeGridCellCount(width: number, profile: HomeGridProfile, minCells = 1) {
  return Math.max(
    minCells,
    Math.min(4, Math.floor((width + profile.gap) / (profile.cell + profile.gap))),
  );
}
export function homeGridTrack(profile: HomeGridProfile) {
  return (profile.cell - profile.gap * (homeGridTracksPerCell - 1)) / homeGridTracksPerCell;
}
export function homeWidgetDimensions(
  size: HomeWidgetSize,
  profile: HomeGridProfile = homeGridProfiles[3],
) {
  const [columns, rows] = size.split('x').map(Number) as [number, number];
  return {
    width: columns * profile.cell + (columns - 1) * profile.gap,
    height: rows * profile.cell + (rows - 1) * profile.gap,
  };
}
export function homeWidgetSizeLabel(
  size: HomeWidgetSize,
  profile: HomeGridProfile = homeGridProfiles[3],
) {
  const { width, height } = homeWidgetDimensions(size, profile);
  return `${size.replace('x', ' × ')} · ${width} × ${height} px`;
}

// Each standard cell occupies six of the 24 columns at every window width.
export const widgetClasses: Record<HomeWidgetSize, string> = {
  '1x1': 'col-span-6 row-span-1',
  '2x1': 'col-span-12 row-span-1',
  '1x2': 'col-span-6 row-span-2',
  '2x2': 'col-span-12 row-span-2',
  '4x1': 'col-span-full row-span-1',
};
export const previewWidgetClasses: Record<HomeWidgetSize, string> = {
  '1x1': 'col-span-6 row-span-1',
  '2x1': 'col-span-12 row-span-1',
  '1x2': 'col-span-6 row-span-2',
  '2x2': 'col-span-12 row-span-2',
  '4x1': 'col-span-full row-span-1',
};
export function defaultHomeLayout(): HomeWidget[] {
  return [
    { id: 'clock', kind: 'clock', size: '2x1' },
    { id: 'countdowns', kind: 'countdowns', size: '2x1' },
    { id: 'sticky', kind: 'sticky', size: '1x1' },
    { id: 'note', kind: 'note', size: '1x1' },
    { id: 'recent', kind: 'recent', size: '2x1' },
  ];
}
export function homeTemplate(name: 'simple' | 'work' | 'life'): HomeWidget[] {
  const kinds: HomeWidgetKind[] =
    name === 'simple'
      ? ['clock', 'recent', 'sticky']
      : name === 'work'
        ? ['clock', 'recent', 'note', 'sticky', 'countdowns']
        : ['clock', 'countdowns', 'weather', 'hitokoto', 'poetry', 'news'];
  return kinds.map((kind) => ({
    id: crypto.randomUUID(),
    kind,
    size:
      kind === 'news'
        ? '2x2'
        : fixedWidgetSize(kind)
          ? '1x1'
          : ['clock', 'recent', 'countdowns', 'weather'].includes(kind)
            ? '2x1'
            : '1x1',
  }));
}
export function moveWidget(layout: HomeWidget[], from: number, to: number): HomeWidget[] {
  if (from < 0 || to < 0 || from >= layout.length || to >= layout.length) return layout;
  const next = [...layout];
  next.splice(to, 0, next.splice(from, 1)[0]);
  return next;
}

export function previewWidgetMove(
  layout: HomeWidget[],
  draggedId: string,
  targetId: string,
  after: boolean,
): HomeWidget[] {
  const from = layout.findIndex((widget) => widget.id === draggedId);
  const target = layout.findIndex((widget) => widget.id === targetId);
  if (from < 0 || target < 0 || from === target) return layout;
  let to = target + (after ? 1 : 0);
  if (from < to) to--;
  return from === to ? layout : moveWidget(layout, from, to);
}
