//! 控制台金额统计表（含中文大写）。

use comfy_table::{presets::UTF8_FULL, Cell, Color, ContentArrangement, Table};

use crate::chinese_yuan::to_chinese_yuan;
use crate::invoice::Stats;
use crate::utils::fmt_amount;

/// 打印分类金额统计到控制台。
pub fn print(stats: &Stats) {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        // 关闭自动压缩，确保大写金额完整显示
        .set_content_arrangement(ContentArrangement::Disabled)
        .set_header(vec![
            Cell::new("类别").fg(Color::Yellow),
            Cell::new("张数").fg(Color::Yellow),
            Cell::new("金额(小写)").fg(Color::Yellow),
            Cell::new("金额(大写)").fg(Color::Yellow),
        ]);

    for (kind, cat) in stats.rows() {
        table.add_row(vec![
            Cell::new(kind.label()),
            Cell::new(cat.count),
            Cell::new(format!("¥{}", fmt_amount(cat.total))),
            Cell::new(to_chinese_yuan(cat.total)),
        ]);
    }

    table.add_row(vec![
        Cell::new("合计").fg(Color::Cyan),
        Cell::new(stats.grand_count()).fg(Color::Cyan),
        Cell::new(format!("¥{}", fmt_amount(stats.grand_total()))).fg(Color::Cyan),
        Cell::new(to_chinese_yuan(stats.grand_total())).fg(Color::Cyan),
    ]);

    println!("\n{table}");
    println!(
        "\n总计：{} 张，金额 ¥{} （{}）",
        stats.grand_count(),
        fmt_amount(stats.grand_total()),
        to_chinese_yuan(stats.grand_total())
    );
}
