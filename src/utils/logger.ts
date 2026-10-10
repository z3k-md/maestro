import { invoke } from "@tauri-apps/api/core";

type Level = "debug" | "info" | "warn" | "error";

function send(level: Level, message: string) {
  invoke("log", { message, level }).catch((error) => console.error("Failed to forward log", error));
}

export class Logger {
  public info(message: string) {
    send("info", message);
    console.log(message);
  }

  public debug(message: string) {
    send("debug", message);
    console.debug(message);
  }

  public warn(message: string) {
    send("warn", message);
    console.warn(message);
  }

  public error(message: string, error?: unknown) {
    send("error", error === undefined ? message : `${message}: ${String(error)}`);
    console.error(message, error);
  }
}

export const logger = new Logger();
