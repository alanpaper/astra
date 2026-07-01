<script lang="ts">
  import { page } from '$app/stores';
  import '../styles/global.css';

  const menuItems = [
    { id: 'workspace', label: '工作空间', path: '/' },
    { id: 'skills', label: 'Skills管理', path: '/skills' }
  ];
</script>

<div class="app-layout">
  <!-- 侧边栏 -->
  <aside class="sidebar">
    <div class="sidebar-header">
      <h2>AiTool</h2>
    </div>
    <nav class="sidebar-nav">
      {#each menuItems as item}
        <a
          href={item.path}
          class:active={$page.url.pathname === item.path}
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
    <slot />
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

  .sidebar {
    width: 220px;
    background: #1a1a2e;
    color: white;
    display: flex;
    flex-direction: column;
    flex-shrink: 0;
  }

  .sidebar-header {
    padding: 24px 20px 20px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.1);
  }

  .sidebar-header h2 {
    margin: 0;
    font-size: 22px;
    font-weight: 700;
    color: #4fc3f7;
    letter-spacing: 1px;
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
  }
</style>
