//! 递归扫描目录，返回按文件名排序的 PDF 列表。

use std::path::{Path, PathBuf};

use anyhow::Result;
use walkdir::WalkDir;

pub fn scan(dir: &Path) -> Result<Vec<PathBuf>> {
    if !dir.exists() {
        anyhow::bail!("发票目录不存在: {}", dir.display());
    }
    let mut files: Vec<PathBuf> = WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .map(|e| e.into_path())
        .filter(|p| p.is_file() && has_pdf_ext(p))
        .collect();
    files.sort();
    Ok(files)
}

fn has_pdf_ext(p: &Path) -> bool {
    p.extension()
        .and_then(|e| e.to_str())
        .map(|e| e.eq_ignore_ascii_case("pdf"))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scan_test_dir() {
        let files = scan(Path::new("./test")).unwrap();
        assert_eq!(files.len(), 22, "test 目录应有 22 个 PDF");
        assert!(files.iter().all(|p| has_pdf_ext(p)));
    }
}
