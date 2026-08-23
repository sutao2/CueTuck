<template>
  <WorkbenchShell
    :database-status="databaseStatus"
    :local-count="localCount"
    @open-launcher="openLauncher"
    @library-changed="localCount = $event"
  />
</template>

<script setup>
import { onMounted, ref } from "vue";
import WorkbenchShell from "./components/WorkbenchShell.vue";
import { getLocalSetting } from "./platform/library.js";
import { openLauncherWindow } from "./platform/launcherWindow.js";
import { DEFAULT_LAUNCHER_SHORTCUT, DEFAULT_NEW_PROMPT_SHORTCUT, DEFAULT_PASTE_RECENT_SHORTCUT, registerLauncherShortcut } from "./platform/shortcut.js";

const databaseStatus = ref("pending");
const localCount = ref(0);

onMounted(async () => {
  if (!window.__TAURI_INTERNALS__) {
    databaseStatus.value = "pending";
    return;
  }
  try {
    const { invoke } = await import("@tauri-apps/api/core");
    await invoke("initialize_local_database");
    databaseStatus.value = await invoke("get_local_database_status");
    localCount.value = await invoke("count_local_prompts");
    const combo = (await getLocalSetting("launcher_shortcut")) || DEFAULT_LAUNCHER_SHORTCUT;
    const createCombo = (await getLocalSetting("new_prompt_shortcut")) || DEFAULT_NEW_PROMPT_SHORTCUT;
    const pasteCombo = (await getLocalSetting("paste_recent_shortcut")) || DEFAULT_PASTE_RECENT_SHORTCUT;
    try {
      await registerLauncherShortcut(combo, {
        extras: [
          {
            combo: createCombo,
            handler: async (event) => {
              if (event?.state && event.state !== "Pressed") return;
              await invoke("open_new_prompt");
            },
          },
          {
            combo: pasteCombo,
            handler: async (event) => {
              if (event?.state && event.state !== "Pressed") return;
              await invoke("paste_recent_prompt");
            },
          },
        ],
      });
    } catch {
      /* 冲突时仍可用顶栏搜索按钮 */
    }
  } catch {
    databaseStatus.value = "failed";
  }
});

function openLauncher() {
  openLauncherWindow();
}
</script>
