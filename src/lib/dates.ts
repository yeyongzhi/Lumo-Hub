import { differenceInCalendarDays, format, isBefore, parseISO, startOfDay } from 'date-fns'
import { zhCN } from 'date-fns/locale'
import type { Countdown } from './types'

export function countdownTarget(item: Countdown, now = new Date()) {
  let target = parseISO(item.target_date)
  const today = startOfDay(now)
  if (item.repeat_type === 'yearly') {
    const month = target.getMonth(), day = target.getDate()
    const annual = (year: number) => new Date(year, month, Math.min(day, new Date(year, month + 1, 0).getDate()))
    target = annual(today.getFullYear())
    if (isBefore(target, today)) target = annual(today.getFullYear() + 1)
  }
  return target
}

export function countdownStatus(item: Countdown, now = new Date()) {
  const target = countdownTarget(item, now)
  const today = startOfDay(now)
  const days = differenceInCalendarDays(startOfDay(target), today)
  if (item.repeat_type === 'once' && days < 0) return { days: Math.abs(days), text: `已过去 ${Math.abs(days)} 天`, past: true }
  if (days === 0) return { days: 0, text: '就是今天', past: false }
  return { days, text: `还有 ${days} 天`, past: false }
}

export function formatLongDate(date = new Date()) {
  return format(date, 'yyyy 年 M 月 d 日 · EEEE', { locale: zhCN })
}
