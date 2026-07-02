import { browser } from '$app/environment';

export type ThemeMode = 'light' | 'dark' | 'system';

let mode = $state<ThemeMode>('system');

function getSystemTheme(): 'light' | 'dark' {
  if (!browser) return 'light';
  return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
}

function applyTheme(theme: 'light' | 'dark') {
  if (!browser) return;
  document.documentElement.setAttribute('data-theme', theme);
}

/** 当前生效的主题（light 或 dark） */
export function getEffectiveTheme(): 'light' | 'dark' {
  return mode === 'system' ? getSystemTheme() : mode;
}

/** 应用当前主题到 DOM */
export function applyCurrentTheme() {
  applyTheme(getEffectiveTheme());
}

/** 获取用户选择的模式 */
export function getThemeMode(): ThemeMode {
  return mode;
}

/** 设置并保存主题模式 */
export function setThemeMode(newMode: ThemeMode) {
  mode = newMode;
  if (browser) {
    localStorage.setItem('theme-mode', newMode);
  }
  applyCurrentTheme();
}

/** 初始化主题：读取 localStorage、监听系统主题变化 */
export function initTheme() {
  if (!browser) return;
  const saved = localStorage.getItem('theme-mode') as ThemeMode | null;
  if (saved && ['light', 'dark', 'system'].includes(saved)) {
    mode = saved;
  }
  applyCurrentTheme();

  window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', () => {
    if (mode === 'system') {
      applyCurrentTheme();
    }
  });
}
