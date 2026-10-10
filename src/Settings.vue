<template>
  <div class="flex flex-col gap-4 p-4 bg-base-300 h-full w-full">
    <!-- Toast container -->
    <div class="toast toast-top toast-center" v-if="toast">
      <div :class="`alert ${toast.success ? 'alert-success' : 'alert-error'}`">
        <span>{{ toast.message }}</span>
      </div>
    </div>
    <!-- Header -->
    <div class="flex flex-row justify-between">
      <h1 class="text-3xl font-bold text-center p-2">Settings</h1>
      <div class="flex flex-row gap-2">
        <button class="btn btn-outline btn-sm btn-info" @click="handleReset">Reset</button>
        <button class="btn btn-outline btn-sm btn-success" @click="handleSave">Save</button>
      </div>
    </div>
    <!-- Audio Mapping -->
    <div id="sessions" class="bg-base-100 p-4 rounded-lg">
      <h2 class="text-xl font-bold text-left">Audio Mapping</h2>
      <div class="flex flex-row gap-2 justify-between">
        <div v-for="(session, index) in config?.sessions" :key="index" class="flex flex-col gap-2">
          <label class="form-control w-full max-w-xs">
            <div class="label">
              <span class="label-text">Session {{ index + 1 }}</span>
            </div>
            <input
              type="text"
              placeholder="master"
              class="input input-sm input-bordered w-24 max-w-xs"
              :value="session.name"
              @input="handleSessionNameChange(index, ($event.target as HTMLInputElement).value)"
            />
          </label>
        </div>
      </div>
    </div>

    <!-- Volume Mixer -->
    <div id="mixer" class="flex flex-col gap-2 p-4 bg-base-100 rounded-lg">
      <h2 class="text-xl font-bold text-left">Volume Mixer</h2>
      <div class="form-control w-52">
        <label class="label cursor-pointer">
          <span class="label-text">Enabled</span>
          <input
            type="checkbox"
            class="toggle toggle-primary"
            :checked="config?.mixer.enabled"
            @change="handleMixerChange('enabled', ($event.target as HTMLInputElement).checked)"
          />
        </label>
      </div>
      <!-- Hotkey -->
      <div class="flex flex-row gap-2">
        <label class="label">
          <span class="label-text whitespace-nowrap">Hotkey</span>
        </label>
        <input
          type="text"
          placeholder="Ctrl + Shift + M"
          class="input input-sm input-bordered w-full max-w-xs"
          :value="config?.mixer.hotkey ?? ''"
          @input="handleMixerChange('hotkey', ($event.target as HTMLInputElement).value || null)"
        />
      </div>
    </div>

    <!-- System Settings -->
    <div id="system" class="flex flex-col gap-2 p-4 bg-base-100 rounded-lg">
      <h2 class="text-xl font-bold text-left">System Settings</h2>
      <div class="form-control w-52">
        <label class="label cursor-pointer">
          <span class="label-text">System Startup</span>
          <input
            type="checkbox"
            class="toggle toggle-primary"
            :checked="config?.system.autostart"
            @change="handleSystemChange('autostart', ($event.target as HTMLInputElement).checked)"
          />
        </label>
        <label class="label cursor-pointer">
          <span class="label-text">Show Debug Console</span>
          <input
            type="checkbox"
            class="toggle toggle-primary"
            :checked="config?.system.show_console"
            @change="handleSystemChange('show_console', ($event.target as HTMLInputElement).checked)"
          />
        </label>
      </div>
      <ThemePicker :theme="config?.system.theme || DEFAULT_THEME" :setTheme="(newTheme: string) => handleSystemChange('theme', newTheme)" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from "vue";
import { PhysicalPosition, PhysicalSize } from "@tauri-apps/api/dpi";
import { currentMonitor, getCurrentWindow } from "@tauri-apps/api/window";
import ThemePicker from "./components/ThemePicker.vue";
import type { Config, MixerConfig, SystemConfig } from "./types/config";
import { Command, invokeCommand } from "./utils/commands";
import { logger } from "./utils/logger";
import { applyTheme, DEFAULT_THEME } from "./utils/ui";

const config = ref<Config>();
const savedConfig = ref<Config>();
const toast = ref<{ message: string; success: boolean } | null>(null);

const showToast = (message: string, success: boolean) => {
  toast.value = { message, success };
  setTimeout(() => {
    toast.value = null;
  }, 1500);
};

const loadConfig = async () => {
  const configData = await invokeCommand(Command.GetConfig);
  config.value = configData;
  savedConfig.value = configData;
  applyTheme(configData.system.theme);
};

const setWindowSizeAndPosition = async () => {
  const monitor = await currentMonitor();
  const appWindow = getCurrentWindow();

  if (monitor) {
    const { scaleFactor } = monitor;

    const windowWidth = Math.round(600 * scaleFactor);
    const windowHeight = Math.round(875 * scaleFactor);
    const taskbarHeight = (await invokeCommand(Command.GetTaskbarHeight)) * scaleFactor + 23; // Magic number
    const padding = 40 * scaleFactor;

    await appWindow.setSize(new PhysicalSize(windowWidth, windowHeight));

    const x = monitor.position.x + monitor.size.width - (windowWidth + padding);
    const y = monitor.position.y + monitor.size.height - (windowHeight + taskbarHeight + padding);

    await appWindow.setPosition(new PhysicalPosition(Math.round(x), Math.round(y)));
  } else {
    logger.error("No monitor found");
  }

  await appWindow.show();
  await appWindow.setFocus();
};

const handleSessionNameChange = (index: number, name: string) => {
  if (!config.value) return;
  const sessions = [...config.value.sessions];
  sessions[index] = { ...sessions[index], name };
  config.value = { ...config.value, sessions };
};

const handleMixerChange = <K extends keyof MixerConfig>(field: K, value: MixerConfig[K]) => {
  if (!config.value) return;
  config.value = { ...config.value, mixer: { ...config.value.mixer, [field]: value } };
};

const handleSystemChange = <K extends keyof SystemConfig>(field: K, value: SystemConfig[K]) => {
  if (!config.value) return;
  config.value = { ...config.value, system: { ...config.value.system, [field]: value } };
};

const handleReset = () => {
  config.value = savedConfig.value;
};

const handleSave = async () => {
  if (!config.value) return;

  try {
    await invokeCommand(Command.SetConfig, { config: config.value });
    savedConfig.value = config.value;
    showToast("Settings saved", true);
  } catch (error) {
    logger.error("Failed to save settings", error);
    showToast("Failed to save settings", false);
  }
};

onMounted(async () => {
  await loadConfig();
  await setWindowSizeAndPosition();
});
</script>
