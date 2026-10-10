<template>
  <div id="container" class="flex flex-col h-screen w-screen bg-base-300 justify-center m-0 p-0">
    <h1 class="text-md font-bold pb-2 px-4 m-0">Maestro</h1>
    <div class="flex flex-col gap-2">
      <VolumeControl
        v-for="session in sessions"
        :key="session.name"
        :sessionName="session.name"
        :volume="session.volume"
        :mute="session.mute"
        :icon="session.icon"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, watch } from "vue";
import { currentMonitor, getCurrentWindow, PhysicalPosition, PhysicalSize } from "@tauri-apps/api/window";
import VolumeControl from "./components/VolumeControl.vue";
import type { AudioSession } from "./types/audioSession";
import { Command, invokeCommand } from "./utils/commands";
import { AppEvent, listenToEvent } from "./utils/events";
import { logger } from "./utils/logger";
import { applyTheme } from "./utils/ui";

const sessions = ref<AudioSession[]>([]);

const fetchSessions = async () => {
  const sessionsData = await invokeCommand(Command.GetAllSessions);

  sessionsData.sort((a: AudioSession, b: AudioSession) => {
    if (a.name.toLowerCase() === "master") return -1;
    if (b.name.toLowerCase() === "master") return 1;
    return a.name.localeCompare(b.name);
  });

  sessions.value = sessionsData;
};

const setWindowSizeAndPosition = async () => {
  const monitor = await currentMonitor();
  if (!monitor) {
    logger.error("No monitor found");
    return;
  }

  const appWindow = getCurrentWindow();
  const { scaleFactor } = monitor;

  const windowWidth = Math.round(300 * scaleFactor);

  // Height grows with the number of sessions
  const baseHeight = 75;
  const padding = 40;
  const windowHeight = Math.round((sessions.value.length * baseHeight + padding) * scaleFactor);

  await appWindow.setSize(new PhysicalSize(windowWidth, windowHeight));

  const x = monitor.position.x + monitor.size.width - (windowWidth + 20);
  const y = monitor.position.y + monitor.size.height - (windowHeight + 80);

  await appWindow.setPosition(new PhysicalPosition(x, y));
};

watch(
  () => sessions.value.length,
  async () => {
    await setWindowSizeAndPosition();
  },
);

onMounted(async () => {
  await fetchSessions();
  await setWindowSizeAndPosition();

  const config = await invokeCommand(Command.GetConfig);
  applyTheme(config.system.theme);

  listenToEvent(AppEvent.MixerVisibilityChange, async (visible: boolean) => {
    if (visible) {
      await fetchSessions();
    }
  });

  listenToEvent(AppEvent.ThemeChange, (theme) => {
    logger.info(`Theme changed to ${theme}`);
    applyTheme(theme);
  });
});
</script>
