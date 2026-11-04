import { afterEach, describe, expect, it, vi } from 'vitest';
import { createApp, h, ref } from 'vue';
import { useRecordAutosave } from './use-record-autosave';
import { useHomeLayoutEditor } from './use-home-layout-editor';
import { rankSearchResults } from './search-ranking';
import { normalizeNoteTags } from './note-tags';
import type { ProductSettings } from './types';

const cleanups: (() => void)[] = [];
afterEach(() => {
  cleanups.splice(0).forEach((fn) => fn());
  vi.useRealTimers();
});
function editor(save = vi.fn().mockResolvedValue(undefined)) {
  const id = ref('one'),
    title = ref('original'),
    content = ref('');
  const notify = vi.fn(),
    committed = vi.fn();
  let flush: (() => Promise<void>) | undefined;
  let result!: ReturnType<typeof useRecordAutosave>;
  const app = createApp({
    setup() {
      result = useRecordAutosave({
        id,
        title,
        content,
        loading: () => false,
        save,
        committed,
        notify,
        label: '笔记',
        register(fn) {
          flush = fn;
          return () => {
            flush = undefined;
          };
        },
      });
      return () => h('div');
    },
  });
  app.mount(document.createElement('div'));
  cleanups.push(() => app.unmount());
  return {
    id,
    title,
    content,
    save,
    notify,
    committed,
    get flush() {
      return flush!;
    },
    get state() {
      return result.saveState.value;
    },
    saveNow: () => result.saveNow(),
  };
}
describe('record saving', () => {
  it('debounces typing and flushes before navigation', async () => {
    vi.useFakeTimers();
    const e = editor();
    e.title.value = 'new';
    e.content.value = 'body';
    await vi.advanceTimersByTimeAsync(599);
    expect(e.save).not.toHaveBeenCalled();
    await e.flush();
    expect(e.save).toHaveBeenCalledExactlyOnceWith('one', 'new', 'body');
    expect(e.state).toBe('已保存');
    await vi.advanceTimersByTimeAsync(1000);
    expect(e.save).toHaveBeenCalledTimes(1);
  });
  it('blocks navigation after failure and retries the current draft', async () => {
    const save = vi.fn().mockRejectedValueOnce(new Error('disk full')).mockResolvedValue(undefined);
    const e = editor(save);
    e.title.value = 'draft';
    await expect(e.flush()).rejects.toThrow('尚未保存');
    expect(e.state).toBe('保存失败');
    expect(e.committed).not.toHaveBeenCalled();
    expect(await e.saveNow()).toBe(true);
    expect(e.state).toBe('已保存');
  });
  it('serializes edits made while the previous snapshot is writing', async () => {
    let release!: () => void;
    const save = vi
      .fn()
      .mockImplementationOnce(
        () =>
          new Promise<void>((resolve) => {
            release = resolve;
          }),
      )
      .mockResolvedValue(undefined);
    const e = editor(save);
    e.title.value = 'first';
    const pending = e.saveNow();
    await Promise.resolve();
    e.title.value = 'latest';
    release();
    await pending;
    expect(save.mock.calls.map((call) => call[1])).toEqual(['first', 'latest']);
    expect(e.state).toBe('已保存');
  });
});
it('keeps exact content matches ahead of fuzzy resources and preserves ties', () => {
  const entries = [
    { title: '旅行工具', kind: 'resource' },
    { title: '旅行', kind: 'note' },
    { title: '旅行计划', kind: 'note' },
  ];
  expect(rankSearchResults(entries, '旅行').map((item) => item.title)).toEqual([
    '旅行',
    '旅行工具',
    '旅行计划',
  ]);
  expect(entries[0].title).toBe('旅行工具');
});
it('migrates legacy tag catalogs without losing note assignments', () => {
  const original = {
    note_tags: { one: ['工作'], __tag_catalog__0: ['未使用', '工作'] },
    tag_catalog: ['现有'],
  } as unknown as ProductSettings;
  const migrated = normalizeNoteTags(original);
  expect(migrated.note_tags).toEqual({ one: ['工作'] });
  expect(migrated.tag_catalog).toEqual(['现有', '未使用', '工作']);
  expect(original.note_tags.__tag_catalog__0).toBeDefined();
});
it('restores canceled layout edits and keeps snapshots independent', () => {
  const initial = [
    { id: 'clock', kind: 'clock' as const, size: '2x2' as const, selected_ids: ['one'] },
  ];
  const e = useHomeLayoutEditor(initial);
  e.beginEdit(initial);
  e.rememberLayout();
  e.widgets.value[0].selected_ids!.push('two');
  expect(e.layoutUndo.value[0][0].selected_ids).toEqual(['one']);
  e.removeWidget('clock');
  e.cancelEdit(initial);
  expect(e.widgets.value).toEqual(initial);
  expect(e.editing.value).toBe(false);
});
