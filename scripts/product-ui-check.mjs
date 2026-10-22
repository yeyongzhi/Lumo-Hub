// Run with: node scripts/product-ui-check.mjs <path-to-playwright> [artifact-directory]
// Desktop commands are replaced with fixtures; no real user data is read or changed.
import { createRequire } from 'node:module'
import assert from 'node:assert/strict'

const require = createRequire(import.meta.url)
const { chromium } = require(process.argv[2] || 'playwright')
const browser = await chromium.launch({ headless: true, channel: 'msedge' })
const artifacts = process.argv[3]
const errors = []

try {
  const page = await browser.newPage({ viewport: { width: 800, height: 720 } })
  page.setDefaultTimeout(10_000)
  page.on('pageerror', error => errors.push(error.message))
  const settings = {
    client: {'lumo-launcher-pinned-group':'group-work','lumo-launcher-fixed-items':['app-7']},
    product: { close_to_tray: true, autostart: false, reminder_hour: 9, reminders: {}, pinned_stickies: [], note_tags: {} },
    theme_mode: 'light', accent: 'blue', font_size: 14, border_radius: 16, sidebar_width: 240,
    common_launchers: ['computer', 'explorer', 'task_manager', 'windows_settings'], welcome_completed: true,
    home_layout: [], launcher: { enabled: true, shortcut: 'Alt+Space', minimal_mode: false },
    auto_backup: { enabled: true, interval_hours: 24, retain: 7 },
  }
  const resources = Array.from({ length: 12 }, (_, index) => ({
    id: `app-${index + 1}`,
    name: `最近应用 ${index + 1}`,
    kind: 'app',
    target: `C:/Apps/${index + 1}.exe`,
    group_id: index===6?'group-work':null,
    is_favorite: false,
    icon_path: null,
    icon_data: null,
    aliases: '',
    usage_count: 12 - index,
    last_opened_at: new Date(Date.now() - index * 60_000).toISOString(),
    created_at: '2026-01-01T00:00:00Z',
    updated_at: '2026-01-01T00:00:00Z',
  }))

  await page.addInitScript(({ settings, resources }) => {
    const callbacks = new Map()
    const listeners = new Map()
    let callbackId = 0
    window.isTauri = true
    window.__fixture = { launched: [], launchedCommon: [], openedTools: [], openedMain: [], resources, settings,
      emit: event => { for(const id of listeners.get(event)||[]) callbacks.get(id)?.({event,payload:null}) }
    }
    window.__TAURI_EVENT_PLUGIN_INTERNALS__ = { unregisterListener: () => {} }
    window.__TAURI_INTERNALS__ = {
      metadata: { currentWindow: { label: 'quick-launch' }, currentWebview: { label: 'quick-launch' } },
      transformCallback: fn => { callbacks.set(++callbackId, fn); return callbackId },
      unregisterCallback: id => callbacks.delete(id),
      convertFileSrc: path => path,
      invoke: async (command, args = {}) => {
        if (command === 'plugin:event|listen') {
          const handlers = listeners.get(args.event) || []
          handlers.push(args.handler)
          listeners.set(args.event, handlers)
          return args.handler
        }
        if (command === 'bootstrap_state') return { bootstrap: { data_dir: 'fixture', default_data_dir: 'fixture' }, settings: structuredClone(settings), clean_uninstall: false, recovery_error: null }
        if (command === 'read_local') return settings.client[args.key] ?? null
        if (command === 'write_local') { settings.client[args.key] = args.value; return }
        if (command === 'list_notes' || command === 'list_stickies' || command === 'list_countdowns') return []
        if (command === 'note_file_issues') return []
        if (command === 'list_groups') return [{id:'group-work',name:'工作应用'}]
        if (command === 'list_resources') return structuredClone(resources)
        if (command === 'common_launcher_icons') return {}
        if (command === 'search_all') return []
        if (command === 'launch_resource') { window.__fixture.launched.push(args.id); return new Date().toISOString() }
        if (command === 'open_builtin_tool') { window.__fixture.openedTools.push(args.id); return null }
        if (command === 'launch_common') { window.__fixture.launchedCommon.push(args.id); return null }
        if (command === 'open_main_page') { window.__fixture.openedMain.push(args.page); return null }
        if (command === 'hide_launcher' || command.startsWith('plugin:') || command === 'open_startup_view') return null
        throw new Error(`Missing fixture: ${command}`)
      },
    }
  }, { settings, resources })

  await page.goto('http://localhost:1420/')
  const search = page.getByRole('combobox', { name: '搜索内容' })
  await search.waitFor()
  const firstRecent = page.locator('[data-launch-grid="recent"]').getByRole('option', { name: '最近应用 1', exact: true })
  await firstRecent.waitFor()
  assert.equal(await firstRecent.getAttribute('aria-selected'), 'true', 'the first recent app should be selected when the panel opens')

  await search.press('ArrowRight')
  assert.equal(await search.getAttribute('aria-activedescendant'), 'quick-result-recent:app-2')
  await search.press('ArrowLeft')
  assert.equal(await search.getAttribute('aria-activedescendant'), 'quick-result-recent:app-1')
  assert.equal(await page.locator('[data-launch-grid="content"] [id^="quick-result-recent:"]').count(), 0)
  const style = await firstRecent.evaluate(node => ({ shadow: getComputedStyle(node).boxShadow, border: getComputedStyle(node).borderWidth }))
  assert.equal(style.shadow, 'none')
  assert.equal(style.border, '0px')
  await page.getByRole('combobox', { name: '搜索范围' }).click()
  await page.getByRole('option', { name: '全部内容', exact: true }).click()
  await search.focus()
  await search.press('ArrowDown')
  const activeId = await search.getAttribute('aria-activedescendant')
  assert.notEqual(activeId, 'quick-result-recent:app-1', 'ArrowDown should move to the next visible row')

  await search.press('ArrowUp')
  const returnedId = await search.getAttribute('aria-activedescendant')
  assert.equal(returnedId, 'quick-result-recent:app-1', 'ArrowUp should return to the first recent app in its column')

  const selectedResourceId = returnedId.replace('quick-result-recent:', '')
  await search.press('Enter')
  await page.waitForFunction(id => window.__fixture.launched.includes(id), selectedResourceId)
  assert.deepEqual(await page.evaluate(() => window.__fixture.launched), [selectedResourceId], 'Enter should launch the highlighted recent app')
  await firstRecent.focus()
  await page.keyboard.press('ArrowRight')
  await page.keyboard.press('Enter')
  await page.waitForFunction(() => window.__fixture.launched.length === 2)
  assert.deepEqual(await page.evaluate(() => window.__fixture.launched), [selectedResourceId, 'app-2'], 'Enter on a focused tile should launch the highlighted app exactly once')
  const grids=await page.locator('[data-launch-grid]').evaluateAll(nodes=>nodes.map(node=>node.getAttribute('data-launch-grid')))
  assert.deepEqual(grids,['recent','pinned','common','group','tools','resource'])
  const commonGrid=page.locator('[data-launch-grid="common"]')
  await commonGrid.getByRole('option', { name: '任务管理器' }).waitFor()
  await commonGrid.getByRole('option', { name: '设置' }).waitFor()
  await commonGrid.getByRole('option', { name: '任务管理器' }).click()
  await page.waitForFunction(() => window.__fixture.launchedCommon.includes('task_manager'))
  await commonGrid.getByRole('option', { name: '设置' }).click()
  await page.waitForFunction(() => window.__fixture.launchedCommon.includes('windows_settings'))
  const pinned=page.locator('[data-launch-grid="pinned"] [role="option"]')
  await pinned.focus()
  await pinned.press('Enter')
  await page.waitForFunction(()=>window.__fixture.launched.length===3)
  assert.equal(await page.evaluate(()=>window.__fixture.launched[2]),'app-7')
  await page.evaluate(()=>{window.__fixture.settings.client['lumo-launcher-order']=['pinned','common','recent','resource'];window.__fixture.emit('lumo-settings-changed')})
  await page.waitForFunction(()=>document.querySelector('[data-launch-grid]')?.getAttribute('data-launch-grid')==='pinned')
  const ordered=await page.locator('[data-launch-grid]').evaluateAll(nodes=>nodes.map(node=>node.getAttribute('data-launch-grid')))
  assert.deepEqual(ordered,['pinned','common','recent','group','tools','resource'])
  for(const mode of ['dark','light']){
    await page.evaluate(mode=>{window.__fixture.settings.theme_mode=mode;window.__fixture.emit('lumo-settings-changed')},mode)
    await page.waitForFunction(mode=>document.documentElement.dataset.theme===mode,mode)
  }
  assert.match(await page.locator('footer time').textContent(), /^\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2} 周[日一二三四五六]$/)
  await page.getByRole('button', { name: '打开主面板' }).focus()
  await page.keyboard.press('Enter')
  assert.deepEqual(await page.evaluate(() => window.__fixture.openedMain), ['home'])
  await page.evaluate(()=>{window.__fixture.settings.client['lumo-launcher-fixed-items']=[];window.__fixture.emit('lumo-settings-changed')})
  await page.waitForFunction(()=>!document.querySelector('[data-launch-grid="pinned"]'))
  assert.equal(await page.getByText('固定项 · 0').count(), 0, 'the pinned section should disappear when it has no items')
  assert.equal(await page.locator('[data-launch-grid="tools"] [role="option"]').count(), 6)
  for (const [query,id] of [['color picker','color'],['colorPicker','color'],['clipboard','clipboard'],['qrcode','qrcode'],['JSON','text'],['悬浮','floating'],['timestamp','timestamp']]) {
    await search.fill(query)
    await page.locator(`#quick-result-tool\\:${id}`).waitFor()
    await search.press('Enter')
    await page.waitForFunction(id=>window.__fixture.openedTools.includes(id),id)
  }
  await search.fill('')
  await page.getByRole('button', { name: '关闭快捷启动' }).focus()
  await page.keyboard.press('Enter')
  assert.deepEqual(errors, [])
  if (artifacts) await page.screenshot({ path: `${artifacts}/quick-launch-keyboard.png` })
  console.log('UI check passed: launcher keyboard selection, Enter launch, pinned section visibility, footer clock, main-panel and close controls, and theme switching.')
} catch (error) {
  console.error('Browser errors:', errors)
  throw error
} finally {
  await browser.close()
}
