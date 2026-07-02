//! 发票打印助手入口。

mod chinese_yuan;
mod cli;
mod composer;
mod extractor;
mod invoice;
mod report;
mod scanner;
mod utils;

use std::fs;
use std::path::PathBuf;

use anyhow::{anyhow, Context, Result};
use clap::Parser;
use pdfium_render::prelude::*;

use cli::Args;
use invoice::Stats;

fn main() -> Result<()> {
    let args = Args::parse();

    let pdfium = load_pdfium()?;

    println!("扫描目录: {}", args.dir.display());
    if !args.dir.exists() {
        fs::create_dir_all(&args.dir)
            .with_context(|| format!("无法创建目录: {}", args.dir.display()))?;
        println!(
            "已创建目录 {}，请将发票 PDF 放入后重新运行。",
            args.dir.display()
        );
        return Ok(());
    }
    let files = scanner::scan(&args.dir)?;
    if files.is_empty() {
        println!(
            "目录 {} 中没有 PDF 发票，请将发票放入后重新运行。",
            args.dir.display()
        );
        return Ok(());
    }
    println!("发现 {} 个 PDF 文件，开始解析…", files.len());

    let mut invoices = Vec::with_capacity(files.len());
    let mut stats = Stats::default();
    for f in &files {
        match extractor::extract(&pdfium, f) {
            Ok(inv) => {
                println!(
                    "  {} -> {:?} ¥{:.2}",
                    f.file_name().unwrap_or_default().to_string_lossy(),
                    inv.kind,
                    inv.amount
                );
                stats.add(inv.kind, inv.amount);
                invoices.push(inv);
            }
            Err(e) => {
                eprintln!("  [跳过] {}: {e:#}", f.display());
            }
        }
    }

    if invoices.is_empty() {
        anyhow::bail!("未解析到任何有效发票，已退出。");
    }

    if let Some(parent) = args.out.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)
                .with_context(|| format!("无法创建输出目录: {}", parent.display()))?;
        }
    }
    let pages = composer::compose(&pdfium, &invoices, &args.out)
        .with_context(|| format!("合成 PDF 失败"))?;
    println!("\n已生成: {}（{} 页）", args.out.display(), pages);

    report::print(&stats);
    Ok(())
}

/// 加载内置 libpdfium，失败则回退系统库。
/// 优先在可执行文件同级/其 lib 子目录查找（支持双击启动），再回退工作目录与系统库。
fn load_pdfium() -> Result<Pdfium> {
    let names = [
        "libpdfium.dylib",
        "libpdfium.so",
        "libpdfium.dll",
        "pdfium.dll",
        "pdfium.so",
    ];

    let mut dirs: Vec<PathBuf> = Vec::new();
    if let Ok(exe) = std::env::current_exe() {
        if let Some(exe_dir) = exe.parent() {
            dirs.push(exe_dir.to_path_buf());
            dirs.push(exe_dir.join("lib"));
        }
    }
    dirs.push(PathBuf::from("./lib"));
    dirs.push(PathBuf::from("."));

    for dir in &dirs {
        for name in &names {
            let p = dir.join(name);
            if p.exists() {
                return Pdfium::bind_to_library(&p)
                    .map(Pdfium::new)
                    .map_err(|e| anyhow!("加载 {} 失败: {e}", p.display()));
            }
        }
    }

    Pdfium::bind_to_system_library()
        .map(Pdfium::new)
        .map_err(|e| {
            anyhow!("未找到 libpdfium。请运行 scripts/fetch-pdfium.sh 下载到 lib/ 目录。\n{e}")
        })
}
