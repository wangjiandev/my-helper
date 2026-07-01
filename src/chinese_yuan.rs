//! 金额转中文发票大写。
//!
//! 规则：整数部分按万亿分组（连续零合并为单个“零”）后接“圆”；
//! 小数部分第一位为“角”、第二位为“分”；若无分则末尾加“整”。

const DIGITS: [&str; 10] = ["零", "壹", "贰", "叁", "肆", "伍", "陆", "柒", "捌", "玖"];

/// 把金额（元，浮点）转为中文大写，例如 `11051.50 -> "壹万壹仟零伍拾壹圆伍角整"`。
pub fn to_chinese_yuan(amount: f64) -> String {
    // 四舍五入到分，避免浮点误差
    let mut cents = (amount * 100.0).round() as i64;
    let negative = cents < 0;
    if negative {
        cents = -cents;
    }
    let yuan = cents / 100;
    let jiao = (cents / 10) % 10;
    let fen = cents % 10;

    let mut out = String::new();
    if negative {
        out.push_str("负");
    }
    if yuan == 0 {
        out.push_str("零圆");
    } else {
        out.push_str(&integer_to_chinese(yuan as u64));
        out.push('圆');
    }

    if jiao == 0 && fen == 0 {
        out.push_str("整");
    } else {
        // 角为零而分非零时，需补“零”衔接（如 1009.05 -> 零伍分）
        if jiao == 0 {
            out.push_str("零");
        }
        if jiao != 0 {
            out.push_str(DIGITS[jiao as usize]);
            out.push('角');
        }
        if fen != 0 {
            out.push_str(DIGITS[fen as usize]);
            out.push('分');
        } else {
            // 有角无分，末尾加“整”
            out.push_str("整");
        }
    }
    out
}

/// 整数部分转大写（不含“圆”）。零已被正确合并，无尾随零。
fn integer_to_chinese(num: u64) -> String {
    if num == 0 {
        return String::new();
    }
    let s = num.to_string();
    let n = s.len();
    let mut out = String::new();
    let mut last_was_zero = false; // 是否处于零的连续区间
    let mut sec_has_nonzero = false; // 当前 4 位段是否出现过非零

    for (i, ch) in s.chars().enumerate() {
        let d = ch.to_digit(10).unwrap() as usize;
        // 距离个位的位数：0=个/万/亿…，1=拾，2=佰，3=仟
        let pos = n - 1 - i;
        let within = pos % 4;
        let section = pos / 4;

        if d == 0 {
            last_was_zero = true;
        } else {
            if last_was_zero {
                out.push_str("零");
                last_was_zero = false;
            }
            out.push_str(DIGITS[d]);
            match within {
                1 => out.push_str("拾"),
                2 => out.push_str("佰"),
                3 => out.push_str("仟"),
                _ => {}
            }
            sec_has_nonzero = true;
        }

        // 段末（个/万/亿位）输出段单位
        if within == 0 {
            if sec_has_nonzero {
                match section {
                    1 => out.push_str("万"),
                    2 => out.push_str("亿"),
                    3 => out.push_str("万亿"),
                    _ => {}
                }
                sec_has_nonzero = false;
            }
            // 段间可能存在的零标记保持不变，交给下一段首位处理
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::to_chinese_yuan as f;

    #[test]
    fn train_amounts() {
        assert_eq!(f(642.00), "陆佰肆拾贰圆整");
        assert_eq!(f(620.00), "陆佰贰拾圆整");
        assert_eq!(f(10096.00), "壹万零玖拾陆圆整");
    }

    #[test]
    fn taxi_and_hotel() {
        assert_eq!(f(129.50), "壹佰贰拾玖圆伍角整");
        assert_eq!(f(189.00), "壹佰捌拾玖圆整");
        assert_eq!(f(388.50), "叁佰捌拾捌圆伍角整");
        assert_eq!(f(567.00), "伍佰陆拾柒圆整");
    }

    #[test]
    fn total() {
        assert_eq!(f(11051.50), "壹万壹仟零伍拾壹圆伍角整");
    }

    #[test]
    fn with_fen() {
        assert_eq!(f(1009.05), "壹仟零玖圆零伍分");
        assert_eq!(f(0.05), "零圆零伍分");
        assert_eq!(f(0.50), "零圆伍角整");
    }

    #[test]
    fn large_and_zero() {
        assert_eq!(f(0.0), "零圆整");
        assert_eq!(f(100000000.0), "壹亿圆整");
        assert_eq!(f(100010009.0), "壹亿零壹万零玖圆整");
        assert_eq!(f(11051.5), "壹万壹仟零伍拾壹圆伍角整");
    }
}
