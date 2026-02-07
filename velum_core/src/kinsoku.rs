//! 避头尾规则 (Kinsoku Shori) 实现
//!
//! 中文和日文排版中，某些标点符号不能出现在行首或行尾。
//! 本模块提供字符检查和行调整功能。

use crate::line_breaking::Line;

/// 禁止出现在行首的字符集
const NO_START_CHARS: &[char] = &[
    // CJK 标点 - 行首禁止
    '，', '。', '．', '、', '：', '；', '？', '！',
    '）', '］', '｝', '〉', '」', '』', '】', '〙',
    '〗', '〟', '"', '\'', '»', '›',
];

/// 禁止出现在行尾的字符集
const NO_END_CHARS: &[char] = &[
    // CJK 标点 - 行尾禁止
    '（', '〔', '【',
];

/// 避头尾调整类型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdjustmentType {
    /// 挤压 - 压缩字符间距
    Shrink,
    /// 悬挂 - 将标点移到行边界外
    Hanging,
    /// 换行 - 将字符移到下一行
    Wrap,
}

/// 避头尾调整信息
#[derive(Debug, Clone, PartialEq)]
pub struct KinsokuAdjustment {
    /// 行号 (0-based)
    pub line_index: usize,
    /// 调整类型
    pub adjustment_type: AdjustmentType,
    /// 行内起始偏移 (字节偏移)
    pub start_offset: usize,
    /// 行内结束偏移 (字节偏移)
    pub end_offset: usize,
    /// 需要的额外空间 (用于挤压/悬挂计算)
    pub extra_space_needed: f64,
    /// 需要移动到下一行的字符数
    pub chars_to_move: usize,
    /// 违规的字符
    pub problem_char: char,
}

impl KinsokuAdjustment {
    /// 创建新的调整
    pub fn new(
        line_index: usize,
        adjustment_type: AdjustmentType,
        start_offset: usize,
        end_offset: usize,
        extra_space_needed: f64,
        chars_to_move: usize,
        problem_char: char,
    ) -> Self {
        Self {
            line_index,
            adjustment_type,
            start_offset,
            end_offset,
            extra_space_needed,
            chars_to_move,
            problem_char,
        }
    }
}

/// 检查字符是否禁止出现在行首
///
/// # Examples
///
/// ```
/// use velum_core::kinsoku::is_no_start_char;
///
/// assert!(is_no_start_char('，'));
/// assert!(!is_no_start_char('中'));
/// ```
pub fn is_no_start_char(ch: char) -> bool {
    NO_START_CHARS.contains(&ch)
}

/// 检查字符是否禁止出现在行尾
///
/// # Examples
///
/// ```
/// use velum_core::kinsoku::is_no_end_char;
///
/// assert!(is_no_end_char('（'));
/// assert!(!is_no_end_char('中'));
/// ```
pub fn is_no_end_char(ch: char) -> bool {
    NO_END_CHARS.contains(&ch)
}

/// 检查字符是否为CJK字符
pub fn is_cjk_char(ch: char) -> bool {
    let cp = ch as u32;
    // CJK统一汉字基本区
    (0x4E00..=0x9FFF).contains(&cp) ||
    // CJK统一汉字扩展区A
    (0x3400..=0x4DBF).contains(&cp) ||
    // CJK统一汉字扩展区B-F
    (0x20000..=0x2A6DF).contains(&cp) ||
    // 日文假名
    (0x3040..=0x309F).contains(&cp) ||
    (0x30A0..=0x30FF).contains(&cp) ||
    // 朝鲜文
    (0xAC00..=0xD7AF).contains(&cp) ||
    (0x1100..=0x11FF).contains(&cp) ||
    // 全角标点符号 (FF00-FFEF)
    (0xFF00..=0xFFEF).contains(&cp) ||
    // CJK标点符号 (3000-303F)
    (0x3000..=0x303F).contains(&cp) ||
    // 常见CJK标点
    matches!(ch,
        '，' | '。' | '、' | '：' | '；' | '？' | '！' |
        '（' | '）' | '【' | '】' | '〔' | '〕' | '《' | '》' |
        '"' | '"' | '\'' | '\'' | '～' | '…' | '—'
    )
}

/// 检查字符是否为可挤压的CJK标点
pub fn is_shrinkable_punctuation(ch: char) -> bool {
    matches!(ch,
        '，' | '。' | '、' | '：' | '；' |
        '？' | '！' | '（' | '）' | '【' | '】'
    )
}

/// 获取字符串中指定偏移处的字符
fn char_at(text: &str, byte_offset: usize) -> Option<char> {
    text[byte_offset..].chars().next()
}

/// 获取字符串中指定偏移处的字符（从末尾）
fn char_at_from_end(text: &str, byte_offset: usize) -> Option<char> {
    text[..byte_offset].chars().last()
}

/// 获取字符宽度 (简化版本)
fn get_char_width(ch: char, default_width: f64) -> f64 {
    if is_cjk_char(ch) || is_shrinkable_punctuation(ch) {
        default_width
    } else if ch.is_ascii() {
        default_width * 0.5
    } else {
        default_width
    }
}

/// 处理文本，应用避头尾规则
///
/// # Arguments
///
/// * `lines` - 行数组
/// * `texts` - 对应的文本数组 (每行对应的完整文本片段)
/// * `line_widths` - 每行的实际宽度
/// * `max_width` - 最大可用宽度
///
/// # Returns
/// 需要调整的行信息列表
pub fn process_for_kinsoku(
    lines: &[Line],
    texts: &[&str],
    line_widths: &[f64],
    max_width: f64,
) -> Vec<KinsokuAdjustment> {
    let mut adjustments = Vec::new();

    for (line_index, (line, line_text)) in lines.iter().zip(texts.iter()).enumerate() {
        if line.is_empty() || line_text.is_empty() {
            continue;
        }

        let line_width = line_widths.get(line_index).copied().unwrap_or(max_width);
        let overflow = line_width - max_width;

        // 检查行首字符
        if let Some(first_char) = line_text.chars().next() {
            if is_no_start_char(first_char) {
                let char_width = get_char_width(first_char, 1.0);
                let hanging_space = max_width - line_width + char_width;

                if hanging_space > 0.0 {
                    // 尝试悬挂
                    adjustments.push(KinsokuAdjustment::new(
                        line_index,
                        AdjustmentType::Hanging,
                        line.start,
                        line.start + first_char.len_utf8(),
                        char_width,
                        1,
                        first_char,
                    ));
                } else {
                    // 需要换行
                    adjustments.push(KinsokuAdjustment::new(
                        line_index,
                        AdjustmentType::Wrap,
                        line.start,
                        line.start + first_char.len_utf8(),
                        0.0,
                        1,
                        first_char,
                    ));
                }
            }
        }

        // 检查行尾字符
        if let Some(last_char) = line_text.chars().last() {
            if is_no_end_char(last_char) {
                let char_width = get_char_width(last_char, 1.0);
                let byte_len = last_char.len_utf8();

                if overflow > 0.0 && overflow >= char_width {
                    // 尝试挤压
                    adjustments.push(KinsokuAdjustment::new(
                        line_index,
                        AdjustmentType::Shrink,
                        line.end - byte_len,
                        line.end,
                        char_width,
                        0,
                        last_char,
                    ));
                } else {
                    // 需要换行
                    adjustments.push(KinsokuAdjustment::new(
                        line_index,
                        AdjustmentType::Wrap,
                        line.end - byte_len,
                        line.end,
                        0.0,
                        1,
                        last_char,
                    ));
                }
            }
        }

        // 检查行中间是否有违规字符对 (禁止行尾 + 禁止行首)
        let chars: Vec<char> = line_text.chars().collect();
        for i in 0..chars.len().saturating_sub(1) {
            let curr = chars[i];
            let next = chars[i + 1];

            if is_no_end_char(curr) && is_no_start_char(next) {
                let curr_width = get_char_width(curr, 1.0);
                let next_width = get_char_width(next, 1.0);
                let total_width = curr_width + next_width;

                if overflow > 0.0 {
                    // 挤压处理
                    let start_byte = line.start + line_text[..line_text.char_indices().nth(i).unwrap_or((0, '\0')).0 + curr.len_utf8()].len();
                    let end_byte = line.start + line_text[..line_text.char_indices().nth(i + 1).unwrap_or((0, '\0')).0 + next.len_utf8()].len();

                    adjustments.push(KinsokuAdjustment::new(
                        line_index,
                        AdjustmentType::Shrink,
                        start_byte,
                        end_byte,
                        total_width,
                        0,
                        curr,
                    ));
                } else {
                    // 换行处理
                    adjustments.push(KinsokuAdjustment::new(
                        line_index,
                        AdjustmentType::Wrap,
                        line.start + line_text[..line_text.char_indices().nth(i).unwrap_or((0, '\0')).0 + curr.len_utf8()].len(),
                        line.start + line_text[..line_text.char_indices().nth(i + 1).unwrap_or((0, '\0')).0 + next.len_utf8()].len(),
                        0.0,
                        2,
                        curr,
                    ));
                }
                break; // 只处理第一个违规对
            }
        }
    }

    adjustments
}

/// 计算挤压因子
///
/// 根据需要的额外空间计算字符间距压缩比例
pub fn calculate_shrink_factor(space_needed: f64, available_space: f64) -> f64 {
    if available_space <= 0.0 {
        return 1.0;
    }
    let ratio = space_needed / available_space;
    // 限制最大压缩比例为0.8 (20%压缩)
    (1.0 - ratio.min(0.2)).max(0.0)
}

/// 查找需要处理的行和对应的文本
///
/// 此函数从 PieceTree 或完整文本中提取指定行的文本内容
pub fn extract_line_texts<'a>(lines: &'a [Line], full_text: &'a str) -> Vec<&'a str> {
    lines
        .iter()
        .map(|line| {
            if line.is_empty() {
                ""
            } else {
                &full_text[line.start..line.end]
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::line_breaking::{BreakType, Line};

    #[test]
    fn test_is_no_start_char() {
        // 行首禁止的标点
        assert!(is_no_start_char('，'));
        assert!(is_no_start_char('。'));
        assert!(is_no_start_char('、'));
        assert!(is_no_start_char('：'));
        assert!(is_no_start_char('；'));
        assert!(is_no_start_char('？'));
        assert!(is_no_start_char('！'));
        assert!(is_no_start_char('）'));
        assert!(is_no_start_char('】'));

        // 允许出现在行首的字符
        assert!(!is_no_start_char('中'));
        assert!(!is_no_start_char('A'));
        assert!(!is_no_start_char('('));
        assert!(!is_no_start_char('1'));
    }

    #[test]
    fn test_is_no_end_char() {
        // 行尾禁止的标点
        assert!(is_no_end_char('（'));
        assert!(is_no_end_char('〔'));
        assert!(is_no_end_char('【'));

        // 允许出现在行尾的字符
        assert!(!is_no_end_char('，'));
        assert!(!is_no_end_char('。'));
        assert!(!is_no_end_char('中'));
        assert!(!is_no_end_char('A'));
    }

    #[test]
    fn test_is_cjk_char() {
        // CJK 汉字
        assert!(is_cjk_char('中'));
        assert!(is_cjk_char('文'));
        assert!(is_cjk_char('汉'));

        // CJK 标点
        assert!(is_cjk_char('，'));
        assert!(is_cjk_char('。'));

        // 日文假名
        assert!(is_cjk_char('あ'));
        assert!(is_cjk_char('ア'));

        // 非 CJK 字符
        assert!(!is_cjk_char('A'));
        assert!(!is_cjk_char('1'));
        assert!(!is_cjk_char('@'));
    }

    #[test]
    fn test_is_shrinkable_punctuation() {
        assert!(is_shrinkable_punctuation('，'));
        assert!(is_shrinkable_punctuation('。'));
        assert!(is_shrinkable_punctuation('、'));
        assert!(is_shrinkable_punctuation('：'));
        assert!(is_shrinkable_punctuation('；'));
        assert!(is_shrinkable_punctuation('？'));
        assert!(is_shrinkable_punctuation('！'));
        assert!(is_shrinkable_punctuation('（'));
        assert!(is_shrinkable_punctuation('）'));
        assert!(is_shrinkable_punctuation('【'));
        assert!(is_shrinkable_punctuation('】'));

        assert!(!is_shrinkable_punctuation('中'));
        assert!(!is_shrinkable_punctuation('A'));
    }

    #[test]
    fn test_calculate_shrink_factor() {
        // 无需挤压
        assert_eq!(calculate_shrink_factor(0.0, 100.0), 1.0);

        // 少量挤压
        let factor = calculate_shrink_factor(5.0, 100.0);
        assert!(factor > 0.8 && factor < 1.0);

        // 最大挤压 (20%)
        let factor = calculate_shrink_factor(20.0, 100.0);
        assert_eq!(factor, 0.8);

        // 超过最大挤压限制
        let factor = calculate_shrink_factor(30.0, 100.0);
        assert_eq!(factor, 0.8);
    }

    #[test]
    fn test_extract_line_texts() {
        let text = "Hello世界，测试。";
        // 注意：Line 使用的是字节偏移
        // "Hello" = 5 字节 [0,5)
        // "世界" = 6 字节 [5,11)
        // "，" = 3 字节 [11,14)
        // "测试" = 6 字节 [14,20)
        // "。" = 3 字节 [20,23)
        let lines = vec![
            Line::new(0, 5, 50.0, BreakType::SoftBreak),    // "Hello"
            Line::new(5, 11, 40.0, BreakType::SoftBreak),   // "世界"
            Line::new(11, 20, 20.0, BreakType::SoftBreak), // "，测试" [11,20)
            Line::new(20, 23, 15.0, BreakType::SoftBreak), // "。" [20,23)
        ];

        let texts = extract_line_texts(&lines, text);
        assert_eq!(texts[0], "Hello");
        assert_eq!(texts[1], "世界");
        assert_eq!(texts[2], "，测试");
        assert_eq!(texts[3], "。");
    }

    #[test]
    fn test_process_for_kinsoku_empty_lines() {
        let lines: [Line; 0] = [];
        let texts: [&str; 0] = [];
        let result = process_for_kinsoku(&lines, &texts, &[], 100.0);
        assert!(result.is_empty());
    }

    #[test]
    fn test_process_for_kinsoku_start_violation() {
        let text = "你好，";
        // "你" = 3 字节, "好" = 3 字节, "，" = 3 字节
        let lines = vec![
            Line::new(0, 6, 20.0, BreakType::SoftBreak), // "你好"
            Line::new(6, 9, 10.0, BreakType::SoftBreak), // "，" (第 6-9 字节)
        ];
        let texts = extract_line_texts(&lines, text);

        let result = process_for_kinsoku(&lines, &texts, &[10.0, 10.0], 10.0);

        // 第二行以 "，" 开头，需要调整
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].line_index, 1);
        assert_eq!(result[0].problem_char, '，');
        assert!(result[0].adjustment_type == AdjustmentType::Wrap ||
                result[0].adjustment_type == AdjustmentType::Hanging);
    }

    #[test]
    fn test_process_for_kinsoku_end_violation() {
        let text = "测试（";
        // "测" = 3, "试" = 3, "（" = 3 字节
        let lines = vec![
            Line::new(0, 9, 30.0, BreakType::SoftBreak), // "测试（" (9 字节)
        ];
        let texts = extract_line_texts(&lines, text);

        let result = process_for_kinsoku(&lines, &texts, &[30.0], 25.0);

        // 行以 "（" 结尾，需要调整
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].line_index, 0);
        assert_eq!(result[0].problem_char, '（');
        assert!(result[0].adjustment_type == AdjustmentType::Shrink ||
                result[0].adjustment_type == AdjustmentType::Wrap);
    }

    #[test]
    fn test_get_char_width() {
        // CJK 字符宽度
        assert_eq!(get_char_width('中', 1.0), 1.0);
        assert_eq!(get_char_width('，', 1.0), 1.0);

        // ASCII 字符宽度减半
        assert_eq!(get_char_width('A', 1.0), 0.5);
    }
}
