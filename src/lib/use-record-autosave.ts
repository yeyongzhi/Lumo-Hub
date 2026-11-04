import { onBeforeUnmount, onMounted, ref, watch, type Ref } from 'vue';

interface Options {
  id: Ref<string>;
  title: Ref<string>;
  content: Ref<string>;
  loading: () => boolean;
  save: (id: string, title: string, content: string) => Promise<unknown>;
  committed: (id: string, title: string, content: string) => void;
  notify: (message: string) => void;
  label: string;
  register: (flush: () => Promise<void>) => () => void;
  beforeSave?: () => Promise<boolean>;
}

export function useRecordAutosave(options: Options) {
  const saveState = ref('');
  let timer: ReturnType<typeof setTimeout> | undefined;
  let chain: Promise<unknown> = Promise.resolve();
  let unregister: (() => void) | undefined;
  async function saveNow(): Promise<boolean> {
    if (options.beforeSave && !(await options.beforeSave())) return false;
    if (timer) clearTimeout(timer);
    const id = options.id.value;
    if (!id) return true;
    if (!saveState.value || saveState.value === '已保存') {
      await chain;
      return true;
    }
    while (options.id.value === id) {
      const title = options.title.value,
        content = options.content.value;
      saveState.value = '保存中…';
      const job = chain.then(() => options.save(id, title, content));
      chain = job.catch(() => {});
      try {
        await job;
        options.committed(id, title, content);
        if (options.id.value !== id) return true;
        if (options.title.value === title && options.content.value === content) {
          saveState.value = '已保存';
          return true;
        }
      } catch (error) {
        if (options.id.value === id) saveState.value = '保存失败';
        options.notify(`${options.label}保存失败：${String(error)}`);
        return false;
      }
    }
    return true;
  }
  watch(
    [options.title, options.content],
    () => {
      if (options.loading() || !options.id.value) return;
      saveState.value = '未保存';
      if (timer) clearTimeout(timer);
      timer = setTimeout(() => void saveNow(), 600);
    },
    { flush: 'sync' },
  );
  onMounted(() => {
    unregister = options.register(async () => {
      if (!(await saveNow())) throw new Error(`${options.label}尚未保存`);
    });
  });
  onBeforeUnmount(() => {
    if (timer) clearTimeout(timer);
    unregister?.();
    if (saveState.value === '未保存') void saveNow();
  });
  return { saveState, saveNow };
}
