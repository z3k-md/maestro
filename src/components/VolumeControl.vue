<template>
  <div class="flex flex-col items-center gap-0 bg-base-100 mx-2 p-2 rounded-md h-14 justify-center">
    <div class="flex flex-row items-center gap-2">
      <SessionButton :name="sessionName" :icon="iconSrc" :volume="volume" :mute="mute" />
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
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from "vue";
import { Command, invokeCommand } from "@/utils/commands";
import { AppEvent, listenToEvent } from "@/utils/events";
import { logger } from "@/utils/logger";
import { sessionIconSrc } from "@/utils/ui";
import SessionButton from "./SessionButton.vue";
import type { AudioSession } from "@/types/audioSession";

interface Props {
  sessionName: string;
  volume: number;
  mute: boolean;
  icon: string | null;
}

const props = defineProps<Props>();

const volume = ref(props.volume);
const mute = ref(props.mute);
const iconSrc = computed(() => sessionIconSrc(props.icon));

// The mixer re-fetches sessions each time it opens
watch(
  () => [props.volume, props.mute] as const,
  ([newVolume, newMute]) => {
    volume.value = newVolume;
    mute.value = newMute;
  },
);

let unlisten: (() => void) | null = null;

onMounted(async () => {
  unlisten = await listenToEvent(AppEvent.VolumeChange, (payload: AudioSession) => {
    if (payload.name !== props.sessionName) {
      return;
    }

    logger.debug(`Volume change event: ${payload.name} ${payload.volume} ${payload.mute}`);

    volume.value = payload.volume;
    mute.value = payload.mute;
  });
});

onUnmounted(() => {
  unlisten?.();
});

async function updateVolume(newVolume: number) {
  if (newVolume === volume.value) {
    return;
  }

  volume.value = newVolume;

  try {
    await invokeCommand(Command.SetSessionVolume, { sessionName: props.sessionName, volume: newVolume });
  } catch (error) {
    logger.error(`Error setting ${props.sessionName} volume`, error);
  }
}

const handleSliderChange = (event: Event) => {
  const target = event.target as HTMLInputElement;
  updateVolume(Number(target.value));
};
</script>
