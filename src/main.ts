import { createApp } from "vue";
import { createPinia } from "pinia";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { isTauri } from "@tauri-apps/api/core";
import "./assets/main.css";

if (import.meta.env.PROD && isTauri()) {
  document.addEventListener("contextmenu", (event) => {
    const target = event.target;
    const editable = target instanceof HTMLElement && (
      target.closest("input:not([readonly]):not([disabled]), textarea:not([readonly]):not([disabled])") !== null ||
      target.isContentEditable
    );
    if (!editable) event.preventDefault();
  });

  window.addEventListener("keydown", (event) => {
    if (event.key === "F5" || ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "r")) {
      event.preventDefault();
      event.stopImmediatePropagation();
    }
  }, { capture: true });
}

async function mountWindow() {
  const { default: root } = await (new URLSearchParams(location.search).has("tool")
    ? import("./views/ToolWindow.vue")
    : isTauri() && getCurrentWindow().label === "quick-launch"
      ? import("./views/LaunchPanel.vue")
      : import("./App.vue"));
  createApp(root).use(createPinia()).mount("#app");
}
void mountWindow();
