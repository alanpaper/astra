<script lang="ts">
  import { page } from '$app/stores';
  import '../styles/global.css';
  let { children } = $props();

  const menuItems = [
    { id: 'workspace', label: '工作空间', path: '/' },
    { id: 'skills', label: 'Skills管理', path: '/skills' }
  ];

  let sidebarOpen = $state(false);
  let sidebarCollapsed = $state(false);

  function toggleMobileMenu() {
    sidebarOpen = !sidebarOpen;
  }

  function closeMobileMenu() {
    sidebarOpen = false;
  }

  function toggleCollapse() {
    sidebarCollapsed = !sidebarCollapsed;
  }
</script>

<div class="app-layout" class:collapsed={sidebarCollapsed}>
  <!-- 移动端遮罩 -->
  {#if sidebarOpen}
    <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions a11y_roles -->
    <div class="mobile-overlay" onclick={closeMobileMenu} role="presentation"></div>
  {/if}

  <!-- 展开按钮（侧边栏收起时显示） -->
  {#if sidebarCollapsed}
    <button class="expand-btn" onclick={toggleCollapse} aria-label="展开侧边栏" title="展开侧边栏">
      <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><polyline points="15 18 9 12 15 6"/></svg>
    </button>
  {/if}

  <!-- 汉堡按钮（移动端） -->
  <button class="hamburger" onclick={toggleMobileMenu} aria-label="切换菜单">
    <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
      {#if sidebarOpen}
        <line x1="18" y1="6" x2="6" y2="18"/>
        <line x1="6" y1="6" x2="18" y2="18"/>
      {:else}
        <line x1="3" y1="6" x2="21" y2="6"/>
        <line x1="3" y1="12" x2="21" y2="12"/>
        <line x1="3" y1="18" x2="21" y2="18"/>
      {/if}
    </svg>
  </button>

  <!-- 侧边栏 -->
  <aside class="sidebar" class:open={sidebarOpen}>
    <div class="sidebar-header">
      <h2>AiTool</h2>
      <button class="collapse-btn" onclick={toggleCollapse} title="收起侧边栏" aria-label="收起侧边栏">
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><polyline points="15 18 9 12 15 6"/></svg>
      </button>
    </div>
    <nav class="sidebar-nav">
      {#each menuItems as item}
        <a
          href={item.path}
          class:active={$page.url.pathname === item.path}
          onclick={closeMobileMenu}
        >
          <span class="nav-icon">{item.id === 'workspace' ? '📁' : '🧩'}</span>
          {item.label}
        </a>
      {/each}
    </nav>

    <!-- 底部设置入口 -->
    <div class="sidebar-footer">
      <a
        href="/settings"
        class:active={$page.url.pathname === '/settings'}
        onclick={closeMobileMenu}
      >
        <svg class="settings-icon" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"/>
        </svg>
        设置
      </a>
    </div>
  </aside>

  <!-- 主内容区 -->
  <main class="content">
    {@render children()}
  </main>
</div>

<style>
  * {
    margin: 0;
    padding: 0;
    box-sizing: border-box;
  }

  .app-layout {
    display: flex;
    height: 100vh;
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, sans-serif;
  }

  /* 展开按钮（侧边栏收起时显示在左边缘） */
  .expand-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    position: fixed;
    top: 12px;
    left: 12px;
    z-index: 1001;
    width: 40px;
    height: 40px;
    background: #1a1a2e;
    color: white;
    border: none;
    border-radius: 10px;
    cursor: pointer;
    box-shadow: 0 2px 8px rgba(0,0,0,0.15);
    transition: all 0.2s;
  }

  .expand-btn:hover {
    background: #2d2d4e;
    transform: scale(1.05);
  }

  /* 汉堡按钮（默认隐藏） */
  .hamburger {
    display: none;
    position: fixed;
    top: 12px;
    left: 12px;
    z-index: 1001;
    background: #1a1a2e;
    color: white;
    border: none;
    border-radius: 10px;
    padding: 10px;
    cursor: pointer;
    box-shadow: 0 2px 8px rgba(0,0,0,0.15);
    transition: all 0.2s;
  }

  .hamburger:hover {
    background: #2d2d4e;
  }

  /* 移动端遮罩 */
  .mobile-overlay {
    display: none;
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0,0,0,0.4);
    z-index: 999;
  }

  .sidebar {
    width: 220px;
    background: #1a1a2e;
    color: white;
    display: flex;
    flex-direction: column;
    flex-shrink: 0;
    transition: width 0.25s cubic-bezier(0.4, 0, 0.2, 1), opacity 0.2s ease;
    overflow: hidden;
  }

  .collapsed .sidebar {
    width: 0;
    opacity: 0;
    padding: 0;
    border: none;
  }

  .sidebar-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 24px 20px 20px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.1);
  }

  .sidebar-header h2 {
    margin: 0;
    font-size: 22px;
    font-weight: 700;
    color: #4fc3f7;
    letter-spacing: 1px;
    white-space: nowrap;
  }

  .collapse-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    background: none;
    border: none;
    color: #a0aec0;
    cursor: pointer;
    padding: 6px;
    border-radius: 8px;
    transition: all 0.2s;
    flex-shrink: 0;
  }

  .collapse-btn:hover {
    background: rgba(255, 255, 255, 0.1);
    color: white;
  }

  .sidebar-nav {
    flex: 1;
    padding: 16px 12px;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .sidebar-nav a {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px 16px;
    color: #a0aec0;
    text-decoration: none;
    font-size: 15px;
    border-radius: 8px;
    transition: all 0.2s ease;
    white-space: nowrap;
  }

  .sidebar-nav a:hover {
    background: rgba(79, 195, 247, 0.1);
    color: #e2e8f0;
  }

  .sidebar-nav a.active {
    background: #4fc3f7;
    color: #1a1a2e;
    font-weight: 600;
  }

  .nav-icon {
    font-size: 18px;
  }

  /* 底部设置 */
  .sidebar-footer {
    padding: 12px;
    border-top: 1px solid rgba(255, 255, 255, 0.1);
  }

  .sidebar-footer a {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px 16px;
    color: #a0aec0;
    text-decoration: none;
    font-size: 15px;
    border-radius: 8px;
    transition: all 0.2s ease;
    white-space: nowrap;
  }

  .sidebar-footer a:hover {
    background: rgba(79, 195, 247, 0.1);
    color: #e2e8f0;
  }

  .sidebar-footer a.active {
    background: #4fc3f7;
    color: #1a1a2e;
    font-weight: 600;
  }

  .settings-icon {
    flex-shrink: 0;
  }

  .content {
    flex: 1;
    overflow-y: auto;
    background: #f0f2f5;
    padding: 32px;
    transition: padding 0.25s cubic-bezier(0.4, 0, 0.2, 1);
  }

  /* ===== 响应式：小屏幕 ===== */
  @media (max-width: 768px) {
    .expand-btn {
      display: none;  /* 小屏幕用汉堡菜单 */
    }

    .hamburger {
      display: flex;
      align-items: center;
      justify-content: center;
    }

    .mobile-overlay {
      display: block;
    }

    .sidebar {
      position: fixed;
      top: 0;
      left: 0;
      bottom: 0;
      z-index: 1000;
      width: 220px;
      transform: translateX(-100%);
      transition: transform 0.25s cubic-bezier(0.4, 0, 0.2, 1);
      opacity: 1;
    }

    .sidebar.open {
      transform: translateX(0);
    }

    .collapsed .sidebar {
      width: 220px;
      opacity: 1;
    }

    .content {
      padding: 20px;
      padding-top: 60px;
    }
  }
</style>
