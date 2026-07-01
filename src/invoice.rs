//! 发票类型与统计结构。

use std::path::PathBuf;

/// 发票类别。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Kind {
    /// 铁路电子客票
    Train,
    /// 打车 / 网约车
    Taxi,
    /// 住宿
    Hotel,
    /// 其它普通发票
    Other,
}

impl Kind {
    /// 中文标签（用于统计表）。
    pub fn label(self) -> &'static str {
        match self {
            Kind::Train => "火车票",
            Kind::Taxi => "打车费",
            Kind::Hotel => "住宿费",
            Kind::Other => "其它",
        }
    }

    /// 是否按 4 行 2 列（8 张/页）排版。
    pub fn is_train(self) -> bool {
        matches!(self, Kind::Train)
    }
}

/// 一张发票的解析结果。
#[derive(Debug, Clone)]
pub struct Invoice {
    pub path: PathBuf,
    pub kind: Kind,
    /// 金额（元）。解析失败时为 0。
    pub amount: f64,
}

/// 分类统计。
#[derive(Debug, Default, Clone)]
pub struct Stats {
    pub train: Category,
    pub taxi: Category,
    pub hotel: Category,
    pub other: Category,
}

#[derive(Debug, Default, Clone)]
pub struct Category {
    pub count: usize,
    pub total: f64,
}

impl Stats {
    pub fn add(&mut self, kind: Kind, amount: f64) {
        let cat = match kind {
            Kind::Train => &mut self.train,
            Kind::Taxi => &mut self.taxi,
            Kind::Hotel => &mut self.hotel,
            Kind::Other => &mut self.other,
        };
        cat.count += 1;
        cat.total += amount;
    }

    pub fn grand_total(&self) -> f64 {
        self.train.total + self.taxi.total + self.hotel.total + self.other.total
    }

    pub fn grand_count(&self) -> usize {
        self.train.count + self.taxi.count + self.hotel.count + self.other.count
    }

    /// 按统计输出顺序返回分类。
    pub fn rows(&self) -> [(Kind, &Category); 4] {
        [
            (Kind::Train, &self.train),
            (Kind::Taxi, &self.taxi),
            (Kind::Hotel, &self.hotel),
            (Kind::Other, &self.other),
        ]
    }
}
