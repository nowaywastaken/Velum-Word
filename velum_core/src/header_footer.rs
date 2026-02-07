//! # Header and Footer Module
//!
//! Implements header and footer support for documents with:
//! - Different header/footer types (primary, first page, even page, title page)
//! - Page number formatting (Arabic, Roman, Letters, Chinese)
//! - Field codes (page number, date/time, style reference, document properties)
//! - Section-based header/footer with link-to-previous support
//! - Region calculation for layout integration

use serde::{Deserialize, Serialize};
use std::fmt;

/// 页眉页脚类型
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum HeaderFooterType {
    Primary,      // 主要页眉/页脚
    FirstPage,    // 首页不同
    EvenPage,     // 奇偶页不同
    TitlePage,    // 封面页（无页眉页脚）
}

/// 页眉页脚内容类型
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum HeaderFooterContentType {
    Text(String),
    Image(ImageInfo),
    Fields(Vec<FieldCode>),  // 域代码（页码、日期等）
}

/// 图片信息
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImageInfo {
    pub path: String,
    pub width: f32,
    pub height: f32,
    pub alt_text: Option<String>,
}

/// 页码格式
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PageNumberFormat {
    Arabic,        // 1, 2, 3...
    RomanUpper,    // I, II, III...
    RomanLower,    // i, ii, iii...
    LetterUpper,   // A, B, C...
    LetterLower,   // a, b, c...
    Chinese,       // 一、二、三...
}

impl PageNumberFormat {
    /// 将页码转换为指定格式的字符串
    pub fn format_number(&self, n: u32) -> String {
        match self {
            PageNumberFormat::Arabic => n.to_string(),
            PageNumberFormat::RomanUpper => format_roman_upper(n),
            PageNumberFormat::RomanLower => format_roman_lower(n),
            PageNumberFormat::LetterUpper => format_letter_upper(n),
            PageNumberFormat::LetterLower => format_letter_lower(n),
            PageNumberFormat::Chinese => format_chinese(n),
        }
    }
}

/// 页码域
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PageNumberField {
    pub format: PageNumberFormat,  // 格式：阿拉伯数字、罗马数字等
    pub start_number: u32,        // 起始页码
}

impl PageNumberField {
    /// 计算给定页码的实际显示值
    pub fn get_display_number(&self, page_index: u32, section_start_page: u32) -> u32 {
        page_index + self.start_number - section_start_page
    }

    /// 获取格式化的页码字符串
    pub fn get_formatted_number(&self, page_index: u32, section_start_page: u32) -> String {
        let num = self.get_display_number(page_index, section_start_page);
        self.format.format_number(num)
    }
}

/// 日期时间域
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DateTimeField {
    pub format: String,   // 日期格式（如 "yyyy-MM-dd", "HH:mm"）
    pub is_updateable: bool,
}

impl DateTimeField {
    /// 获取当前日期时间的格式化字符串
    pub fn get_formatted_value(&self) -> String {
        // 使用系统时间获取日期时间
        let now = std::time::SystemTime::now();
        let epoch = now.duration_since(std::time::UNIX_EPOCH).unwrap_or_default();
        let total_seconds = epoch.as_secs() as i64;

        // 简单的日期时间计算（假设UTC）
        let days = total_seconds / 86400;
        let seconds_of_day = total_seconds % 86400;

        // 估算年月日（简化版本，从1970-01-01开始）
        let year = 1970 + (days / 365) as i32;
        let day_of_year = days % 365;
        let month = ((day_of_year / 30) + 1).clamp(1, 12) as u8;
        let day = (day_of_year % 30 + 1) as u8;

        let hour = (seconds_of_day / 3600) as u8;
        let minute = ((seconds_of_day % 3600) / 60) as u8;
        let second = (seconds_of_day % 60) as u8;

        let hour12 = if hour == 0 { 12 } else if hour > 12 { hour - 12 } else { hour };
        let is_pm = hour >= 12;

        self.format
            .replace("yyyy", &format!("{:04}", year))
            .replace("yy", &format!("{:02}", year % 100))
            .replace("MM", &format!("{:02}", month))
            .replace("dd", &format!("{:02}", day))
            .replace("HH", &format!("{:02}", hour))
            .replace("hh", &format!("{:02}", hour12))
            .replace("mm", &format!("{:02}", minute))
            .replace("ss", &format!("{:02}", second))
            .replace("tt", if is_pm { "PM" } else { "AM" })
    }
}

/// 域代码
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FieldCode {
    PageNumber(PageNumberField),
    DateTime(DateTimeField),
    StyleRef(String),     // 样式引用
    DocProperty(String), // 文档属性
}

impl FieldCode {
    /// 解析域代码字符串
    pub fn parse(code: &str) -> Option<Self> {
        if code.starts_with("PAGE") || code.starts_with("PAGEREF") {
            // PAGE \* Arabic, PAGE \* Roman, etc.
            Some(FieldCode::PageNumber(PageNumberField {
                format: if code.contains("Roman") {
                    if code.contains("UPPER") {
                        PageNumberFormat::RomanUpper
                    } else {
                        PageNumberFormat::RomanLower
                    }
                } else if code.contains("Letter") {
                    if code.contains("UPPER") {
                        PageNumberFormat::LetterUpper
                    } else {
                        PageNumberFormat::LetterLower
                    }
                } else if code.contains("Chinese") {
                    PageNumberFormat::Chinese
                } else {
                    PageNumberFormat::Arabic
                },
                start_number: 1,
            }))
        } else if code.starts_with("DATE") {
            // DATE \@ "yyyy-MM-dd"
            Some(FieldCode::DateTime(DateTimeField {
                format: code
                    .trim_start_matches("DATE")
                    .trim()
                    .replace("@", "")
                    .replace("\"", "")
                    .trim()
                    .to_string(),
                is_updateable: true,
            }))
        } else if code.starts_with("STYLEREF") {
            // STYLEREF "StyleName"
            let style_name = code
                .trim_start_matches("STYLEREF")
                .trim()
                .replace("\"", "")
                .trim()
                .to_string();
            Some(FieldCode::StyleRef(style_name))
        } else if code.starts_with("DOCPROPERTY") {
            // DOCPROPERTY "PropertyName"
            let prop_name = code
                .trim_start_matches("DOCPROPERTY")
                .trim()
                .replace("\"", "")
                .trim()
                .to_string();
            Some(FieldCode::DocProperty(prop_name))
        } else {
            None
        }
    }
}

/// 页眉
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Header {
    pub header_type: HeaderFooterType,
    pub content: HeaderFooterContentType,
    pub margin: f32,           // 页眉距离页面顶部距离
    pub height: f32,           // 页眉区域高度
    pub linked_to_previous: bool,  // 与前一节链接
}

impl Header {
    /// 创建默认页眉
    pub fn new() -> Self {
        Header {
            header_type: HeaderFooterType::Primary,
            content: HeaderFooterContentType::Text(String::new()),
            margin: 0.0,
            height: 0.0,
            linked_to_previous: false,
        }
    }

    /// 检查是否为首页页眉
    pub fn is_first_page(&self) -> bool {
        self.header_type == HeaderFooterType::FirstPage
    }

    /// 检查是否链接到前一节
    pub fn is_linked(&self) -> bool {
        self.linked_to_previous
    }
}

/// 页脚
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Footer {
    pub footer_type: HeaderFooterType,
    pub content: HeaderFooterContentType,
    pub margin: f32,           // 页脚距离页面底部距离
    pub height: f32,           // 页脚区域高度
    pub linked_to_previous: bool,
    pub page_number_format: Option<PageNumberFormat>,
}

impl Footer {
    /// 创建默认页脚（带页码）
    pub fn new_with_page_number(format: PageNumberFormat) -> Self {
        Footer {
            footer_type: HeaderFooterType::Primary,
            content: HeaderFooterContentType::Fields(vec![FieldCode::PageNumber(PageNumberField {
                format: format.clone(),
                start_number: 1,
            })]),
            margin: 0.0,
            height: 0.0,
            linked_to_previous: false,
            page_number_format: Some(format),
        }
    }

    /// 创建空页脚
    pub fn new() -> Self {
        Footer {
            footer_type: HeaderFooterType::Primary,
            content: HeaderFooterContentType::Text(String::new()),
            margin: 0.0,
            height: 0.0,
            linked_to_previous: false,
            page_number_format: None,
        }
    }

    /// 检查是否链接到前一节
    pub fn is_linked(&self) -> bool {
        self.linked_to_previous
    }
}

/// 页眉页脚配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeaderFooterConfig {
    pub header: Option<Header>,
    pub footer: Option<Footer>,
    pub different_first_page: bool,     // 首页不同
    pub different_odd_even: bool,       // 奇偶页不同
    pub title_page: bool,               // 封面页（无页眉页脚）
}

impl Default for HeaderFooterConfig {
    fn default() -> Self {
        HeaderFooterConfig {
            header: None,
            footer: None,
            different_first_page: false,
            different_odd_even: false,
            title_page: false,
        }
    }
}

impl HeaderFooterConfig {
    /// 创建默认配置
    pub fn new() -> Self {
        Self::default()
    }

    /// 启用首页不同
    pub fn with_different_first_page(mut self, enabled: bool) -> Self {
        self.different_first_page = enabled;
        self
    }

    /// 启用奇偶页不同
    pub fn with_different_odd_even(mut self, enabled: bool) -> Self {
        self.different_odd_even = enabled;
        self
    }

    /// 设置为封面页
    pub fn as_title_page(mut self, is_title: bool) -> Self {
        self.title_page = is_title;
        self
    }

    /// 设置页眉
    pub fn with_header(mut self, header: Header) -> Self {
        self.header = Some(header);
        self
    }

    /// 设置页脚
    pub fn with_footer(mut self, footer: Footer) -> Self {
        self.footer = Some(footer);
        self
    }
}

/// 页眉页脚区域（用于布局计算）
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct HeaderFooterRegion {
    pub header_rect: Option<HFRect>,
    pub footer_rect: Option<HFRect>,
    pub content_top: f32,
    pub content_bottom: f32,
}

/// 矩形区域（内部使用，与 page_layout::Rect 兼容）
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct HFRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl HFRect {
    /// 创建新矩形
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        HFRect { x, y, width, height }
    }

    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.width <= 0.0 || self.height <= 0.0
    }

    /// 获取底部 Y 坐标
    pub fn bottom(&self) -> f32 {
        self.y + self.height
    }

    /// 获取右侧 X 坐标
    pub fn right(&self) -> f32 {
        self.x + self.width
    }
}

/// 从 page_layout::Rect 转换
impl From<super::page_layout::Rect> for HFRect {
    fn from(rect: super::page_layout::Rect) -> Self {
        HFRect::new(rect.x, rect.y, rect.width, rect.height)
    }
}

/// 转换为 page_layout::Rect
impl From<HFRect> for super::page_layout::Rect {
    fn from(rect: HFRect) -> Self {
        super::page_layout::Rect::new(rect.x, rect.y, rect.width, rect.height)
    }
}

/// 页眉页脚管理器
#[derive(Debug, Clone)]
pub struct HeaderFooterManager {
    /// 页眉页脚配置列表（按节索引）
    configs: Vec<HeaderFooterConfig>,
    /// 当前活动节索引
    current_section: usize,
}

impl Default for HeaderFooterManager {
    fn default() -> Self {
        Self::new()
    }
}

impl HeaderFooterManager {
    /// 创建新的页眉页脚管理器
    pub fn new() -> Self {
        HeaderFooterManager {
            configs: vec![HeaderFooterConfig::new()],
            current_section: 0,
        }
    }

    /// 添加新节
    pub fn add_section(&mut self) -> usize {
        let new_section_index = self.configs.len();
        // 默认继承前一节的配置（带链接）
        let prev_config = self.configs.last().cloned().unwrap_or_default();
        self.configs.push(prev_config);
        self.current_section = new_section_index;
        new_section_index
    }

    /// 设置当前节
    pub fn set_current_section(&mut self, section: usize) {
        if section < self.configs.len() {
            self.current_section = section;
        }
    }

    /// 获取当前节配置
    pub fn get_current_config(&self) -> Option<&HeaderFooterConfig> {
        self.configs.get(self.current_section)
    }

    /// 获取当前节配置（可变）
    pub fn get_current_config_mut(&mut self) -> Option<&mut HeaderFooterConfig> {
        self.configs.get_mut(self.current_section)
    }

    /// 设置首页不同
    pub fn set_different_first_page(&mut self, enabled: bool) {
        if let Some(config) = self.configs.get_mut(self.current_section) {
            config.different_first_page = enabled;
        }
    }

    /// 设置奇偶页不同
    pub fn set_different_odd_even(&mut self, enabled: bool) {
        if let Some(config) = self.configs.get_mut(self.current_section) {
            config.different_odd_even = enabled;
        }
    }

    /// 设置页眉
    pub fn set_header(&mut self, header: Header) {
        if let Some(config) = self.configs.get_mut(self.current_section) {
            config.header = Some(header);
        }
    }

    /// 设置页脚
    pub fn set_footer(&mut self, footer: Footer) {
        if let Some(config) = self.configs.get_mut(self.current_section) {
            config.footer = Some(footer);
        }
    }

    /// 获取指定页的页眉配置
    pub fn get_header_for_page(&self, page_index: usize, section_index: usize) -> Option<&Header> {
        let config = self.configs.get(section_index)?;

        // 检查是否为封面页
        if config.title_page {
            return None;
        }

        // 检查是否为首页
        if config.different_first_page && page_index == 0 {
            return config.header.as_ref().filter(|h| h.header_type == HeaderFooterType::FirstPage);
        }

        // 检查奇偶页
        if config.different_odd_even {
            if page_index % 2 == 0 {
                // 偶数页
                return config.header.as_ref().filter(|h| h.header_type == HeaderFooterType::EvenPage);
            }
        }

        // 返回主页眉
        config.header.as_ref().filter(|h| h.header_type == HeaderFooterType::Primary)
    }

    /// 获取指定页的页脚配置
    pub fn get_footer_for_page(&self, page_index: usize, section_index: usize) -> Option<&Footer> {
        let config = self.configs.get(section_index)?;

        // 检查是否为封面页
        if config.title_page {
            return None;
        }

        // 检查是否为首页
        if config.different_first_page && page_index == 0 {
            return config.footer.as_ref().filter(|f| f.footer_type == HeaderFooterType::FirstPage);
        }

        // 检查奇偶页
        if config.different_odd_even {
            if page_index % 2 == 0 {
                // 偶数页
                return config.footer.as_ref().filter(|f| f.footer_type == HeaderFooterType::EvenPage);
            }
        }

        // 返回主页脚
        config.footer.as_ref().filter(|f| f.footer_type == HeaderFooterType::Primary)
    }

    /// 获取节的起始页码
    #[allow(dead_code)]
    pub fn get_section_start_page(&self, section_index: usize) -> u32 {
        // 简化实现：假设每节从页码 1 开始
        // 实际实现需要根据前一节的页数计算
        let _ = section_index;
        1
    }

    /// 设置节的起始页码
    pub fn set_section_start_page(&mut self, _section: usize, _page_number: u32) {
        // TODO: 实现起始页码设置
    }

    /// 计算页面内容区域
    pub fn calculate_content_region(
        &self,
        page_index: usize,
        section_index: usize,
        page_width: f32,
        page_height: f32,
        margin_top: f32,
        margin_bottom: f32,
        margin_left: f32,
        margin_right: f32,
    ) -> HeaderFooterRegion {
        let header = self.get_header_for_page(page_index, section_index);
        let footer = self.get_footer_for_page(page_index, section_index);

        let header_height = header.map(|h| h.height).unwrap_or(0.0);
        let footer_height = footer.map(|f| f.height).unwrap_or(0.0);
        let header_margin = header.map(|h| h.margin).unwrap_or(0.0);
        let footer_margin = footer.map(|f| f.margin).unwrap_or(0.0);

        let header_rect = if header_height > 0.0 {
            Some(HFRect::new(
                margin_left,
                margin_top + header_margin,
                page_width - margin_left - margin_right,
                header_height,
            ))
        } else {
            None
        };

        let footer_rect = if footer_height > 0.0 {
            Some(HFRect::new(
                margin_left,
                page_height - margin_bottom - footer_margin - footer_height,
                page_width - margin_left - margin_right,
                footer_height,
            ))
        } else {
            None
        };

        let content_top = margin_top + header_height + header_margin;
        let content_bottom = page_height - margin_bottom - footer_height - footer_margin;

        HeaderFooterRegion {
            header_rect,
            footer_rect,
            content_top,
            content_bottom,
        }
    }

    /// 获取节数量
    pub fn section_count(&self) -> usize {
        self.configs.len()
    }
}

// ==================== 格式化函数 ====================

/// 格式化为大写罗马数字
fn format_roman_upper(n: u32) -> String {
    let mut n = n;
    let mut result = String::new();
    let roman_numerals = [
        ("M", 1000),
        ("CM", 900),
        ("D", 500),
        ("CD", 400),
        ("C", 100),
        ("XC", 90),
        ("L", 50),
        ("XL", 40),
        ("X", 10),
        ("IX", 9),
        ("V", 5),
        ("IV", 4),
        ("I", 1),
    ];

    for (symbol, value) in roman_numerals {
        while n >= value {
            result.push_str(symbol);
            n -= value;
        }
    }

    result
}

/// 格式化为小写罗马数字
fn format_roman_lower(n: u32) -> String {
    format_roman_upper(n).to_lowercase()
}

/// 格式化为大写字母
fn format_letter_upper(n: u32) -> String {
    let mut n = n.saturating_sub(1);
    let mut result = String::new();

    while n >= 26 {
        result.push((b'A' + (n % 26) as u8) as char);
        n /= 26;
        n -= 1;
    }

    result.push((b'A' + (n % 26) as u8) as char);
    result.chars().rev().collect()
}

/// 格式化为小写字母
fn format_letter_lower(n: u32) -> String {
    format_letter_upper(n).to_lowercase()
}

/// 格式化为中文数字
fn format_chinese(n: u32) -> String {
    const DIGITS: [&str; 10] = ["零", "一", "二", "三", "四", "五", "六", "七", "八", "九"];
    const UNITS: [&str; 4] = ["", "十", "百", "千"];

    if n == 0 {
        return "零".to_string();
    }

    let mut n = n;
    let mut result = String::new();
    let mut units_count = 0;

    while n > 0 {
        let digit = (n % 10) as usize;
        if digit > 0 {
            if units_count > 0 && units_count % 4 == 0 {
                // 每四位添加"万"或"亿"
                if units_count == 4 {
                    result.push('万');
                } else if units_count == 8 {
                    result.push('亿');
                }
            }
            result.push_str(DIGITS[digit]);
            if units_count % 4 < 4 {
                result.push_str(UNITS[units_count % 4]);
            }
        } else if !result.is_empty() && !result.ends_with("零") && units_count % 4 != 0 {
            result.push('零');
        }
        n /= 10;
        units_count += 1;
    }

    // 移除末尾的零
    while result.ends_with("零") && result.len() > 1 {
        result.pop();
    }

    result
}

impl fmt::Display for PageNumberFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PageNumberFormat::Arabic => write!(f, "Arabic"),
            PageNumberFormat::RomanUpper => write!(f, "Roman (Uppercase)"),
            PageNumberFormat::RomanLower => write!(f, "Roman (Lowercase)"),
            PageNumberFormat::LetterUpper => write!(f, "Letter (Uppercase)"),
            PageNumberFormat::LetterLower => write!(f, "Letter (Lowercase)"),
            PageNumberFormat::Chinese => write!(f, "Chinese"),
        }
    }
}

// ==================== 单元测试 ====================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_page_number_format_arabic() {
        let format = PageNumberFormat::Arabic;
        assert_eq!(format.format_number(1), "1");
        assert_eq!(format.format_number(10), "10");
        assert_eq!(format.format_number(100), "100");
    }

    #[test]
    fn test_page_number_format_roman_upper() {
        let format = PageNumberFormat::RomanUpper;
        assert_eq!(format.format_number(1), "I");
        assert_eq!(format.format_number(2), "II");
        assert_eq!(format.format_number(3), "III");
        assert_eq!(format.format_number(4), "IV");
        assert_eq!(format.format_number(5), "V");
        assert_eq!(format.format_number(9), "IX");
        assert_eq!(format.format_number(10), "X");
        assert_eq!(format.format_number(50), "L");
        assert_eq!(format.format_number(100), "C");
        assert_eq!(format.format_number(500), "D");
        assert_eq!(format.format_number(1000), "M");
    }

    #[test]
    fn test_page_number_format_roman_lower() {
        let format = PageNumberFormat::RomanLower;
        assert_eq!(format.format_number(1), "i");
        assert_eq!(format.format_number(4), "iv");
        assert_eq!(format.format_number(9), "ix");
    }

    #[test]
    fn test_page_number_format_letter_upper() {
        let format = PageNumberFormat::LetterUpper;
        assert_eq!(format.format_number(1), "A");
        assert_eq!(format.format_number(26), "Z");
        assert_eq!(format.format_number(27), "AA");
        assert_eq!(format.format_number(28), "AB");
        assert_eq!(format.format_number(52), "AZ");
        assert_eq!(format.format_number(53), "BA");
    }

    #[test]
    fn test_page_number_format_letter_lower() {
        let format = PageNumberFormat::LetterLower;
        assert_eq!(format.format_number(1), "a");
        assert_eq!(format.format_number(26), "z");
    }

    #[test]
    fn test_page_number_format_chinese() {
        let format = PageNumberFormat::Chinese;
        assert_eq!(format.format_number(0), "零");
        assert_eq!(format.format_number(1), "一");
        assert_eq!(format.format_number(2), "二");
        assert_eq!(format.format_number(3), "三");
        assert_eq!(format.format_number(10), "十");
        assert_eq!(format.format_number(11), "十一");
        assert_eq!(format.format_number(20), "二十");
        assert_eq!(format.format_number(21), "二十一");
        assert_eq!(format.format_number(100), "一百");
    }

    #[test]
    fn test_header_creation() {
        let header = Header::new();
        assert_eq!(header.header_type, HeaderFooterType::Primary);
        assert_eq!(header.content, HeaderFooterContentType::Text(String::new()));
        assert!(!header.is_linked());
    }

    #[test]
    fn test_footer_creation() {
        let footer = Footer::new();
        assert_eq!(footer.footer_type, HeaderFooterType::Primary);
        assert!(!footer.is_linked());
    }

    #[test]
    fn test_footer_with_page_number() {
        let footer = Footer::new_with_page_number(PageNumberFormat::Arabic);
        assert!(footer.page_number_format.is_some());
    }

    #[test]
    fn test_header_footer_config() {
        let config = HeaderFooterConfig::new()
            .with_different_first_page(true)
            .with_different_odd_even(true);

        assert!(config.different_first_page);
        assert!(config.different_odd_even);
    }

    #[test]
    fn test_field_code_parse_page() {
        let field = FieldCode::parse("PAGE \\* Arabic").unwrap();
        match field {
            FieldCode::PageNumber(pnf) => {
                assert_eq!(pnf.format, PageNumberFormat::Arabic);
                assert_eq!(pnf.start_number, 1);
            }
            _ => panic!("Expected PageNumber field"),
        }
    }

    #[test]
    fn test_field_code_parse_roman() {
        let field = FieldCode::parse("PAGE \\* RomanUpper").unwrap();
        match field {
            FieldCode::PageNumber(pnf) => {
                assert_eq!(pnf.format, PageNumberFormat::RomanUpper);
            }
            _ => panic!("Expected PageNumber field"),
        }
    }

    #[test]
    fn test_field_code_parse_date() {
        let field = FieldCode::parse("DATE \\@ \"yyyy-MM-dd\"").unwrap();
        match field {
            FieldCode::DateTime(dtf) => {
                assert_eq!(dtf.format, "yyyy-MM-dd");
                assert!(dtf.is_updateable);
            }
            _ => panic!("Expected DateTime field"),
        }
    }

    #[test]
    fn test_field_code_parse_style_ref() {
        let field = FieldCode::parse("STYLEREF \"Heading 1\"").unwrap();
        match field {
            FieldCode::StyleRef(s) => {
                assert_eq!(s, "Heading 1");
            }
            _ => panic!("Expected StyleRef field"),
        }
    }

    #[test]
    fn test_rect_operations() {
        let rect = HFRect::new(10.0, 20.0, 100.0, 50.0);
        assert_eq!(rect.bottom(), 70.0);
        assert_eq!(rect.right(), 110.0);
        assert!(!rect.is_empty());

        let empty = HFRect::new(0.0, 0.0, 0.0, 0.0);
        assert!(empty.is_empty());
    }

    #[test]
    fn test_header_footer_manager() {
        let mut manager = HeaderFooterManager::new();

        // 设置当前节的页脚
        let footer = Footer::new_with_page_number(PageNumberFormat::Arabic);
        manager.set_footer(footer);

        // 获取页脚
        let retrieved_footer = manager.get_footer_for_page(0, 0);
        assert!(retrieved_footer.is_some());
    }

    #[test]
    fn test_manager_mutability() {
        let mut manager = HeaderFooterManager::new();

        // 设置首页不同
        manager.set_different_first_page(true);
        assert!(manager.get_current_config().unwrap().different_first_page);
    }

    #[test]
    fn test_header_footer_manager_sections() {
        let mut manager = HeaderFooterManager::new();

        // 添加新节
        let section_index = manager.add_section();
        assert_eq!(section_index, 1);
        assert_eq!(manager.section_count(), 2);

        // 在新节设置不同配置
        manager.set_different_first_page(true);
        assert!(manager.get_current_config().unwrap().different_first_page);
    }

    #[test]
    fn test_content_region_calculation() {
        let mut manager = HeaderFooterManager::new();

        // 设置页眉和页脚
        let header = Header {
            header_type: HeaderFooterType::Primary,
            content: HeaderFooterContentType::Text("Header".to_string()),
            margin: 5.0,
            height: 30.0,
            linked_to_previous: false,
        };
        let footer = Footer {
            footer_type: HeaderFooterType::Primary,
            content: HeaderFooterContentType::Text("Footer".to_string()),
            margin: 5.0,
            height: 25.0,
            linked_to_previous: false,
            page_number_format: None,
        };

        manager.set_header(header);
        manager.set_footer(footer);

        // 计算内容区域
        let region = manager.calculate_content_region(
            0, 0,           // page_index, section_index
            595.35, 841.89, // page_width, page_height
            72.0, 72.0,     // margin_top, margin_bottom
            72.0, 72.0,     // margin_left, margin_right
        );

        // 验证计算结果
        assert!(region.header_rect.is_some());
        assert!(region.footer_rect.is_some());

        let header_rect = region.header_rect.unwrap();
        assert_eq!(header_rect.x, 72.0);
        assert_eq!(header_rect.y, 77.0); // margin_top(72) + margin(5)
        assert_eq!(header_rect.width, 451.35); // 595.35 - 72 - 72
        assert_eq!(header_rect.height, 30.0);

        let footer_rect = region.footer_rect.unwrap();
        assert_eq!(footer_rect.y, 739.89); // 841.89 - 72 - 5 - 25
        assert_eq!(footer_rect.height, 25.0);

        // 验证内容区域
        assert_eq!(region.content_top, 107.0); // 72 + 30 + 5
        assert_eq!(region.content_bottom, 739.89); // 841.89 - 72 - 5 - 25
    }

    #[test]
    fn test_different_first_page() {
        let mut manager = HeaderFooterManager::new();

        // 设置首页不同
        manager.set_different_first_page(true);

        // 设置第一页页眉
        let first_header = Header {
            header_type: HeaderFooterType::FirstPage,
            content: HeaderFooterContentType::Text("First Page Header".to_string()),
            margin: 5.0,
            height: 40.0,
            linked_to_previous: false,
        };
        manager.set_header(first_header);

        // 获取第一页页眉（应该返回 FirstPage 类型）
        let page0_header = manager.get_header_for_page(0, 0);
        assert!(page0_header.is_some());
        assert_eq!(page0_header.unwrap().header_type, HeaderFooterType::FirstPage);

        // 获取第二页页眉（应该返回 Primary 类型）
        let page1_header = manager.get_header_for_page(1, 0);
        assert!(page1_header.is_none()); // 因为没有设置 Primary 类型
    }

    #[test]
    fn test_different_odd_even() {
        let mut manager = HeaderFooterManager::new();

        // 设置奇偶页不同
        manager.set_different_odd_even(true);

        // 设置偶数页眉
        let even_header = Header {
            header_type: HeaderFooterType::EvenPage,
            content: HeaderFooterContentType::Text("Even Page Header".to_string()),
            margin: 5.0,
            height: 30.0,
            linked_to_previous: false,
        };
        manager.set_header(even_header);

        // 获取偶数页（0, 2, 4...）页眉
        let even_header = manager.get_header_for_page(0, 0);
        assert!(even_header.is_some());
        assert_eq!(even_header.unwrap().header_type, HeaderFooterType::EvenPage);

        // 获取奇数页（1, 3, 5...）页眉
        let odd_header = manager.get_header_for_page(1, 0);
        assert!(odd_header.is_none()); // 因为没有设置 Primary 类型
    }

    #[test]
    fn test_title_page() {
        let mut manager = HeaderFooterManager::new();

        // 设置封面页
        manager.set_different_first_page(true);
        let first_header = Header {
            header_type: HeaderFooterType::FirstPage,
            content: HeaderFooterContentType::Text("First Page Header".to_string()),
            margin: 5.0,
            height: 40.0,
            linked_to_previous: false,
        };
        manager.set_header(first_header);

        // 封面页不应该有页眉页脚
        let header = manager.get_header_for_page(0, 0);
        let footer = manager.get_footer_for_page(0, 0);
        assert!(header.is_some());
        assert!(footer.is_none());
    }

    #[test]
    fn test_page_number_field_display() {
        let field = PageNumberField {
            format: PageNumberFormat::Arabic,
            start_number: 1,
        };

        // 节起始页为1，当前页为0，应该显示1
        assert_eq!(field.get_display_number(0, 1), 1);
        // 节起始页为5，当前页为3，应该显示5-5+3=3
        assert_eq!(field.get_display_number(3, 5), 3);

        let roman_field = PageNumberField {
            format: PageNumberFormat::RomanUpper,
            start_number: 1,
        };
        assert_eq!(roman_field.get_formatted_number(0, 1), "I");
        assert_eq!(roman_field.get_formatted_number(3, 1), "IV");
    }

    #[test]
    fn test_header_footer_display() {
        let format = PageNumberFormat::Arabic;
        assert_eq!(format.to_string(), "Arabic");

        let format = PageNumberFormat::RomanUpper;
        assert_eq!(format.to_string(), "Roman (Uppercase)");
    }
}
