import { beforeEach, expect, it, vi } from 'vitest';
import { createPinia, setActivePinia } from 'pinia';
import { nextTick } from 'vue';
vi.mock('@/lib/api', () => ({
  api: {
    setSetting: vi.fn(),
    listResources: vi.fn(),
    listGroups: vi.fn(),
    listNotes: vi.fn(),
    listTrash: vi.fn(),
    restoreTrash: vi.fn(),
    listStickies: vi.fn(),
  },
}));
vi.mock('@/lib/theme', () => ({ applySettings: vi.fn() }));
vi.mock('@/lib/local-files', () => ({ writeLocal: vi.fn(), migrateLegacyStorage: vi.fn() }));
import { api } from '@/lib/api';
import { applySettings } from '@/lib/theme';
import { useWorkspaceStore } from './workspace';
beforeEach(() => {
  setActivePinia(createPinia());
  vi.clearAllMocks();
});
it('rolls back failed optimistic settings and continues saving', async () => {
  const store = useWorkspaceStore();
  const old = store.settings.accent;
  vi.mocked(api.setSetting).mockRejectedValueOnce(new Error('write failed'));
  const failed = store.setSetting('accent', 'red');
  expect(store.settings.accent).toBe('red');
  await expect(failed).rejects.toThrow();
  expect(store.settings.accent).toBe(old);
  vi.mocked(api.setSetting).mockResolvedValue({ ...store.settings, accent: 'green' });
  await store.setSetting('accent', 'green');
  expect(store.settings.accent).toBe('green');
});
it('refreshes resource data without fetching unrelated notes', async () => {
  const store = useWorkspaceStore();
  vi.mocked(api.listResources).mockResolvedValue([]);
  vi.mocked(api.listGroups).mockResolvedValue([]);
  await store.reloadResources();
  expect(api.listResources).toHaveBeenCalledOnce();
  expect(api.listNotes).not.toHaveBeenCalled();
});
it('does not reapply theme when only note tags change', async () => {
  const store = useWorkspaceStore();
  await nextTick();
  vi.mocked(applySettings).mockClear();
  store.settings.product.note_tags.one = ['work'];
  await nextTick();
  expect(applySettings).not.toHaveBeenCalled();
});
it('undo restores only records deleted by that action', async () => {
  const store = useWorkspaceStore();
  vi.mocked(api.listTrash).mockResolvedValue([
    { id: 'trash-one', original_id: 'one', kind: 'sticky', title: 'same', deleted_at: '' },
    { id: 'trash-other', original_id: 'other', kind: 'sticky', title: 'same', deleted_at: '' },
  ]);
  vi.mocked(api.restoreTrash).mockResolvedValue();
  vi.mocked(api.listStickies).mockResolvedValue([]);
  await store.notifyDeleted('sticky', ['one']);
  expect(store.undoAction).toBeTypeOf('function');
  await store.undoAction!();
  expect(api.restoreTrash).toHaveBeenCalledExactlyOnceWith('trash-one');
  expect(store.undoAction).toBeNull();
});
