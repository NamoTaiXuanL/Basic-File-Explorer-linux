use eframe::egui;
use std::path::{Path, PathBuf};
use std::fs;
use crate::utils;

pub struct DirectoryTree {
    tree_nodes: Vec<TreeNode>,
    expanded_dirs: std::collections::HashSet<PathBuf>,
}


#[derive(Clone)]
struct TreeNode {
    path: PathBuf,
    name: String,
    is_dir: bool,
    children: Vec<TreeNode>,
}

impl DirectoryTree {
    pub fn new() -> Self {
        Self {
            tree_nodes: Vec::new(),
            expanded_dirs: std::collections::HashSet::new(),
        }
    }

    pub fn refresh(&mut self, root_path: &Path) {
        self.tree_nodes.clear();
        // 只加载第一层子目录，大幅减少IO操作
        if let Some(node) = self.build_tree_node(root_path, 2) {
            self.tree_nodes.push(node);
        }
    }

    fn build_tree_node(&self, path: &Path, max_depth: usize) -> Option<TreeNode> {
        if max_depth == 0 {
            return None;
        }

        let name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("未知")
            .to_string();

        let is_dir = path.is_dir();
        let mut children = Vec::new();

        // 大幅优化：只在第一层加载目录，子目录延迟加载
        if is_dir && max_depth == 2 {
            if let Ok(entries) = fs::read_dir(path) {
                let mut dir_count = 0;
                const MAX_DIRS_PER_LEVEL: usize = 50; // 限制每个目录最多显示的子目录数

                for entry in entries.flatten() {
                    if dir_count >= MAX_DIRS_PER_LEVEL {
                        break; // 限制目录数量，避免性能问题
                    }

                    let entry_path = entry.path();
                    if entry_path.is_dir() {
                        // 只添加占位符节点，不递归加载
                        let child_name = entry_path
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("未知")
                            .to_string();

                        children.push(TreeNode {
                            path: entry_path,
                            name: child_name,
                            is_dir: true,
                            children: Vec::new(), // 不预加载子目录
                        });

                        dir_count += 1;
                    }
                }
            }
        }

        Some(TreeNode {
            path: path.to_path_buf(),
            name,
            is_dir,
            children,
        })
    }

    pub fn show(&mut self, ui: &mut egui::Ui, current_path: &mut PathBuf) -> bool {
        let mut should_navigate = false;
        let nodes = self.tree_nodes.clone(); // 简单克隆，避免借用问题

        egui::ScrollArea::vertical().show(ui, |ui| {
            for node in &nodes {
                if self.show_node_simple(ui, node, 0, current_path, &mut should_navigate) {
                    should_navigate = true;
                }
            }
        });

        should_navigate
    }

    fn show_node_simple(
        &mut self,
        ui: &mut egui::Ui,
        node: &TreeNode,
        depth: usize,
        current_path: &mut PathBuf,
        should_navigate: &mut bool,
    ) -> bool {
        let is_selected = current_path == &node.path;
        let is_expanded = self.expanded_dirs.contains(&node.path);

        // 完全模仿内容框的按钮逻辑
        let button_response = ui.add_sized(
            [ui.available_width(), ui.spacing().interact_size.y * 1.5],
            egui::Button::new({
                let indent = "  ".repeat(depth);

                let icon = if node.is_dir {
                    if is_expanded {
                        "📂"
                    } else {
                        "📁"
                    }
                } else {
                    "📄"
                };

                format!("{}{} {}", indent, icon, node.name)
            })
            .fill(if is_selected { ui.visuals().widgets.inactive.bg_fill } else { egui::Color32::TRANSPARENT })
            .stroke(if is_selected {
                egui::Stroke::new(1.0, ui.visuals().widgets.active.fg_stroke.color)
            } else {
                egui::Stroke::NONE
            })
        );

        // 完全模仿内容框的点击处理
        if button_response.clicked() && node.is_dir {
            *current_path = node.path.clone();
            *should_navigate = true;
        }

        // 双击展开/折叠
        if button_response.double_clicked() && node.is_dir {
            if is_expanded {
                self.expanded_dirs.remove(&node.path);
            } else {
                self.expanded_dirs.insert(node.path.clone());
            }
        }

        // 显示子节点
        if node.is_dir && is_expanded {
            for child in &node.children {
                if self.show_node_simple(ui, child, depth + 1, current_path, should_navigate) {
                    *should_navigate = true;
                }
            }
        }

        *should_navigate
    }

    
    
    pub fn ensure_path_loaded(&mut self, path: &Path) {
        // 只展开路径，不重新构建整个目录树
        self.expand_to_path(path);
    }

    pub fn expand_to_path(&mut self, path: &Path) {
        let mut current = path.to_path_buf();
        while let Some(parent) = current.parent() {
            self.expanded_dirs.insert(parent.to_path_buf());
            current = parent.to_path_buf();
        }
    }
}