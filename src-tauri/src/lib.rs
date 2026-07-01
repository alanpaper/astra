use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::process::Command;
use tauri::Manager;

// ===== 类型定义 =====

#[derive(Debug, Serialize, Clone)]
struct SubProject {
    name: String,
    path: String,
}

#[derive(Debug, Serialize)]
struct ProjectCard {
    name: String,
    path: String,
    has_readme: bool,
    sub_projects: Vec<SubProject>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct EditorSetting {
    name: String,
    command: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct AppSettings {
    editor: EditorSetting,
    workspace_path: Option<String>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            editor: EditorSetting {
                name: "VS Code".to_string(),
                command: "code".to_string(),
            },
            workspace_path: None,
        }
    }
}

// ===== 设置文件路径 =====

fn get_settings_path(app: &tauri::AppHandle) -> Result<std::path::PathBuf, String> {
    let config_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("获取应用数据目录失败: {}", e))?;
    fs::create_dir_all(&config_dir).map_err(|e| format!("创建配置目录失败: {}", e))?;
    Ok(config_dir.join("settings.json"))
}

// ===== 命令：扫描工作空间 =====

#[tauri::command]
fn scan_workspace(path: String) -> Result<Vec<ProjectCard>, String> {
    let dir = Path::new(&path);

    if !dir.is_dir() {
        return Err(format!("路径不是有效的目录: {}", path));
    }

    let entries = fs::read_dir(dir).map_err(|e| format!("读取目录失败: {}", e))?;

    let mut projects = Vec::new();

    for entry in entries {
        let entry = entry.map_err(|e| format!("读取条目失败: {}", e))?;
        let entry_path = entry.path();

        if !entry_path.is_dir() {
            continue;
        }

        // 跳过隐藏目录（以 . 开头）
        let folder_name = entry_path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        if folder_name.starts_with('.') {
            continue;
        }

        // 检查 README.md
        let readme_path = entry_path.join("README.md");
        let has_readme = readme_path.exists();

        let project_name = if has_readme {
            // 读取 README.md 的第一行作为项目名称
            match fs::read_to_string(&readme_path) {
                Ok(content) => content
                    .lines()
                    .next()
                    .unwrap_or(&folder_name)
                    .trim()
                    .trim_start_matches("# ")
                    .trim()
                    .to_string(),
                Err(_) => folder_name.clone(),
            }
        } else {
            // 如果没有 README.md，自动生成一个
            let readme_content =
                format!("# {}\n\nThis is the {} project.", folder_name, folder_name);
            if let Err(e) = fs::write(&readme_path, &readme_content) {
                eprintln!("创建 README.md 失败: {}", e);
            }
            folder_name.clone()
        };

        projects.push(ProjectCard {
            name: project_name,
            path: entry_path.to_string_lossy().to_string(),
            has_readme,
            sub_projects: scan_sub_projects(&entry_path),
        });
    }

    // 按名称排序
    projects.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

    Ok(projects)
}

// ===== 扫描项目内的 casp/ids 子目录 =====

fn scan_sub_projects(project_dir: &Path) -> Vec<SubProject> {
    let mut subs = Vec::new();

    let entries = match fs::read_dir(project_dir) {
        Ok(e) => e,
        Err(_) => return subs,
    };

    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        let entry_path = entry.path();

        if !entry_path.is_dir() {
            continue;
        }

        let folder_name = match entry_path.file_name() {
            Some(n) => n.to_string_lossy().to_string(),
            None => continue,
        };

        let lower = folder_name.to_lowercase();
        if lower.starts_with("casp") || lower.starts_with("ids") {
            subs.push(SubProject {
                name: folder_name,
                path: entry_path.to_string_lossy().to_string(),
            });
        }
    }

    subs.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    subs
}

// ===== 命令：创建新项目 =====

#[tauri::command]
fn create_project(
    workspace_path: String,
    folder_name: String,
    project_name: String,
) -> Result<ProjectCard, String> {
    // 验证文件夹名称
    if folder_name.trim().is_empty() {
        return Err("文件夹名称不能为空".to_string());
    }

    let folder_name = folder_name.trim();

    // 不允许以 . 开头（隐藏目录）
    if folder_name.starts_with('.') {
        return Err("文件夹名称不能以 . 开头".to_string());
    }

    // 不允许包含路径分隔符
    if folder_name.contains('/') || folder_name.contains('\\') {
        return Err("文件夹名称不能包含路径分隔符".to_string());
    }

    let project_dir = Path::new(&workspace_path).join(folder_name);

    // 检查目录是否已存在
    if project_dir.exists() {
        return Err(format!("文件夹 '{}' 已存在", folder_name));
    }

    // 创建目录
    fs::create_dir_all(&project_dir).map_err(|e| format!("创建目录失败: {}", e))?;

    let project_name = if project_name.trim().is_empty() {
        folder_name.to_string()
    } else {
        project_name.trim().to_string()
    };

    // 写入 README.md
    let readme_content = format!("# {}\n\nThis is the {} project.", project_name, folder_name);
    let readme_path = project_dir.join("README.md");
    fs::write(&readme_path, &readme_content).map_err(|e| format!("创建 README.md 失败: {}", e))?;

    Ok(ProjectCard {
        name: project_name,
        path: project_dir.to_string_lossy().to_string(),
        has_readme: true,
        sub_projects: scan_sub_projects(&project_dir),
    })
}

// ===== 命令：获取设置 =====

#[tauri::command]
fn get_settings(app: tauri::AppHandle) -> AppSettings {
    let path = match get_settings_path(&app) {
        Ok(p) => p,
        Err(_) => return AppSettings::default(),
    };

    if !path.exists() {
        return AppSettings::default();
    }

    match fs::read_to_string(&path) {
        Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
        Err(_) => AppSettings::default(),
    }
}

// ===== 命令：保存设置 =====

#[tauri::command]
fn save_settings(app: tauri::AppHandle, settings: AppSettings) -> Result<(), String> {
    let path = get_settings_path(&app)?;
    let json =
        serde_json::to_string_pretty(&settings).map_err(|e| format!("序列化设置失败: {}", e))?;
    fs::write(&path, json).map_err(|e| format!("写入设置文件失败: {}", e))?;
    Ok(())
}

// ===== 命令：在编辑器中打开项目 =====

#[tauri::command]
fn open_in_editor(path: String, editor_command: String) -> Result<(), String> {
    Command::new(&editor_command)
        .arg(&path)
        .spawn()
        .map_err(|e| {
            format!(
                "启动编辑器失败: {}（请确认 {} 已安装且在系统 PATH 中）",
                e, editor_command
            )
        })?;
    Ok(())
}

// ===== 命令：获取预定义编辑器列表 =====

#[tauri::command]
fn get_preset_editors() -> Vec<EditorSetting> {
    vec![
        EditorSetting {
            name: "VS Code".to_string(),
            command: "code".to_string(),
        },
        EditorSetting {
            name: "Zed".to_string(),
            command: "zed".to_string(),
        },
        EditorSetting {
            name: "Cursor".to_string(),
            command: "cursor".to_string(),
        },
        EditorSetting {
            name: "WebStorm".to_string(),
            command: "webstorm".to_string(),
        },
        EditorSetting {
            name: "IntelliJ IDEA".to_string(),
            command: "idea".to_string(),
        },
        EditorSetting {
            name: "Sublime Text".to_string(),
            command: "subl".to_string(),
        },
        EditorSetting {
            name: "Atom".to_string(),
            command: "atom".to_string(),
        },
        EditorSetting {
            name: "Windsurf".to_string(),
            command: "windsurf".to_string(),
        },
    ]
}

// ===== 应用入口 =====

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            scan_workspace,
            create_project,
            get_settings,
            save_settings,
            open_in_editor,
            get_preset_editors,
            list_skills,
            delete_skill,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// ===== 保留默认 greet =====

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

// ===== Skills 管理 =====

#[derive(Debug, Serialize)]
struct SkillCard {
    name: String,
    description: String,
    path: String,
    version: String,
}

/// 解析 SKILL.md 的 YAML frontmatter
fn parse_skill_frontmatter(content: &str) -> (String, String, String) {
    let mut name = String::new();
    let mut description = String::new();
    let mut version = String::new();

    let content = content.trim();
    if !content.starts_with("---") {
        return (name, description, version);
    }

    let after_first = content.strip_prefix("---").unwrap_or("");
    if let Some(end) = after_first.find("---") {
        let frontmatter = &after_first[..end];
        let mut in_description = false;
        let mut desc_parts: Vec<String> = Vec::new();

        for line in frontmatter.lines() {
            if in_description {
                let trimmed = line.trim();
                if trimmed.is_empty()
                    || !trimmed.starts_with(' ')
                        && !trimmed.starts_with(|c: char| c.is_whitespace())
                {
                    // check if this is a new key
                    if trimmed.contains(':') && !trimmed.starts_with(' ') {
                        in_description = false;
                        // parse as key... but just fall through
                    } else {
                        desc_parts.push(trimmed.to_string());
                    }
                } else {
                    desc_parts.push(trimmed.to_string());
                }
                continue;
            }

            if let Some(val) = line.strip_prefix("name:") {
                name = val.trim().trim_matches('"').to_string();
            } else if let Some(val) = line.strip_prefix("description:") {
                let val = val.trim();
                if val.starts_with('"') {
                    description = val.trim_matches('"').to_string();
                } else if val.starts_with(">-") || val.starts_with('|') {
                    in_description = true;
                } else if !val.is_empty() {
                    description = val.to_string();
                } else {
                    in_description = true;
                }
            } else if let Some(val) = line.strip_prefix("version:") {
                version = val.trim().trim_matches('"').to_string();
            }
        }

        if !desc_parts.is_empty() {
            description = desc_parts.join(" ");
        }
    }

    (name, description, version)
}

#[tauri::command]
fn list_skills() -> Result<Vec<SkillCard>, String> {
    let home = dirs::home_dir().ok_or("无法获取用户主目录")?;
    let skills_dir = home.join(".agents").join("skills");

    if !skills_dir.exists() {
        return Ok(Vec::new());
    }

    let entries = fs::read_dir(&skills_dir).map_err(|e| format!("读取 skills 目录失败: {}", e))?;

    let mut skills = Vec::new();

    for entry in entries {
        let entry = entry.map_err(|e| format!("读取条目失败: {}", e))?;
        let entry_path = entry.path();

        if !entry_path.is_dir() {
            continue;
        }

        let folder_name = entry_path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        if folder_name.starts_with('.') {
            continue;
        }

        let skill_md_path = entry_path.join("SKILL.md");
        let (name, description, version) = if skill_md_path.exists() {
            match fs::read_to_string(&skill_md_path) {
                Ok(content) => parse_skill_frontmatter(&content),
                Err(_) => (folder_name.clone(), String::new(), String::new()),
            }
        } else {
            (folder_name.clone(), String::new(), String::new())
        };

        skills.push(SkillCard {
            name: if name.is_empty() { folder_name } else { name },
            description,
            path: entry_path.to_string_lossy().to_string(),
            version,
        });
    }

    skills.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    Ok(skills)
}

#[tauri::command]
fn delete_skill(path: String) -> Result<(), String> {
    let dir = Path::new(&path);
    if !dir.exists() {
        return Err("技能目录不存在".to_string());
    }
    fs::remove_dir_all(dir).map_err(|e| format!("删除失败: {}", e))?;
    Ok(())
}
