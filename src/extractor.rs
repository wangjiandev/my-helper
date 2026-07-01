//! 从发票 PDF 抽取文字、分类、解析金额。

use std::path::Path;

use anyhow::{Context, Result};
use pdfium_render::prelude::*;
use regex::Regex;
use std::sync::LazyLock;

use crate::invoice::{Invoice, Kind};

static TRAIN_PRICE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"票价[：:]\s*[￥¥]\s*(\d+(?:\.\d+)?)").unwrap());

static ANY_AMOUNT: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"[￥¥]\s*(\d+\.\d+)").unwrap());

/// 解析一张发票。
pub fn extract(pdfium: &Pdfium, path: &Path) -> Result<Invoice> {
    let text = page_text(pdfium, path)?;

    let kind = classify(&text);
    let amount = parse_amount(kind, &text).unwrap_or(0.0);

    Ok(Invoice {
        path: path.to_path_buf(),
        kind,
        amount,
    })
}

/// 抽取首页文字：优先 pdfium；若结果为空（部分 PDF 缺 ToUnicode 映射），
/// 回退到 poppler 的 `pdftotext`。
fn page_text(pdfium: &Pdfium, path: &Path) -> Result<String> {
    if let Ok(doc) = pdfium.load_pdf_from_file(path, None) {
        if doc.pages().len() > 0 {
            if let Ok(page) = doc.pages().get(0) {
                if let Ok(text) = page.text() {
                    let t = text.all();
                    if t.trim().chars().count() > 8 {
                        return Ok(t);
                    }
                }
            }
        }
    }

    let out = std::process::Command::new("pdftotext")
        .arg("-layout")
        .arg(path)
        .arg("-")
        .output()
        .context("调用 pdftotext 失败，请确保已安装 poppler")?;
    if !out.status.success() {
        anyhow::bail!(
            "pdftotext 解析失败: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

/// 按优先级分类。
pub fn classify(text: &str) -> Kind {
    // 折叠所有空白（PDF 抽取常把词语断行，如“客运服\n务费”）
    let compact: String = text.chars().filter(|c| !c.is_whitespace()).collect();
    if compact.contains("铁路电子客票") {
        Kind::Train
    } else if compact.contains("住宿") || compact.contains("酒店") {
        Kind::Hotel
    } else if compact.contains("客运服务")
        || compact.contains("交通运输服务")
        || compact.contains("旅客运输")
        || compact.contains("滴滴")
    {
        Kind::Taxi
    } else {
        Kind::Other
    }
}

/// 解析金额（元）。火车票取“票价”；普通发票取所有 ¥金额中的最大值
/// （价税合计为含税总额，必然最大）。
pub fn parse_amount(kind: Kind, text: &str) -> Option<f64> {
    let compact: String = text.chars().filter(|c| !c.is_whitespace()).collect();
    if kind.is_train() {
        return TRAIN_PRICE
            .captures(&compact)
            .and_then(|c| c.get(1)?.as_str().parse::<f64>().ok());
    }
    ANY_AMOUNT
        .captures_iter(&compact)
        .filter_map(|c| c.get(1)?.as_str().parse::<f64>().ok())
        .fold(None::<f64>, |acc, v| Some(acc.map_or(v, |m| m.max(v))))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_samples() {
        assert_eq!(classify("电子发票（铁路电子客票）票价"), Kind::Train);
        assert_eq!(classify("电子发票（普通发票）客运服务费 滴滴"), Kind::Taxi);
        assert_eq!(classify("电子发票 住宿服务 长沙酒店"), Kind::Hotel);
        assert_eq!(classify("随便一张发票"), Kind::Other);
    }

    #[test]
    fn parse_train_amount() {
        assert_eq!(parse_amount(Kind::Train, "票价:￥642.00"), Some(642.0));
        assert_eq!(parse_amount(Kind::Train, "票价：￥620.00 元"), Some(620.0));
    }

    #[test]
    fn parse_jiaoshui_with_wrapped_number() {
        // 模拟住宿发票跨行断裂
        let t = "价税合计（大写）壹佰圆整（小写）¥189.\n00";
        assert_eq!(parse_amount(Kind::Hotel, t), Some(189.0));
    }

    #[test]
    fn parse_taxi_amount() {
        let t = "（ 小 写 ） ¥129.50";
        assert_eq!(parse_amount(Kind::Taxi, t), Some(129.5));
    }
}
