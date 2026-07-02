# 发票打印助手（invoice-printer）

一个命令行工具：把一个目录下的电子发票（火车票、打车票、住宿票等 PDF）排版成一份 A4 竖版 PDF，方便打印裁切，并在控制台输出分类金额统计（含中文大写）。

## 下载即用（免编译）

不想装 Rust 也能用：到本仓库的 **Releases** 页面下载对应平台压缩包，解压到你的发票目录，双击启动器即可。

| 平台 | 下载 | 启动文件 |
|---|---|---|
| macOS（Apple Silicon + Intel 通用） | `invoice-printer-mac.zip` | 双击 `run.command` |
| Windows（x64） | `invoice-printer-win.zip` | 双击 `run.bat` |

行为：读取启动器所在文件夹下 `source/`（含子目录）的所有 PDF，生成 `out/output.pdf`，并在终端打印分类金额统计表。

**首次运行放行（仅需一次，因未做开发者签名）：**

- **macOS**：首次双击 `run.command` 若提示「无法打开/无法验证开发者」，请 `右键 → 打开 → 打开`；或在终端对解压后的文件夹执行 `xattr -dr com.apple.quarantine invoice-printer-mac`（启动器内部也会自动清除隔离属性）。
- **Windows**：若 SmartScreen 提示「已保护你的电脑」，点 `更多信息 → 仍要运行`。

> 包内已自带对应平台的 `libpdfium` 动态库，无需再运行 `fetch-pdfium.sh`。需要自行从源码构建时，继续看下方「快速开始」。

## 功能

- 递归扫描发票目录，读取所有 PDF。
- **火车票**：A4 竖版，每页 8 张（4 行 × 2 列），自上而下排列。
- **普通发票**（打车 / 住宿 / 其它）：A4 竖版，每页 2 张（2 行 × 1 列）。
- 每页绘制一条**水平中心虚线**，沿虚线裁切即得上下两片 A5，对称分布、方便裁剪。
- 自动分类：火车票 / 打车费 / 住宿费 / 其它。
- 自动提取金额并汇总：控制台输出分类统计表，同时给出**中文大写**。

## 环境要求

- Rust 工具链（推荐 `cargo` 1.70+，本项目在 1.96 验证通过）。
- `libpdfium` 动态库（见下方安装步骤，项目内置下载脚本）。
- 可选：`poppler`（提供 `pdftotext`，作为部分缺 ToUnicode 映射发票的文字抽取回退）。

## 快速开始

### 1. 安装依赖

```bash
# 下载 libpdfium 动态库到 lib/ 目录（自动识别平台）
bash scripts/fetch-pdfium.sh
```

脚本支持的系统：macOS（arm64/x64）、Linux（x64/arm64）、Windows（x64）。它会把动态库放到 `lib/`。

### 2. 编译并运行

```bash
cargo run --release -- --dir ./test --out ./out/output.pdf
```

运行后会：

1. 扫描 `./test` 下所有 PDF；
2. 逐张解析、分类、提取金额并打印到控制台；
3. 生成 `./out/output.pdf`；
4. 在控制台输出金额统计表。

示例输出：

```
扫描目录: ./test
发现 22 个 PDF 文件，开始解析…
  f00001.pdf -> Train ¥642.00
  ...
已生成: ./out/output.pdf（5 页）

┌────────┬──────┬────────────┬──────────────────────────┐
│ 类别   ┆ 张数 ┆ 金额(小写) ┆ 金额(大写)               │
╞════════╪══════╪════════════╪══════════════════════════╡
│ 火车票 ┆ 16   ┆ ¥10,096.00 ┆ 壹万零玖拾陆圆整         │
│ 打车费 ┆ 3    ┆ ¥388.50    ┆ 叁佰捌拾捌圆伍角整       │
│ 住宿费 ┆ 3    ┆ ¥567.00    ┆ 伍佰陆拾柒圆整           │
│ 合计   ┆ 22   ┆ ¥11,051.50 ┆ 壹万壹仟零伍拾壹圆伍角整 │
└────────┴──────┴────────────┴──────────────────────────┘
```

## 命令行参数

```text
invoice-printer --dir <DIR> --out <OUT>

选项：
  -d, --dir <DIR>    发票目录（递归扫描）  [默认: ./source]
  -o, --out <OUT>    输出 PDF 路径          [默认: ./out/output.pdf]
  -h, --help         帮助
  -V, --version      版本
```

示例：

```bash
# 指定自定义目录和输出
cargo run --release -- --dir ~/Documents/invoices --out ~/Desktop/reimburse.pdf

# 仅处理某个子目录
cargo run --release -- -d ./test/trains
```

## 输出 PDF 结构

所有页面均为 A4 竖版（595.28 × 841.89 pt），每页含一条水平中心虚线。

| 页面 | 内容 | 排版 |
|---|---|---|
| 火车票页 | 8 张 / 页 | 4 行 × 2 列，每片 A5 含 4 张 |
| 普通发票页 | 2 张 / 页 | 2 行 × 1 列，每片 A5 含 1 张 |

打印后沿中心虚线裁切，每张 A4 即得 2 片 A5。

> 金额统计仅在控制台输出（见上文表格），不写入 PDF。

## 分类与金额提取规则

**分类（按优先级）：**

1. 含 `铁路电子客票` → 火车票
2. 含 `住宿` 或销方 `酒店` → 住宿费
3. 含 `客运服务费` / `滴滴` / `交通运输服务` → 打车费
4. 其余 → 其它

**金额提取：**

- 火车票：匹配 `票价：￥xxx`。
- 普通发票：折叠所有空白（含换行）后，取所有 `￥金额` 中的最大值（即价税合计）。

**中文大写：** 整数部分按万亿分组（连续零合并）后接“圆”；角 / 分分别输出；无分则末尾加“整”。

## 测试

```bash
cargo test
```

包含 `chinese_yuan`、`extractor`、`scanner`、`utils` 的单元测试。

## 容错处理

- 缺 `libpdfium`：提示运行 `scripts/fetch-pdfium.sh`。
- 金额解析失败：告警并跳过该张统计（仍会排版）。
- 非 PDF / 损坏文件：跳过并告警。
- 某类发票为空：正常处理，金额记 0。
- 目录不存在：自动创建 `source/` 后提示放入发票并退出（输出目录 `out/` 也会自动创建）。

## 目录结构

```
invoice-printer/
├── Cargo.toml
├── scripts/fetch-pdfium.sh   # 下载 libpdfium → lib/
├── lib/libpdfium.dylib       # 运行时按路径加载（自行下载）
├── test/                      # 测试发票目录（22 张示例 PDF）
├── examples/dump_text.rs      # 调试用：导出发票文字
└── src/
    ├── main.rs          # 编排入口
    ├── cli.rs           # 命令行参数
    ├── scanner.rs       # 递归扫描 *.pdf
    ├── invoice.rs       # 类型与统计结构
    ├── extractor.rs     # 文字抽取 → 分类 → 金额
    ├── chinese_yuan.rs  # 金额转中文大写
    ├── composer.rs      # PDF 合成 / 排版 / 中心虚线
    ├── report.rs        # 控制台统计表
    └── utils.rs         # 金额格式化等
```

## 常见问题

- **加载 libpdfium 失败**：重新运行 `bash scripts/fetch-pdfium.sh`；macOS 若因隔离属性报错，脚本已自动 `xattr -dr`，仍失败可手动执行。
- **金额为 0**：该发票文字抽取异常或格式不符正则，控制台会标注；可安装 `poppler` 以启用 `pdftotext` 回退。
