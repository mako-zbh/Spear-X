import { invoke } from '@tauri-apps/api/core'

// === 类型定义 ===

export interface Tool {
  id?: string
  name: string
  path: string
  fileName: string
  value: string
  command: string
  optional: string
  description?: string
  tags: string[]
  sourceUrl?: string
  iconPath?: string
  openCount: number
  createdAt?: string
  lastUsedAt?: string
}

export interface Category {
  name: string
  icon?: string
  tools: Tool[]
}

export interface Categories {
  categories: Category[]
}

export interface JavaConfig {
  Java8: string
  Java11: string
  Java17: string
}

export interface ScannedTool {
  path: string
  category: string
  possibleFiles: string[]
}

export interface FileInfo {
  name: string
  isDir: boolean
  size: number
  modTime: string
  path: string
  extension: string
  isExecutable: boolean
}

export interface CleanupResult {
  invalidToolsCount: number
  invalidCategoriesCount: number
  cleanedNotes: number
  migratedNotes: number
  invalidToolNames: string[]
  migratedToolNames: string[]
}

// === 配置 ===

export const getCategories = () => invoke<Categories>('get_categories')
export const getJavaConfig = () => invoke<JavaConfig | null>('get_java_config')
export const saveJavaConfig = (config: JavaConfig) => invoke('save_java_config', { configData: config })

// === 执行 ===

export const executeCommand = (path: string, optional: string, value: string, filename: string) =>
  invoke('execute_command', { path, optional, value, filename })

export const executeCustomCommand = (path: string, optional: string, value: string, filename: string, customCommand: string) =>
  invoke('execute_custom_command', { path, optional, value, filename, customCommand })

export const executeCommandWithCustom = (path: string, optional: string, value: string, filename: string, customCommand: string, javaPath: string) =>
  invoke('execute_command_with_custom', { path, optional, value, filename, customCommand, javaPath })

export const executeToolCommand = (tool: Tool, customCommand: string) =>
  invoke('execute_tool_command', { tool, customCommand })

// === 工具 CRUD ===

export const addTool = (tool: Tool, categoryName: string) =>
  invoke('add_tool', { tool, categoryName })

export const deleteTool = (toolName: string, categoryName: string) =>
  invoke('delete_tool', { toolName, categoryName })

export const updateTool = (originalName: string, categoryName: string, tool: Tool) =>
  invoke('update_tool', { originalName, categoryName, tool })

export const updateToolDescription = (toolName: string, categoryName: string, description: string) =>
  invoke('update_tool_description', { toolName, categoryName, description })

export const searchTools = (query: string) =>
  invoke<Tool[]>('search_tools', { query })

export const getAllTags = () =>
  invoke<string[]>('get_all_tags')

export const getToolTypes = () =>
  invoke<string[]>('get_tool_types')

export const getToolAbsolutePath = (toolPath: string, fileName: string) =>
  invoke<string>('get_tool_absolute_path', { toolPath, fileName })

export const getNewToolsFromScanned = (tools: ScannedTool[]) =>
  invoke<ScannedTool[]>('get_new_tools_from_scanned', { tools })

export const autoAddScannedTools = (tools: ScannedTool[]) =>
  invoke('auto_add_scanned_tools', { tools })

// === 分类 ===

export const addCategory = (categoryName: string) =>
  invoke('add_category', { categoryName })

export const deleteCategory = (categoryName: string) =>
  invoke('delete_category', { categoryName })

export const updateCategoryTools = (categoryName: string, tools: Tool[]) =>
  invoke('update_category_tools', { categoryName, tools })

export const updateCategoryName = (oldName: string, newName: string) =>
  invoke('update_category_name', { oldName, newName })

export const updateCategoriesOrder = (orderedCategories: Category[]) =>
  invoke('update_categories_order', { orderedCategories })

export const updateCategoryIcon = (categoryName: string, icon: string) =>
  invoke('update_category_icon', { categoryName, icon })

// === 扫描 ===

export const scanResourcesForTools = () =>
  invoke<ScannedTool[]>('scan_resources_for_tools')

export const scanCustomDirectoryForTools = (customPath: string) =>
  invoke<ScannedTool[]>('scan_custom_directory_for_tools', { customPath })

export const scanToolsInPath = (scanPath: string) =>
  invoke<ScannedTool[]>('scan_tools_in_path', { scanPath })

export const scanToolsInCustomPath = (scanPath: string) =>
  invoke<ScannedTool[]>('scan_tools_in_custom_path', { scanPath })

export const cleanInvalidPaths = () =>
  invoke<CleanupResult>('clean_invalid_paths')

// === 笔记 ===

export const getToolNote = (toolPath: string, toolName: string) =>
  invoke<string>('get_tool_note', { toolPath, toolName })

export const saveToolNote = (toolPath: string, toolName: string, content: string) =>
  invoke('save_tool_note', { toolPath, toolName, content })

export const deleteToolNote = (toolPath: string, toolName: string) =>
  invoke('delete_tool_note', { toolPath, toolName })

// === 对话框 ===

export const openFileDialog = () =>
  invoke<Record<string, string> | null>('open_file_dialog')

export const openDirectoryDialog = () =>
  invoke<string>('open_directory_dialog')

export const selectDirectory = () =>
  invoke<string>('select_directory')

export const selectFile = () =>
  invoke<string>('select_file')

export const selectJavaPath = () =>
  invoke<string>('select_java_path')

export const select = (selectFolder: boolean) =>
  invoke<string>('select', { selectFolder })

// === 文件浏览 ===

export const browseDirectory = (pathInput: string) =>
  invoke<FileInfo[]>('browse_directory', { pathInput })

export const getToolDirectory = (toolPath: string) =>
  invoke<FileInfo[]>('get_tool_directory', { toolPath })

export const getFileInfo = (filePath: string) =>
  invoke<Record<string, string>>('get_file_info', { filePath })

export const getFilePath = (fileName: string) =>
  invoke<string>('get_file_path', { fileName })

// === 维护 ===

export const cleanupToolPaths = () => invoke('cleanup_tool_paths_cmd')
export const repairConfigFile = () => invoke('repair_config_file_cmd')
export const cleanupDuplicateTools = () => invoke('cleanup_duplicate_tools_cmd')
export const debugAllToolPaths = () => invoke('debug_all_tool_paths_cmd')

// === 杂项 ===

export const openToolDirectory = (path: string) =>
  invoke('open_tool_directory', { path })

export const openGitHubPage = () => invoke('open_github_page')

export const getFileTypes = () =>
  invoke<Record<string, string>[]>('get_file_types')
