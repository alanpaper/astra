<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { onMount } from 'svelte';

  // ===== 类型 =====
  interface SubProject {
    name: string;
    path: string;
  }

  interface ProjectCard {
    name: string;
    path: string;
    has_readme: boolean;
    sub_projects: SubProject[];
  }

  interface EditorSetting {
    name: string;
    command: string;
  }

  interface AppSettings {
    editor: EditorSetting;
    workspace_path: string | null;
  }

  // ===== 状态 =====
  let workspacePath = $state('');
  let projects = $state<ProjectCard[]>([]);
  let loading = $state(false);
  let error = $state('');
  let searchQuery = $state('');
  let editorSetting = $state<EditorSetting>({ name: '', command: '' });

  // ===== 页面加载时自动读取设置并扫描 =====
  onMount(async () => {
    await loadAndScan();
  });

  // ===== 加载设置并扫描 =====
  async function loadAndScan() {
    try {
      const settings = await invoke<AppSettings>('get_settings');
      editorSetting = settings.editor;

      if (settings.workspace_path) {
        workspacePath = settings.workspace_path;
        scanWorkspace(settings.workspace_path);
      }
    } catch (e) {
      console.error('加载设置失败:', e);
    }
  }

  // ===== 扫描工作空间 =====
  async function scanWorkspace(path: string) {
    loading = true;
    error = '';
    searchQuery = '';

    try {
      const result = await invoke<ProjectCard[]>('scan_workspace', { path });
      projects = result;
    } catch (e) {
      error = `扫描失败: ${e}`;
      projects = [];
    } finally {
      loading = false;
    }
  }

  // ===== 点击卡片打开编辑器 =====
  async function openProject(path: string) {
    if (!editorSetting.command) {
      error = '请先在「设置」页面配置默认编辑器';
      return;
    }
    try {
      await invoke('open_in_editor', { path, editorCommand: editorSetting.command });
    } catch (e) {
      error = `打开编辑器失败: ${e}`;
    }
  }

  // ===== 计算属性 =====
  let filteredProjects = $derived.by(() => {
    let list = projects;

    if (searchQuery.trim()) {
      const q = searchQuery.trim().toLowerCase();
      list = list.filter(p =>
        p.name.toLowerCase().includes(q) ||
        p.path.toLowerCase().includes(q)
      );
    }

    return list;
  });

  // ===== 高亮搜索匹配 =====
  function highlight(text: string): string {
    if (!searchQuery.trim()) return text;
    const q = searchQuery.trim();
    const escaped = q.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
    const regex = new RegExp(`(${escaped})`, 'gi');
    return text.replace(regex, '<mark>$1</mark>');
  }

  // ===== 文件夹名称提取 =====
  function folderName(path: string): string {
    const parts = path.replace(/\\/g, '/').split('/');
    return parts[parts.length - 1];
  }

  // ===== 获取文件夹图标颜色 =====
  function folderColor(name: string): string {
    const colors = [
      '#4fc3f7', '#ff7043', '#66bb6a', '#ab47bc',
      '#ffa726', '#26c6da', '#ec407a', '#7e57c2',
      '#8d6e63', '#78909c', '#29b6f6', '#f06292',
    ];
    let hash = 0;
    for (let i = 0; i < name.length; i++) {
      hash = name.charCodeAt(i) + ((hash << 5) - hash);
    }
    return colors[Math.abs(hash) % colors.length];
  }

  // ===== 自动聚焦搜索框 =====
  function focusOnMount(node: HTMLInputElement) {
    node.focus();
  }

  // ===== 新建项目弹窗状态 =====
  let showCreateModal = $state(false);
  let newFolderName = $state('');
  let newProjectName = $state('');
  let creating = $state(false);

  function openCreateModal() {
    newFolderName = '';
    newProjectName = '';
    showCreateModal = true;
  }

  function closeCreateModal() {
    if (creating) return;
    showCreateModal = false;
  }

  async function createNewProject() {
    const folder = newFolderName.trim();
    if (!folder) return;
    creating = true;
    error = '';
    try {
      const newProject = await invoke<ProjectCard>('create_project', {
        workspacePath: workspacePath,
        folderName: folder,
        projectName: newProjectName.trim()
      });
      projects = [...projects, newProject];
      projects.sort((a, b) => a.name.toLowerCase().localeCompare(b.name.toLowerCase()));
      showCreateModal = false;
    } catch (e) {
      error = `创建失败: ${e}`;
    } finally {
      creating = false;
    }
  }
</script>

<div class="workspace-page">
  <!-- 未设置工作空间 -->
  {#if !workspacePath && !loading}
    <div class="no-workspace">
      <div class="no-workspace-icon">📂</div>
      <h3>尚未设置工作空间</h3>
      <p>前往设置页面选择一个目录作为工作空间</p>
      <a href="/settings" class="btn-primary">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/></svg>
        前往设置
      </a>
    </div>
  {/if}

  <!-- 错误提示 -->
  {#if error}
    <div class="error-banner">
      <div class="error-icon">⚠️</div>
      <span>{error}</span>
      <button class="error-dismiss" onclick={() => error = ''}>✕</button>
    </div>
  {/if}

  <!-- 加载状态 -->
  {#if loading}
    <div class="loading-container">
      <div class="loading-card">
        <div class="spinner-ring"></div>
        <div class="loading-text">
          <span class="loading-title">正在扫描工作空间</span>
          <span class="loading-desc">遍历目录并检查 README.md...</span>
        </div>
      </div>
    </div>
  {/if}

  <!-- 扫描结果区域 -->
  {#if !loading && workspacePath}
    <!-- 居中搜索区 -->
    <div class="hero-search">
      <div class="hero-search-inner">
        <div class="hero-search-icon">
          <svg width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="11" cy="11" r="8"/><line x1="21" y1="21" x2="16.65" y2="16.65"/></svg>
        </div>
        <input
          type="text"
          class="hero-search-input"
          placeholder="搜索项目名称..."
          bind:value={searchQuery}
          use:focusOnMount
        />
        {#if searchQuery}
          <button class="hero-search-clear" onclick={() => searchQuery = ''}>
            ✕
          </button>
        {/if}
      </div>
      <div class="hero-meta">
        <span class="hero-count">{projects.length} 个项目</span>
        <span class="hero-dot">·</span>
        <button class="hero-create" onclick={openCreateModal}>
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/></svg>
          新建项目
        </button>
      </div>
    </div>

    <!-- 搜索无结果 -->
    {#if filteredProjects.length === 0 && projects.length > 0}
      <div class="no-results">
        <span class="no-results-icon">🔍</span>
        <h3>未找到匹配的项目</h3>
        <p>尝试使用不同的关键词搜索，或 <button class="link-btn" onclick={() => searchQuery = ''}>清除搜索</button></p>
      </div>
    {/if}

    <!-- 项目卡片网格 -->
    {#if filteredProjects.length > 0}
      <div class="card-grid" style="--card-count: {filteredProjects.length}">
        {#each filteredProjects as project (project.path)}
          <div class="project-card" style="--card-accent: {folderColor(project.name)}" onclick={() => openProject(project.path)} onkeydown={(e) => e.key === 'Enter' && openProject(project.path)} role="button" tabindex="0" title="在 {editorSetting.name || '编辑器'} 中打开">
            <div class="card-accent-bar"></div>
            <div class="card-content">
              <div class="card-header">
                <div class="card-avatar" style="background: {folderColor(project.name)}22; color: {folderColor(project.name)}">
                  {project.name.charAt(0).toUpperCase()}
                </div>
                <div class="card-header-text">
                  <h3 class="card-title">{@html highlight(project.name)}</h3>
                  <span class="card-folder">{@html highlight(folderName(project.path))}</span>
                </div>
              </div>

              <div class="card-path" title={project.path}>
                <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/></svg>
                <span>{project.path}</span>
              </div>

              {#if project.sub_projects?.length > 0}
                <div class="sub-projects">
                  <div class="sub-label">
                    <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/></svg>
                    <span>子项目 ({project.sub_projects.length})</span>
                  </div>
                  <div class="sub-list">
                    {#each project.sub_projects as sub}
                      <button
                        class="sub-item"
                        onclick={(e) => { e.stopPropagation(); openProject(sub.path); }}
                        title="在 {editorSetting.name || '编辑器'} 中打开 {sub.name}"
                      >
                        <span class="sub-item-name">{sub.name}</span>
                      </button>
                    {/each}
                  </div>
                </div>
              {/if}

              <div class="card-footer">
                <div class="footer-right">
                  <span class="open-hint">
                    {#if editorSetting.command}
                      <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polygon points="5 3 19 12 5 21 5 3"/></svg>
                      在 {editorSetting.name} 中打开
                    {/if}
                  </span>
                </div>
              </div>
            </div>
          </div>
        {/each}
      </div>
    {/if}

    <!-- 空工作空间（无项目） -->
      {#if projects.length === 0 && !loading}
        <div class="empty-state">
          <div class="empty-illustration">
            <span class="empty-icon">📭</span>
          </div>
          <h3>这里还没有项目</h3>
          <p>该目录下没有找到任何子文件夹</p>
        </div>
      {/if}
    {/if}

    <!-- ===== 新建项目弹窗 ===== -->
    {#if showCreateModal}
      <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions a11y_roles -->
      <div class="modal-overlay" onclick={closeCreateModal} role="presentation">
        <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions a11y_roles -->
        <div class="modal" onclick={(e) => e.stopPropagation()} role="dialog" aria-label="新建项目" tabindex="-1">
          <div class="modal-header">
            <h2>新建项目</h2>
            <button class="modal-close" onclick={closeCreateModal} disabled={creating} aria-label="关闭">
              <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
            </button>
          </div>
          <div class="modal-body">
            <div class="form-group">
              <label for="folder-name">文件夹名称 <span class="required">*</span></label>
              <input
                id="folder-name"
                type="text"
                placeholder="例如：my-project"
                bind:value={newFolderName}
                disabled={creating}
                onkeydown={(e) => e.key === 'Enter' && createNewProject()}
              />
              <p class="form-hint">将在当前工作空间下创建此名称的子文件夹</p>
            </div>
            <div class="form-group">
              <label for="project-name">项目名称 <span class="required">*</span></label>
              <input
                id="project-name"
                type="text"
                placeholder="输入项目显示名称"
                bind:value={newProjectName}
                disabled={creating}
                onkeydown={(e) => e.key === 'Enter' && createNewProject()}
              />
              <p class="form-hint">项目 README.md 的标题</p>
            </div>
          </div>
          <div class="modal-footer">
            <button class="btn-cancel" onclick={closeCreateModal} disabled={creating}>取消</button>
            <button
              class="btn-confirm"
              onclick={createNewProject}
              disabled={!newFolderName.trim() || !newProjectName.trim() || creating}
            >
              {#if creating}
                <div class="btn-spinner"></div>
                创建中...
              {:else}
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><path d="M20 14.66V20a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2h5.34"/><polygon points="18 2 22 6 12 16 8 16 8 12 18 2"/></svg>
                创建项目
              {/if}
            </button>
          </div>
        </div>
      </div>
    {/if}
  </div>

<style>
  .workspace-page {
    margin: 0 auto;
    animation: fadeIn 0.3s ease;
  }

  @keyframes fadeIn {
    from { opacity: 0; transform: translateY(8px); }
    to { opacity: 1; transform: translateY(0); }
  }

  /* ========== 未设置工作空间 ========== */
  .no-workspace {
    text-align: center;
    padding: 80px 20px;
    background: white;
    border-radius: 16px;
    border: 2px dashed #e2e8f0;
    margin-top: 20px;
  }

  .no-workspace-icon {
    font-size: 56px;
    margin-bottom: 16px;
    display: block;
  }

  .no-workspace h3 {
    font-size: 20px;
    color: #475569;
    margin-bottom: 8px;
  }

  .no-workspace p {
    color: #94a3b8;
    font-size: 15px;
    margin-bottom: 24px;
  }

  .btn-primary {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    padding: 12px 24px;
    background: linear-gradient(135deg, #667eea, #764ba2);
    border: none;
    border-radius: 12px;
    font-size: 15px;
    font-weight: 600;
    color: white;
    text-decoration: none;
    cursor: pointer;
    transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  }

  .btn-primary:hover {
    transform: translateY(-2px);
    box-shadow: 0 8px 20px rgba(102, 126, 234, 0.4);
  }

  /* ========== 错误 ========== */
  .error-banner {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 14px 18px;
    background: #fef2f2;
    border: 1px solid #fecaca;
    border-radius: 12px;
    color: #dc2626;
    margin-bottom: 20px;
    font-size: 14px;
    animation: slideDown 0.3s ease;
  }

  .error-icon {
    font-size: 18px;
    flex-shrink: 0;
  }

  .error-dismiss {
    margin-left: auto;
    background: none;
    border: none;
    color: #fca5a5;
    cursor: pointer;
    padding: 2px 6px;
    border-radius: 4px;
    font-size: 16px;
  }

  .error-dismiss:hover {
    background: #fee2e2;
    color: #dc2626;
  }

  @keyframes slideDown {
    from { opacity: 0; transform: translateY(-12px); }
    to { opacity: 1; transform: translateY(0); }
  }

  /* ========== 加载 ========== */
  .loading-container {
    display: flex;
    justify-content: center;
    padding: 60px 0;
  }

  .loading-card {
    display: flex;
    align-items: center;
    gap: 20px;
    background: white;
    padding: 32px 40px;
    border-radius: 16px;
    box-shadow: 0 2px 8px rgba(0,0,0,0.06);
  }

  .spinner-ring {
    width: 36px;
    height: 36px;
    border: 3px solid #e2e8f0;
    border-top-color: #667eea;
    border-radius: 50%;
    animation: spin 0.8s cubic-bezier(0.4, 0, 0.2, 1) infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .loading-text {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .loading-title {
    font-weight: 600;
    color: #1e293b;
    font-size: 16px;
  }

  .loading-desc {
    color: #94a3b8;
    font-size: 13px;
  }

  /* ========== 居中搜索区 ========== */
  .hero-search {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 60px 20px 40px;
    animation: fadeIn 0.4s ease;
  }

  .hero-search-inner {
    display: flex;
    align-items: center;
    gap: 14px;
    width: 100%;
    max-width: 560px;
    padding: 16px 24px;
    background: white;
    border: 1px solid #e2e8f0;
    border-radius: 16px;
    box-shadow: 0 2px 8px rgba(0,0,0,0.04), 0 8px 24px rgba(0,0,0,0.04);
    transition: all 0.3s ease;
  }

  .hero-search-inner:focus-within {
    border-color: #667eea;
    box-shadow: 0 2px 8px rgba(102, 126, 234, 0.08), 0 8px 32px rgba(102, 126, 234, 0.12);
  }

  .hero-search-icon {
    color: #94a3b8;
    flex-shrink: 0;
    display: flex;
  }

  .hero-search-input {
    flex: 1;
    border: none;
    outline: none;
    font-size: 20px;
    font-weight: 500;
    color: #1e293b;
    background: none;
    min-width: 0;
  }

  .hero-search-input::placeholder {
    color: #cbd5e1;
    font-weight: 400;
  }

  .hero-search-clear {
    background: none;
    border: none;
    color: #94a3b8;
    cursor: pointer;
    padding: 4px 8px;
    border-radius: 8px;
    font-size: 18px;
    flex-shrink: 0;
    transition: all 0.2s;
  }

  .hero-search-clear:hover {
    color: #475569;
    background: #f1f5f9;
  }

  .hero-meta {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-top: 16px;
    font-size: 14px;
    color: #94a3b8;
  }

  .hero-count {
    font-weight: 500;
  }

  .hero-dot {
    color: #e2e8f0;
  }

  .hero-create {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    background: none;
    border: none;
    font-size: 14px;
    font-weight: 500;
    color: #667eea;
    cursor: pointer;
    padding: 4px 8px;
    border-radius: 6px;
    transition: all 0.2s;
  }

  .hero-create:hover {
    background: #eef2ff;
    color: #4f46e5;
  }

  /* ========== 搜索无结果 ========== */
  .no-results {
    text-align: center;
    padding: 40px 20px;
    background: white;
    border-radius: 14px;
    border: 1px solid #f1f5f9;
    margin: 8px auto;
    max-width: 1100px;
  }

  .no-results-icon {
    font-size: 40px;
    display: block;
    margin-bottom: 12px;
  }

  .no-results h3 {
    font-size: 16px;
    color: #475569;
    margin-bottom: 6px;
  }

  .no-results p {
    color: #94a3b8;
    font-size: 14px;
  }

  .link-btn {
    background: none;
    border: none;
    color: #6366f1;
    cursor: pointer;
    font-size: 14px;
    font-weight: 500;
    text-decoration: underline;
    text-underline-offset: 2px;
  }

  .link-btn:hover {
    color: #4f46e5;
  }

  /* ========== 卡片网格 ========== */
  .card-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
    gap: 16px;
    padding: 12px 32px 32px;
    max-width: 1100px;
    margin: 0 auto;
    animation: fadeIn 0.4s ease;
  }

  .project-card {
    background: white;
    border-radius: 14px;
    overflow: hidden;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.04);
    border: 1px solid #f1f5f9;
    transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
    display: flex;
    flex-direction: column;
    cursor: pointer;
    outline: none;
  }

  .project-card:hover {
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.08);
    transform: translateY(-3px);
    border-color: transparent;
  }

  .project-card:focus-visible {
    box-shadow: 0 0 0 3px rgba(99, 102, 241, 0.3);
    transform: translateY(-2px);
  }

  .card-accent-bar {
    height: 4px;
    background: var(--card-accent, #667eea);
    flex-shrink: 0;
  }

  .card-content {
    padding: 20px;
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 14px;
  }

  .card-header {
    display: flex;
    align-items: center;
    gap: 14px;
  }

  .card-avatar {
    width: 42px;
    height: 42px;
    border-radius: 12px;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 18px;
    font-weight: 700;
    flex-shrink: 0;
  }

  .card-header-text {
    min-width: 0;
    flex: 1;
  }

  .card-title {
    font-size: 16px;
    font-weight: 600;
    color: #1e293b;
    margin: 0;
    line-height: 1.3;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .card-title :global(mark) {
    background: #fef08a;
    color: #1e293b;
    border-radius: 3px;
    padding: 0 2px;
  }

  .card-folder {
    font-size: 12px;
    color: #94a3b8;
    font-family: ui-monospace, monospace;
  }

  .card-folder :global(mark) {
    background: #fef08a;
    color: #1e293b;
    border-radius: 3px;
    padding: 0 2px;
  }

  .card-path {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 12px;
    color: #94a3b8;
    padding: 8px 10px;
    background: #f8fafc;
    border-radius: 8px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    border: 1px solid #f1f5f9;
  }

  .card-path svg {
    flex-shrink: 0;
  }

  .card-path span {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  /* ===== 子项目 ===== */
  .sub-projects {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .sub-label {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 11px;
    color: #94a3b8;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .sub-label svg {
    flex-shrink: 0;
  }

  .sub-list {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }

  .sub-item {
    display: inline-flex;
    align-items: center;
    padding: 4px 10px;
    background: #f0f4ff;
    border: 1px solid #dbeafe;
    border-radius: 6px;
    font-size: 12px;
    font-weight: 500;
    color: #3b82f6;
    cursor: pointer;
    transition: all 0.15s ease;
    font-family: ui-monospace, monospace;
  }

  .sub-item:hover {
    background: #dbeafe;
    border-color: #93c5fd;
    color: #2563eb;
  }

  .sub-item:active {
    transform: scale(0.97);
  }

  .card-footer {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    margin-top: auto;
  }

  .footer-right {
    display: flex;
    align-items: center;
  }

  .open-hint {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    font-size: 11px;
    color: #a0aec0;
    opacity: 0;
    transition: opacity 0.2s ease;
  }

  .project-card:hover .open-hint {
    opacity: 1;
  }

  /* ========== 空状态 ========== */
  .empty-state {
    text-align: center;
    padding: 64px 20px;
    background: white;
    border-radius: 16px;
    border: 1px solid #f1f5f9;
  }

  .empty-illustration {
    width: 80px;
    height: 80px;
    background: #f1f5f9;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    margin: 0 auto 16px;
  }

  .empty-icon {
    font-size: 36px;
  }

  .empty-state h3 {
    font-size: 18px;
    color: #475569;
    margin-bottom: 6px;
  }

  .empty-state p {
    color: #94a3b8;
    font-size: 14px;
  }

  /* ========== 弹窗遮罩 ========== */
  .modal-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
    animation: fadeInOverlay 0.2s ease;
  }

  @keyframes fadeInOverlay {
    from { opacity: 0; }
    to { opacity: 1; }
  }

  .modal {
    background: white;
    border-radius: 20px;
    padding: 0;
    width: 480px;
    max-width: 90vw;
    box-shadow: 0 24px 48px rgba(0, 0, 0, 0.2);
    animation: slideUp 0.25s cubic-bezier(0.4, 0, 0.2, 1);
  }

  @keyframes slideUp {
    from { opacity: 0; transform: translateY(16px) scale(0.97); }
    to { opacity: 1; transform: translateY(0) scale(1); }
  }

  .modal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 20px 24px 0;
  }

  .modal-header h2 {
    font-size: 20px;
    font-weight: 700;
    color: #1e293b;
  }

  .modal-close {
    background: none;
    border: none;
    color: #94a3b8;
    cursor: pointer;
    padding: 6px;
    border-radius: 8px;
    transition: all 0.2s;
  }

  .modal-close:hover:not(:disabled) {
    background: #f1f5f9;
    color: #475569;
  }

  .modal-close:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .modal-body {
    padding: 20px 24px;
    display: flex;
    flex-direction: column;
    gap: 18px;
  }

  .form-group {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .form-group label {
    font-size: 14px;
    font-weight: 600;
    color: #475569;
  }

  .required {
    color: #ef4444;
  }

  .form-group input {
    padding: 10px 14px;
    border: 1px solid #e2e8f0;
    border-radius: 10px;
    font-size: 14px;
    outline: none;
    transition: all 0.2s;
  }

  .form-group input:focus {
    border-color: #667eea;
    box-shadow: 0 0 0 3px rgba(102, 126, 234, 0.1);
  }

  .form-group input:disabled {
    background: #f8fafc;
    cursor: not-allowed;
  }

  .form-hint {
    font-size: 12px;
    color: #94a3b8;
  }

  .modal-footer {
    display: flex;
    justify-content: flex-end;
    gap: 10px;
    padding: 16px 24px 20px;
  }

  .btn-cancel {
    padding: 10px 20px;
    background: #f8fafc;
    border: 1px solid #e2e8f0;
    border-radius: 10px;
    font-size: 14px;
    font-weight: 500;
    color: #475569;
    cursor: pointer;
    transition: all 0.2s;
  }

  .btn-cancel:hover:not(:disabled) {
    background: #f1f5f9;
  }

  .btn-cancel:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .btn-confirm {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    padding: 10px 22px;
    background: linear-gradient(135deg, #667eea, #764ba2);
    border: none;
    border-radius: 10px;
    font-size: 14px;
    font-weight: 600;
    color: white;
    cursor: pointer;
    transition: all 0.2s ease;
    box-shadow: 0 2px 6px rgba(102, 126, 234, 0.3);
  }

  .btn-confirm:hover:not(:disabled) {
    transform: translateY(-1px);
    box-shadow: 0 4px 12px rgba(102, 126, 234, 0.4);
  }

  .btn-confirm:disabled {
    opacity: 0.6;
    cursor: not-allowed;
    box-shadow: none;
    transform: none;
  }

  .btn-spinner {
    width: 16px;
    height: 16px;
    border: 2px solid rgba(255,255,255,0.3);
    border-top-color: white;
    border-radius: 50%;
    animation: spin 0.7s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }
</style>
