use eframe::egui;
use std::path::{Path, PathBuf};
use std::fs;

#[derive(Clone)]
struct TreeNode {
    path: PathBuf,
    name: String,
    is_expanded: bool,
    children: Vec<TreeNode>,
}

pub struct DirectoryTree {
    root_nodes: Vec<TreeNode>,
    show_hidden: bool,
    current_path: PathBuf,
}

impl DirectoryTree {
    pub fn new(root_path: PathBuf) -> Self {
        let mut tree = Self {
            root_nodes: Vec::new(),
            show_hidden: false,
            current_path: root_path.clone(),
        };
        tree.refresh(root_path);
        tree
    }

    pub fn refresh(&mut self, root_path: PathBuf) {
        self.current_path = root_path.clone();
        self.root_nodes = self.build_tree(&root_path, 0);
    }

    // 更新当前路径但保持展开状态
    pub fn update_current_path(&mut self, new_path: &PathBuf) {
        self.current_path = new_path.clone();
        // 不重新构建树，保持展开状态
    }

    fn build_tree(&self, path: &Path, depth: usize) -> Vec<TreeNode> {
        if depth > 3 { // 限制递归深度避免性能问题
            return Vec::new();
        }

        let mut nodes = Vec::new();

        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                let entry_path = entry.path();
                if entry_path.is_dir() {
                    let name = entry_path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("未知文件夹")
                        .to_string();

                    // 跳过隐藏文件夹
                    if !self.show_hidden && name.starts_with('.') {
                        continue;
                    }

                    let children = self.build_tree(&entry_path, depth + 1);

                    nodes.push(TreeNode {
                        path: entry_path,
                        name,
                        is_expanded: depth < 2, // 前两层默认展开
                        children,
                    });
                }
            }
        }

        nodes.sort_by(|a, b| a.name.cmp(&b.name));
        nodes
    }

    pub fn show(&mut self, ui: &mut egui::Ui, new_path: &mut Option<PathBuf>, _selected_file: &mut Option<PathBuf>) {
        // 使用稳定的迭代器避免闪烁
        for node in &mut self.root_nodes {
            if Self::show_node_static(ui, node, new_path, &self.current_path, 0) {
                // 如果导航了，立即返回避免继续处理
                break;
            }
        }
    }

    fn show_node_static(ui: &mut egui::Ui, node: &mut TreeNode, new_path: &mut Option<PathBuf>, current_path: &PathBuf, indent: usize) -> bool {
        let indent_space = indent as f32 * 16.0;
        let is_current_path = current_path == &node.path;
        let mut navigated = false;

        ui.horizontal(|ui| {
            ui.add_space(indent_space);

            // 展开/折叠按钮（小箭头）
            let expand_button = if node.is_expanded { "▼" } else { "▶" };
            if ui.add_sized([20.0, 20.0], egui::Button::new(expand_button)).clicked() {
                node.is_expanded = !node.is_expanded;
            }

            // 文件夹图标
            ui.label("📁");

            // 文件夹名称 - 可点击导航
            let folder_label = ui.selectable_label(
                is_current_path, // 高亮当前选中的路径
                &node.name
            );

            if folder_label.clicked() {
                *new_path = Some(node.path.clone());
                navigated = true;
            }
        });

        if navigated {
            return true;
        }

        if node.is_expanded {
            for child in &mut node.children {
                if Self::show_node_static(ui, child, new_path, current_path, indent + 1) {
                    return true; // 如果子节点导航了，也返回true
                }
            }
        }

        false
    }
}