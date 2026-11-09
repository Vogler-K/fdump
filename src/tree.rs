#![allow(dead_code)]
use crate::files::FileCollection;

use std::fmt::{self, Write};
use std::path::{self, Path};

#[derive(Debug)]
pub struct RootFS {
    root: TreeItem,
    secret: bool,
}
impl RootFS {
    pub fn new() -> Self {
        RootFS {
            root: TreeItem::new("", true),
            secret: false,
        }
    }
    pub fn add_to_tree(&mut self, path: &Path) {
        let names: Vec<String> = path
            .components()
            .filter_map(|t| match t {
                path::Component::Normal(s) => s.to_str().map(str::to_string),
                _ => None,
            })
            .collect();
        let mut last_child = &mut self.root;
        for name in names {
            last_child = last_child.create_child(&name, path.is_dir());
        }
    }
    pub fn adds_to_tree(&mut self, paths: &[path::PathBuf]) {
        for path in paths {
            self.add_to_tree(path);
        }
    }
}
impl Default for RootFS {
    fn default() -> Self {
        RootFS::new()
    }
}
impl From<&FileCollection> for RootFS {
    fn from(file_collection: &FileCollection) -> Self {
        let mut root_fs: RootFS = RootFS::new();
        root_fs.adds_to_tree(&file_collection.paths);
        root_fs.secret = file_collection.secret;
        root_fs
    }
}
impl fmt::Display for RootFS {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.root.print_with_children(
            0,
            "",
            self.root.children.as_ref().is_some_and(|c| c.len() == 1),
            &mut TreeItemPrintHelper {
                f,
                omit_allowed: true,
                omitted: String::new(),
                secret: self.secret,
            },
        )?;
        Ok(())
    }
}
#[derive(Debug)]
struct TreeItem {
    name: String,
    children: Option<Vec<TreeItem>>,
}
struct TreeItemPrintHelper<'a, 'b> {
    f: &'a mut fmt::Formatter<'b>,
    omit_allowed: bool,
    omitted: String,
    secret: bool,
}
impl TreeItem {
    pub fn new(name: &str, folder: bool) -> Self {
        if folder {
            Self {
                name: name.to_string(),
                children: Some(Vec::new()),
            }
        } else {
            Self {
                name: name.to_string(),
                children: None,
            }
        }
    }
    pub fn create_child(&mut self, name: &str, folder: bool) -> &mut TreeItem {
        let self_children = self.children.get_or_insert_with(Vec::new);
        if let Some(pos) = self_children.iter().position(|child| child.name == name) {
            return &mut self_children[pos];
        }
        self_children.push_mut(TreeItem::new(name, folder))
    }
    pub fn get_child(&mut self, name: &str) -> Option<&mut TreeItem> {
        if let Some(self_children) = &mut self.children {
            return self_children.iter_mut().find(|child| child.name == name);
        }
        None
    }
    const SUBCHILD: &str = "├── ";
    const LASTSUBCHILD: &str = "└── ";
    const PROCEEDING: &str = "│   ";
    const RECEDING: &str = "    ";
    const NONE: &str = "";
    pub fn print_with_children(
        &self,
        depth: usize,
        parent_prefix: &str,
        last_child: bool,
        helper: &mut TreeItemPrintHelper,
    ) -> fmt::Result {
        let prefix = match (depth != 0, !last_child) {
            (true, true) => TreeItem::SUBCHILD,
            (true, false) => TreeItem::LASTSUBCHILD,
            (false, _) => TreeItem::NONE,
        };
        if let Some(self_children) = &self.children {
            if let Some(only) = self_children.first()
                && (helper.omit_allowed && self_children.len() == 1)
            {
                if !helper.secret {
                    write!(helper.omitted, "{self}/")?;
                }
                only.print_with_children(depth, parent_prefix, last_child, helper)?;
            } else {
                writeln!(helper.f, "{}{parent_prefix}{prefix}{self}/", helper.omitted)?;
                helper.omit_allowed = false;
                helper.omitted.clear();

                let next_parent_prefix: String = match (depth == 0, !last_child) {
                    (true, _) => TreeItem::NONE.to_owned(),
                    (false, true) => format!("{}{}", parent_prefix, TreeItem::PROCEEDING),
                    (false, false) => format!("{}{}", parent_prefix, TreeItem::RECEDING),
                };

                let mut iter = self_children.iter().peekable();
                while let Some(child) = iter.next() {
                    let last_child: bool = iter.peek().is_none();
                    child.print_with_children(
                        depth + 1,
                        &next_parent_prefix,
                        last_child,
                        helper,
                    )?;
                }
            }
        } else {
            writeln!(helper.f, "{}{parent_prefix}{prefix}{self}", helper.omitted)?;
        }
        Ok(())
    }
}
impl fmt::Display for TreeItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}
