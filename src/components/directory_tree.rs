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
        // 限制初始深度为3，避免过深的递归
        if let Some(node) = self.build_tree_node(root_path, 3) {
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

        // 限制递归深度，避免无限循环
        if is_dir {
            if let Ok(entries) = fs::read_dir(path) {
                for entry in entries.flatten() {
                    let entry_path = entry.path();
                    if entry_path.is_dir() {
                        if let Some(child_node) = self.build_tree_node(&entry_path, max_depth - 1) {
                            children.push(child_node);
                        }
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

        egui::ScrollArea::vertical().show(ui, |ui| {
            // 直接使用引用，避免克隆
            let nodes = &self.tree_nodes;
            for node in nodes {
                if self.show_node(ui, node, 0, current_path, &mut should_navigate) {
                    should_navigate = true;
                }
            }
        });

        should_navigate
    }

    fn show_node(
        &mut self,
        ui: &mut egui::Ui,
        node: &TreeNode,
        depth: usize,
        current_path: &mut PathBuf,
        should_navigate: &mut bool,
    ) -> bool {
        let is_current = current_path == &node.path;

        // 整行按钮，使用与内容框相同的点击逻辑
        let button_response = ui.add_sized(
            [ui.available_width(), ui.spacing().interact_size.y * 1.5],
            egui::Button::new({
                let indent = "  ".repeat(depth);

                let icon = if node.is_dir {
                    if self.expanded_dirs.contains(&node.path) {
                        "📂"
                    } else {
                        "📁"
                    }
                } else {
                    "📄"
                };

                format!("{}{} {}", indent, icon, node.name)
            })
            .fill(if is_current {
                ui.visuals().widgets.inactive.bg_fill
            } else {
                egui::Color32::TRANSPARENT
            })
            .stroke(if is_current {
                egui::Stroke::new(1.0, ui.visuals().widgets.active.fg_stroke.color)
            } else {
                egui::Stroke::NONE
            })
        );

        // 处理点击事件
        if button_response.clicked() && node.is_dir {
            *current_path = node.path.clone();
            *should_navigate = true;
        }

        // 处理双击展开/折叠
        if button_response.double_clicked() && node.is_dir {
            if self.expanded_dirs.contains(&node.path) {
                self.expanded_dirs.remove(&node.path);
            } else {
                self.expanded_dirs.insert(node.path.clone());
            }
        }

        // 显示子节点
        if node.is_dir && self.expanded_dirs.contains(&node.path) {
            for child in &node.children {
                if self.show_node(ui, child, depth + 1, current_path, should_navigate) {
                    *should_navigate = true;
                }
            }
        }

        *should_navigate
    }

    pub fn expand_to_path(&mut self, path: &Path) {
        let mut current = path.to_path_buf();
        while let Some(parent) = current.parent() {
            self.expanded_dirs.insert(parent.to_path_buf());
            current = parent.to_path_buf();
        }
    }
}