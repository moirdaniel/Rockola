const LS_SOURCE = "rockola.sourcePath";
const LS_THEME = "rockola.theme";
const LS_SCALE = "rockola.scale";

export const Settings = {
  getSourcePath(): string {
    return localStorage.getItem(LS_SOURCE) || "";
  },
  setSourcePath(v: string) {
    localStorage.setItem(LS_SOURCE, v);
  },

  getTheme(): "light" | "dark" {
    return (localStorage.getItem(LS_THEME) as any) || "light";
  },
  setTheme(v: "light" | "dark") {
    localStorage.setItem(LS_THEME, v);
  },

  getScale(): number {
    const v = Number(localStorage.getItem(LS_SCALE));
    return Number.isFinite(v) && v >= 0.8 && v <= 1.4 ? v : 1.0;
  },
  setScale(v: number) {
    localStorage.setItem(LS_SCALE, String(v));
  },
};
