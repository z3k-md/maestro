export const DEFAULT_THEME = "dark";

export function applyTheme(theme: string | null | undefined) {
  document.documentElement.setAttribute("data-theme", theme || DEFAULT_THEME);
}

export function sessionIconSrc(icon: string | null | undefined): string {
  return icon ? `data:image/png;base64,${icon}` : "/master-speaker-512.png";
}
