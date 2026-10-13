import test from 'node:test'
import assert from 'node:assert/strict'
import { countdownTarget, countdownStatus } from '../src/lib/dates.ts'
import { targetKey, stableResourceOrder } from '../src/lib/library-preferences.ts'
import { parseFeed } from '../src/lib/home-feeds.ts'
const date = value => new Date(`${value}T12:00:00`)
const event = (target_date, repeat_type='yearly') => ({target_date,repeat_type})
const day = value => `${value.getFullYear()}-${String(value.getMonth()+1).padStart(2,'0')}-${String(value.getDate()).padStart(2,'0')}`
test('leap anniversaries use February 28 in ordinary years and return to February 29',()=>{
  assert.equal(day(countdownTarget(event('2024-02-29'),date('2026-02-01'))),'2026-02-28')
  assert.equal(day(countdownTarget(event('2024-02-29'),date('2027-03-01'))),'2028-02-29')
})
test('annual filters can use the next occurrence, and expiry changes after midnight',()=>{
  assert.equal(day(countdownTarget(event('2000-01-03'),date('2026-12-31'))),'2027-01-03')
  assert.equal(countdownStatus(event('2026-10-13','once'),date('2026-10-13')).past,false)
  assert.equal(countdownStatus(event('2026-10-13','once'),date('2026-10-14')).past,true)
})
test('target identity normalizes Windows separators and URL host casing',()=>{
  assert.equal(targetKey('app','C:/Apps/Example.exe'),targetKey('app','c:\\apps\\example.exe'))
  assert.equal(targetKey('web','https://EXAMPLE.com'),targetKey('web','https://example.com/'))
  assert.notEqual(targetKey('web','https://example.com/A'),targetKey('web','https://example.com/a'))
})
test('visible resource order stays stable after usage changes and appends new entries',()=>{
  assert.deepEqual(stableResourceOrder([{id:'b'},{id:'c'},{id:'a'}],['a','b']).map(i=>i.id),['a','b','c'])
})
test('weather translates smoke and haze and prefers supplied Chinese labels',()=>{
  const weather=desc=>({current_condition:[{temp_C:'24',weatherDesc:[{value:desc}]}]})
  assert.equal(parseFeed('weather',weather('Smoky haze')).content,'霾')
  const raw=weather('Clear');raw.current_condition[0].lang_zh=[{value:'晴朗'}]
  assert.equal(parseFeed('weather',raw).content,'晴朗')
})
