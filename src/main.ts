import { createApp } from "vue";
import { createPinia } from "pinia";
import App from "./App.vue";
import ToolWindow from "./views/ToolWindow.vue";
import LaunchPanel from "./views/LaunchPanel.vue";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { isTauri } from "@tauri-apps/api/core";
import "./assets/main.css";

createApp(new URLSearchParams(location.search).has("tool") ? ToolWindow : isTauri() && getCurrentWindow().label === "quick-launch" ? LaunchPanel : App).use(createPinia()).mount("#app");
