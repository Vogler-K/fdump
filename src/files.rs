#![allow(dead_code)]
use std::{
    collections::VecDeque,
    fmt,
    fmt::Display,
    fmt::Write,
    fs, mem,
    path::{self, Component, Path, PathBuf},
};

#[derive(Debug)]
pub struct FileCollection {
    pub paths: Vec<PathBuf>,
    pub secret: bool,
}
impl FileCollection {
    pub fn new() -> Self {
        Self {
            paths: Vec::new(),
            secret: false,
        }
    }
    pub fn add_rel(&mut self, relative_path: String) {
        if let Ok(abs_path) = path::absolute(PathBuf::from(relative_path)) {
            self.paths.push(abs_path);
        }
    }
    pub fn add_abs(&mut self, absolute_path: PathBuf) -> &mut PathBuf {
        self.paths.push_mut(absolute_path)
    }
    pub fn squash(&mut self) {
        self.paths.sort_by(|a, b| {
            a.components()
                .count()
                .cmp(&b.components().count())
                .then_with(|| a.is_dir().cmp(&b.is_dir()))
                .then_with(|| a.cmp(b))
        });
        self.paths.dedup();
        self.paths.retain(|p| p.is_dir() || p.is_file());
        let old_paths = mem::take(&mut self.paths);
        for p in old_paths {
            if !self.paths.iter().any(|parent| p.starts_with(parent)) {
                self.paths.push(p);
            }
        }
    }
    pub fn expand(&mut self) {
        let old_paths = mem::take(&mut self.paths);
        let mut paths_deque: VecDeque<PathBuf> = VecDeque::from(old_paths);

        while let Some(p) = paths_deque.pop_front() {
            if p.is_file() {
                self.paths.push(p);
            } else if p.is_dir()
                && let Ok(entries) = fs::read_dir(p)
            {
                for entry in entries.flatten() {
                    paths_deque.push_back(entry.path());
                }
            }
        }
    }
    pub fn get_common(&self) -> String {
        if self.paths.is_empty() {
            return String::new();
        }
        let mut common_components: Vec<Component> = self.paths[0].components().collect();
        common_components.truncate(common_components.len().saturating_sub(1));
        for path in &self.paths[1..] {
            let mut new_len = 0;
            for (c1, c2) in common_components.iter().zip(path.components()) {
                if c1 == &c2 {
                    new_len += 1;
                } else {
                    break;
                }
            }
            common_components.truncate(new_len);
            if common_components.is_empty() {
                break;
            }
        }
        let common_path: PathBuf = common_components.into_iter().collect();
        common_path.to_string_lossy().into_owned()
    }
    pub fn collect_files(&self) -> String {
        let mut collector: String = String::new();
        let base_str = self.get_common();
        let base_path = Path::new(&base_str);
        for p in &self.paths {
            let rel = p.strip_prefix(base_path).unwrap_or(p);
            let _ = writeln!(collector, " ==== {} ==== ", rel.display());
            if let Ok(content) = fs::read_to_string(p) {
                let _ = writeln!(collector, "{content}\n");
            }
        }
        collector
    }
}
impl Display for FileCollection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.collect_files())
    }
}
