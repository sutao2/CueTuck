import { createApp, h } from "vue";
import LauncherApp from "./LauncherApp.vue";
import AppLockGate from "./components/AppLockGate.vue";
import "./styles/tokens.css";

document.documentElement.classList.add("launcher-root");
document.body.classList.add("launcher-page");

createApp({ render: () => h(AppLockGate, { launcher: true }, { default: () => h(LauncherApp) }) }).mount("#app");
