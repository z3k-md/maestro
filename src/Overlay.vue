<template>
  <div
    class="flex flex-col h-screen w-screen bg-base-300 justify-center align-middle"
    @mousedown="resetHideTimeout"
    @mousemove="resetHideTimeout"
    @mouseup="resetHideTimeout"
    @mouseover="resetHideTimeout"
  >
    <div class="flex items-center gap-0 m-2 rounded-md justify-center">
      <div class="flex flex-row items-center gap-2">
        <SessionButton :name="sessionName" :icon="icon" :volume="volume" :mute="mute" hoverStyle="bg-base-100" />
        <input
          type="range"
          min="0"
          max="100"
          :value="volume"
          :class="`range range-xs ${mute ? 'range-error' : 'range-primary'}`"
          @input="handleSliderChange"
        />
        <h2 class="text-lg w-12 text-center cursor-default">{{ volume }}</h2>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import { currentMonitor, getCurrentWindow, PhysicalPosition, PhysicalSize } from "@tauri-apps/api/window";
import SessionButton from "./components/SessionButton.vue";
import type { AudioSession } from "./types/audioSession";
import { Command, invokeCommand } from "./utils/commands";
import { AppEvent, listenToEvent } from "./utils/events";
import { logger } from "./utils/logger";
import { applyTheme, sessionIconSrc } from "./utils/ui";

const sessionName = ref("master");
const volume = ref(0);
const mute = ref(false);
const icon = ref(sessionIconSrc(null));

let hideTimeout: ReturnType<typeof setTimeout> | null = null;

const setSession = (session: AudioSession) => {
  sessionName.value = session.name;
  volume.value = session.volume;
  mute.value = session.mute;
  icon.value = sessionIconSrc(session.icon);
};

function resetHideTimeout() {
  if (hideTimeout) {
    clearTimeout(hideTimeout);
  }
  hideTimeout = setTimeout(() => {
    getCurrentWindow().hide();
  }, 1000);
}

async function updateVolume(newVolume: number) {
  if (newVolume === volume.value) {
    return;
  }

  volume.value = newVolume;

  logger.info(`Setting ${sessionName.value} volume to ${newVolume}`);

  try {
    await invokeCommand(Command.SetSessionVolume, { sessionName: sessionName.value, volume: newVolume });
  } catch (error) {
    logger.error("Error setting volume", error);
  }
}

const handleSliderChange = (event: Event) => {
  const target = event.target as HTMLInputElement;
  const volumeValue = Number(target.value);
  updateVolume(volumeValue);
};

let unlisten: (() => void) | null = null;

onMounted(async () => {
  const session = await invokeCommand(Command.GetSession, { sessionName: sessionName.value });
  if (session) {
    setSession(session);
  }

  const config = await invokeCommand(Command.GetConfig);
  applyTheme(config.system.theme);

  listenToEvent(AppEvent.ThemeChange, (theme) => {
    applyTheme(theme);
  });

  const initWindow = async () => {
    const window = getCurrentWindow();
    const monitor = await currentMonitor();

    if (!monitor) {
      logger.error("No monitor found");
      return;
    }

    const scaleFactor = monitor.scaleFactor;

    const width = 300 * scaleFactor;
    const height = 60 * scaleFactor;

    const x = Math.round(monitor.position.x + (monitor.size.width - width) / 2);
    const y = Math.round(monitor.position.y + 20 * scaleFactor);

    logger.debug(`Setting window size to ${width}x${height}, location to ${x}, ${y}`);

    await window.setSize(new PhysicalSize(width, height));
    await window.setPosition(new PhysicalPosition(x, y));
  };

  await initWindow();

  unlisten = await listenToEvent(AppEvent.VolumeChange, (session: AudioSession) => {
    logger.debug(`Volume change event: ${session.name} ${session.volume} ${session.mute}`);

    setSession(session);

    resetHideTimeout();
  });

  resetHideTimeout();
});

onUnmounted(() => {
  if (unlisten) {
    unlisten();
  }
});
</script>
