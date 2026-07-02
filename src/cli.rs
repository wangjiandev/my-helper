//! 命令行参数。

use std::path::PathBuf;

use clap::Parser;

/// 发票打印助手：把一个目录下的电子发票排版成 A4 PDF 并汇总金额。
#[derive(Parser, Debug)]
#[command(name = "invoice-printer", version, about)]
pub struct Args {
    /// 需要报销的发票目录（递归扫描），默认为当前项目的 `source` 目录。
    #[arg(long, short = 'd', default_value = "./source")]
    pub dir: PathBuf,

    /// 输出 PDF 路径，默认输出到当前项目的 `out` 目录。
    #[arg(long, short = 'o', default_value = "./out/output.pdf")]
    pub out: PathBuf,
}
