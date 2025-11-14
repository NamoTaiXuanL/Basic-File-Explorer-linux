use eframe::egui;
use std::path::{Path, PathBuf};
use std::fs;
use crate::utils;

pub struct DirectoryTree {
    tree_nodes: Vec<TreeNode>,
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

        egui::ScrollArea::vertical().show(ui, |ui| {
            // 先收集需要处理的操作，避免借用问题
            let mut operations = Vec::new();
            self.collect_node_operations(&self.tree_nodes, 0, current_path, &mut operations);

            // 执行操作并收集导航信号
            for (node_ref, depth, path) in operations {
                if self.process_node_interaction(ui, &node_ref, depth, current_path, &mut should_navigate) {
                    should_navigate = true;
                }
            }
        });

        should_navigate
    }

    fn collect_node_operations(&self, nodes: &[TreeNode], depth: usize, _current_path: &Path, operations: &mut Vec<(TreeNode, usize, PathBuf)>) {
        for node in nodes {
            // 克隆节点用于后续处理
            operations.push((node.clone(), depth, node.path.clone()));

            // 递归收集所有子节点（不检查展开状态）
            self.collect_node_operations(&node.children, depth + 1, _current_path, operations);
        }
    }

    fn process_node_interaction(
        &mut self,
        ui: &mut egui::Ui,
        node: &TreeNode,
        depth: usize,
        current_path: &mut PathBuf,
        should_navigate: &mut bool,
    ) -> bool {
        let is_selected = current_path == &node.path;

        // 完全模仿内容框的按钮逻辑
        let button_response = ui.add_sized(
            [ui.available_width(), ui.spacing().interact_size.y * 1.5],
            egui::Button::new({
                let indent = "  ".repeat(depth);

                let icon = if node.is_dir {
                    "📁"  // 目录图标固定为文件夹
                } else {
                    "📄"  // 文件图标
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

        let mut nav_result = false;

        // 完全模仿内容框的点击处理
        if button_response.clicked() && node.is_dir {
            *current_path = node.path.clone();
            *should_navigate = true;
            nav_result = true;
        }

        nav_result
    }
}