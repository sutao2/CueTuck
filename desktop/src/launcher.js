import { createApp } from "vue";
import LauncherApp from "./LauncherApp.vue";
import "./styles/tokens.css";

document.documentElement.classList.add("launcher-root");
document.body.classList.add("launcher-page");

createApp(LauncherApp).mount("#app");
