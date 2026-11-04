import { ref } from 'vue';
import { normalizeHomeWidget } from './home-layout';
import type { HomeWidget } from './types';
export function useHomeLayoutEditor(initial: HomeWidget[]) {
  const widgets = ref(initial.map(normalizeHomeWidget));
  const editing = ref(false);
  const layoutUndo = ref<HomeWidget[][]>([]);
  function rememberLayout() {
    layoutUndo.value.push(widgets.value.map(normalizeHomeWidget));
    if (layoutUndo.value.length > 20) layoutUndo.value.shift();
  }
  function changeLayout(value: HomeWidget[]) {
    rememberLayout();
    widgets.value = value.map(normalizeHomeWidget);
  }
  function removeWidget(id: string) {
    rememberLayout();
    widgets.value = widgets.value.filter((widget) => widget.id !== id);
  }
  function beginEdit(layout: HomeWidget[]) {
    layoutUndo.value = [];
    widgets.value = layout.map(normalizeHomeWidget);
    editing.value = true;
  }
  function cancelEdit(layout: HomeWidget[]) {
    widgets.value = layout.map(normalizeHomeWidget);
    editing.value = false;
  }
  return {
    widgets,
    editing,
    layoutUndo,
    rememberLayout,
    changeLayout,
    removeWidget,
    beginEdit,
    cancelEdit,
  };
}
