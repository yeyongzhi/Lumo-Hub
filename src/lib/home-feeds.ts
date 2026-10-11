export type FeedKind = 'weather' | 'hitokoto' | 'news' | 'poetry'
export interface FeedData {
  content: string
  source: string
  author: string
  date: string
  news: string[]
  location: string
  temperature: string
  feelsLike: string
  humidity: string
  wind: string
}
const object = (value: unknown): Record<string, unknown> => value && typeof value === 'object' ? value as Record<string, unknown> : {}
const string = (value: unknown) => typeof value === 'string' ? value : ''
const first = (value: unknown) => object(Array.isArray(value) ? value[0] : null)
const label = (value: unknown) => string(first(value).value)
export function parseFeed(kind: FeedKind, raw: unknown): FeedData {
  const value = object(raw)
  const result: FeedData = { content: '', source: '', author: '', date: '', news: [], location: '', temperature: '', feelsLike: '', humidity: '', wind: '' }
  if (kind === 'weather') {
    const current = first(value.current_condition)
    const area = first(value.nearest_area)
    result.temperature = string(current.temp_C)
    result.feelsLike = string(current.FeelsLikeC)
    result.humidity = string(current.humidity)
    result.wind = string(current.windspeedKmph)
    result.location = label(area.areaName)
    const codes: Record<string, string> = { '113': '晴', '116': '多云', '119': '阴', '122': '阴', '143': '薄雾', '176': '局部阵雨', '179': '局部降雪', '200': '雷阵雨', '248': '雾', '260': '冻雾', '296': '小雨', '302': '中雨', '308': '大雨', '353': '小阵雨', '356': '阵雨', '389': '雷雨', '338': '大雪' }
    result.content = codes[string(current.weatherCode)] || label(current.lang_zh) || label(current.weatherDesc)
    if (!result.temperature || !result.content) throw new Error('天气服务暂未返回有效数据')
  } else if (kind === 'news') {
    if (value.code !== 200) throw new Error('新闻服务暂不可用')
    const data = object(value.data)
    result.news = Array.isArray(data.news) ? data.news.filter((line): line is string => typeof line === 'string' && !!line.trim()) : []
    result.date = string(data.date)
    result.content = string(data.tip)
    result.source = '每日 60s'
    if (!result.news.length) throw new Error('暂未获取到新闻内容')
  } else {
    result.content = string(kind === 'hitokoto' ? value.hitokoto : value.content)
    result.source = string(kind === 'hitokoto' ? value.from : value.origin)
    result.author = string(kind === 'hitokoto' ? value.from_who : value.author)
    if (!result.content) throw new Error('服务暂未返回有效内容')
  }
  return result
}
