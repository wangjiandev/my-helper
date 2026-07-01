//! 通用工具。

/// 金额千分位格式化，保留两位小数。例如 `11051.5 -> "11,051.50"`。
pub fn fmt_amount(v: f64) -> String {
    let neg = v < 0.0;
    let abs = v.abs();
    let cents = (abs * 100.0).round() as i64;
    let int_part = (cents / 100).to_string();
    let dec = format!("{:02}", cents % 100);
    let bytes = int_part.as_bytes();
    let mut grouped = String::new();
    for (i, b) in bytes.iter().enumerate() {
        if i > 0 && (bytes.len() - i) % 3 == 0 {
            grouped.push(',');
        }
        grouped.push(*b as char);
    }
    if neg {
        format!("-{}.{dec}", grouped)
    } else {
        format!("{}.{dec}", grouped)
    }
}

#[cfg(test)]
mod tests {
    use super::fmt_amount;

    #[test]
    fn grouping() {
        assert_eq!(fmt_amount(11051.5), "11,051.50");
        assert_eq!(fmt_amount(10096.0), "10,096.00");
        assert_eq!(fmt_amount(388.5), "388.50");
        assert_eq!(fmt_amount(0.0), "0.00");
        assert_eq!(fmt_amount(1000000.0), "1,000,000.00");
    }
}
