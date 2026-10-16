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
    client: {},
    product: { close_to_tray: true, autostart: false, reminder_hour: 9, reminders: {}, pinned_stickies: [], note_tags: {} },
    theme_mode: 'light', accent: 'blue', font_size: 14, border_radius: 16, sidebar_width: 240,
    common_launchers: ['computer', 'explorer'], welcome_completed: true,
    home_layout: [], launcher: { enabled: true, shortcut: 'Alt+Space', minimal_mode: false },
    auto_backup: { enabled: true, interval_hours: 24, retain: 7 },
  }
  const resources = Array.from({ length: 12 }, (_, index) => ({
    id: `app-${index + 1}`,
    name: `最近应用 ${index + 1}`,
    kind: 'app',
    target: `C:/Apps/${index + 1}.exe`,
    group_id: null,
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
    window.__fixture = { launched: [], resources, settings }
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
        if (command === 'list_notes' || command === 'list_stickies' || command === 'list_countdowns' || command === 'list_groups') return []
        if (command === 'list_resources') return structuredClone(resources)
        if (command === 'common_launcher_icons') return {}
        if (command === 'search_all') return []
        if (command === 'launch_resource') { window.__fixture.launched.push(args.id); return new Date().toISOString() }
        if (command === 'hide_launcher' || command.startsWith('plugin:') || command === 'open_startup_view') return null
        throw new Error(`Missing fixture: ${command}`)
      },
    }
  }, { settings, resources })

  await page.goto('http://localhost:1420/')
  const search = page.getByRole('combobox', { name: '搜索内容' })
  await search.waitFor()
  const firstRecent = page.getByRole('option', { name: '最近应用 1' })
  await firstRecent.waitFor()
  assert.equal(await firstRecent.getAttribute('aria-selected'), 'true', 'the first recent app should be selected when the panel opens')

  await search.press('ArrowDown')
  const activeId = await search.getAttribute('aria-activedescendant')
  assert.ok(activeId?.startsWith('quick-result-recent:app-'), `ArrowDown should keep navigation in the recent-app grid, got ${activeId}`)
  const selectedRecent = page.locator('[data-launch-grid="recent"] [aria-selected="true"]')
  assert.equal(await selectedRecent.getAttribute('aria-selected'), 'true')

  await search.press('ArrowUp')
  const returnedId = await search.getAttribute('aria-activedescendant')
  assert.equal(returnedId, 'quick-result-recent:app-1', 'ArrowUp should return to the first recent app in its column')

  const selectedResourceId = returnedId.replace('quick-result-recent:', '')
  await search.press('Enter')
  await page.waitForFunction(id => window.__fixture.launched.includes(id), selectedResourceId)
  assert.deepEqual(await page.evaluate(() => window.__fixture.launched), [selectedResourceId], 'Enter should launch the highlighted recent app')
  assert.deepEqual(errors, [])
  if (artifacts) await page.screenshot({ path: `${artifacts}/quick-launch-keyboard.png` })
  console.log('UI check passed: recent app tiles participate in arrow-key selection and Enter launches the selected app.')
} catch (error) {
  console.error('Browser errors:', errors)
  throw error
} finally {
  await browser.close()
}
