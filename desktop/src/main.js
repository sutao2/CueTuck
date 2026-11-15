import { createApp, h } from "vue";
import App from "./App.vue";
import AppLockGate from "./components/AppLockGate.vue";
import "./styles/tokens.css";

createApp({ render: () => h(AppLockGate, { launcher: false }, { default: () => h(App) }) }).mount("#app");
