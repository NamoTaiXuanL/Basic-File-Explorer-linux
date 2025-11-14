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
}

impl DirectoryTree {
    pub fn new(root_path: PathBuf) -> Self {
        let mut tree = Self {
            root_nodes: Vec::new(),
            show_hidden: false,
        };
        tree.refresh(root_path);
        tree
    }

    pub fn refresh(&mut self, root_path: PathBuf) {
        self.root_nodes = self.build_tree(&root_path, 0);
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
        let nodes: Vec<_> = self.root_nodes.iter_mut().collect();
        for node in nodes {
            if Self::show_node_static(ui, node, new_path, 0) {
                return;
            }
        }
    }

    fn show_node_static(ui: &mut egui::Ui, node: &mut TreeNode, new_path: &mut Option<PathBuf>, indent: usize) -> bool {
        let indent_space = indent as f32 * 16.0;

        let mut should_return = false;
        ui.horizontal(|ui| {
            ui.add_space(indent_space);

            // 展开/折叠按钮
            let expanded_text = if node.is_expanded { "📂" } else { "📁" };
            if ui.button(expanded_text).clicked() {
                node.is_expanded = !node.is_expanded;
            }

            // 文件夹名称
            if ui.selectable_label(
                false, // TODO: 跟踪当前选中的路径
                &node.name
            ).clicked() {
                *new_path = Some(node.path.clone());
                should_return = true;
            }
        });

        if should_return {
            return true;
        }

        if node.is_expanded {
            for child in &mut node.children {
                if Self::show_node_static(ui, child, new_path, indent + 1) {
                    return true;
                }
            }
        }

        false
    }
}