import { createApp } from "vue";
import { createPinia } from "pinia";
import App from "./App.vue";
import ToolWindow from "./views/ToolWindow.vue";
import LaunchPanel from "./views/LaunchPanel.vue";
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

createApp(new URLSearchParams(location.search).has("tool") ? ToolWindow : isTauri() && getCurrentWindow().label === "quick-launch" ? LaunchPanel : App).use(createPinia()).mount("#app");
