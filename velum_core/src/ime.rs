//! IME (Input Method Editor) 输入支持模块
//!
//! 提供 IME 输入状态管理、组合文本处理、候选词窗口支持等功能。

use serde::{Serialize, Deserialize};
use std::fmt;

/// IME 状态
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ImeState {
    /// 是否正在组合文本
    pub is_composing: bool,
    /// 当前组合文本
    pub composing_text: String,
    /// 组合文本在文档中的范围 (起始位置, 结束位置)
    pub composing_range: (usize, usize),
    /// 当前选区范围 (起始位置, 结束位置)
    pub selection_range: (usize, usize),
}

impl ImeState {
    /// 创建一个新的 IME 状态
    pub fn new() -> Self {
        ImeState {
            is_composing: false,
            composing_text: String::new(),
            composing_range: (0, 0),
            selection_range: (0, 0),
        }
    }

    /// 重置状态
    pub fn reset(&mut self) {
        self.is_composing = false;
        self.composing_text.clear();
        self.composing_range = (0, 0);
    }

    /// 开始组合
    pub fn start_composition(&mut self, position: usize) {
        self.is_composing = true;
        self.composing_text.clear();
        self.composing_range = (position, position);
        self.selection_range = (position, position);
    }

    /// 更新组合文本
    pub fn update_composition(&mut self, text: String, cursor_pos: usize) {
        let text_len = text.len();
        self.composing_text = text;
        let start = self.composing_range.0;
        let end = start + text_len;
        self.composing_range = (start, end);
        self.selection_range = (start + cursor_pos, start + cursor_pos);
    }

    /// 获取组合文本长度
    pub fn composition_length(&self) -> usize {
        self.composing_text.len()
    }
}

/// IME 提交信息
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImeCommit {
    /// 提交的文本
    pub text: String,
    /// 提交位置（文档中的起始位置）
    pub position: usize,
    /// 提交长度
    pub committed_length: usize,
}

impl ImeCommit {
    /// 创建一个新的提交信息
    pub fn new(text: String, position: usize) -> Self {
        let len = text.len();
        ImeCommit {
            text,
            position,
            committed_length: len,
        }
    }
}

/// IME 预编辑信息
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImePreEdit {
    /// 预编辑文本
    pub text: String,
    /// 光标位置（相对于预编辑文本开头）
    pub cursor_position: usize,
    /// 选中文本范围 (起始, 结束)，相对于预编辑文本
    pub selection: Option<(usize, usize)>,
}

impl ImePreEdit {
    /// 创建一个新的预编辑信息
    pub fn new(text: String, cursor_position: usize) -> Self {
        ImePreEdit {
            text,
            cursor_position,
            selection: None,
        }
    }

    /// 创建带选区的预编辑信息
    pub fn with_selection(text: String, cursor_position: usize, selection: (usize, usize)) -> Self {
        ImePreEdit {
            text,
            cursor_position,
            selection: Some(selection),
        }
    }
}

/// IME 事件类型
#[derive(Debug, Clone, PartialEq)]
pub enum ImeEvent {
    /// 开始组合
    CompositionStart,
    /// 更新组合文本
    CompositionUpdate(String),
    /// 结束组合
    CompositionEnd(String),
    /// 直接提交（未组合的输入）
    Commit(String),
    /// 切换输入法
    InputMethodChanged(String),
}

impl fmt::Display for ImeEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ImeEvent::CompositionStart => write!(f, "CompositionStart"),
            ImeEvent::CompositionUpdate(text) => write!(f, "CompositionUpdate({})", text),
            ImeEvent::CompositionEnd(text) => write!(f, "CompositionEnd({})", text),
            ImeEvent::Commit(text) => write!(f, "Commit({})", text),
            ImeEvent::InputMethodChanged(name) => write!(f, "InputMethodChanged({})", name),
        }
    }
}

/// 候选词信息
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CandidateWord {
    /// 候选词文本
    pub text: String,
    /// 候选词索引
    pub index: usize,
    /// 候选词类型（用于显示样式）
    pub word_type: CandidateWordType,
}

impl CandidateWord {
    /// 创建一个新的候选词
    pub fn new(text: String, index: usize) -> Self {
        CandidateWord {
            text,
            index,
            word_type: CandidateWordType::Normal,
        }
    }

    /// 创建一个带类型的候选词
    pub fn with_type(text: String, index: usize, word_type: CandidateWordType) -> Self {
        CandidateWord {
            text,
            index,
            word_type,
        }
    }
}

/// 候选词类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CandidateWordType {
    /// 普通候选词
    Normal,
    /// 常用词/高频词
    Frequent,
    /// 用户自定义词
    UserDefined,
    /// 表情符号
    Emoji,
    /// 符号
    Symbol,
}

impl Default for CandidateWordType {
    fn default() -> Self {
        CandidateWordType::Normal
    }
}

/// 候选词窗口配置
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CandidateWindowConfig {
    /// 每页显示候选词数量
    pub page_size: usize,
    /// 最大显示页数
    pub max_pages: usize,
    /// 候选词最大长度
    pub max_word_length: usize,
    /// 是否显示页码
    pub show_page_number: bool,
    /// 是否启用实时搜索
    pub enable_realtime_search: bool,
}

impl Default for CandidateWindowConfig {
    fn default() -> Self {
        CandidateWindowConfig {
            page_size: 5,
            max_pages: 10,
            max_word_length: 32,
            show_page_number: true,
            enable_realtime_search: true,
        }
    }
}

/// 候选词窗口
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct CandidateWindow {
    /// 候选词列表
    candidates: Vec<CandidateWord>,
    /// 当前页码（从0开始）
    current_page: usize,
    /// 总页数
    total_pages: usize,
    /// 配置
    config: CandidateWindowConfig,
    /// 当前选中的候选词索引
    selected_index: Option<usize>,
}

impl CandidateWindow {
    /// 创建一个新的候选词窗口
    pub fn new() -> Self {
        CandidateWindow {
            candidates: Vec::new(),
            current_page: 0,
            total_pages: 0,
            config: CandidateWindowConfig::default(),
            selected_index: None,
        }
    }

    /// 创建一个带配置的候选词窗口
    pub fn with_config(config: CandidateWindowConfig) -> Self {
        CandidateWindow {
            candidates: Vec::new(),
            current_page: 0,
            total_pages: 0,
            config,
            selected_index: None,
        }
    }

    /// 设置候选词列表
    pub fn set_candidates(&mut self, candidates: Vec<CandidateWord>) {
        self.candidates = candidates;
        self.total_pages = (self.candidates.len() + self.config.page_size - 1) / self.config.page_size;
        self.current_page = 0;
        self.selected_index = None;
    }

    /// 获取当前页的候选词
    pub fn get_current_page_candidates(&self) -> &[CandidateWord] {
        let start = self.current_page * self.config.page_size;
        let end = std::cmp::min(start + self.config.page_size, self.candidates.len());
        &self.candidates[start..end]
    }

    /// 获取当前选中的候选词
    pub fn get_selected_candidate(&self) -> Option<&CandidateWord> {
        self.selected_index.and_then(|idx| self.candidates.get(idx))
    }

    /// 选择下一个候选词
    pub fn select_next(&mut self) -> bool {
        let page_start = self.current_page * self.config.page_size;
        let page_end = std::cmp::min(page_start + self.config.page_size, self.candidates.len());

        if self.selected_index.is_none() {
            if page_start < page_end {
                self.selected_index = Some(page_start);
                return true;
            }
        } else if let Some(idx) = self.selected_index {
            if idx + 1 < page_end {
                self.selected_index = Some(idx + 1);
                return true;
            }
        }
        false
    }

    /// 选择上一个候选词
    pub fn select_prev(&mut self) -> bool {
        if let Some(idx) = self.selected_index {
            if idx > 0 {
                self.selected_index = Some(idx - 1);
                return true;
            }
        }
        false
    }

    /// 翻到下一页
    pub fn next_page(&mut self) -> bool {
        if self.current_page + 1 < self.total_pages {
            self.current_page += 1;
            self.selected_index = None;
            return true;
        }
        false
    }

    /// 翻到上一页
    pub fn prev_page(&mut self) -> bool {
        if self.current_page > 0 {
            self.current_page -= 1;
            self.selected_index = None;
            return true;
        }
        false
    }

    /// 获取当前页码（1-based，用于显示）
    pub fn current_page_number(&self) -> usize {
        self.current_page + 1
    }

    /// 获取总页数
    pub fn total_pages(&self) -> usize {
        self.total_pages
    }

    /// 获取候选词总数
    pub fn total_count(&self) -> usize {
        self.candidates.len()
    }

    /// 是否为空
    pub fn is_empty(&self) -> bool {
        self.candidates.is_empty()
    }

    /// 清空候选词
    pub fn clear(&mut self) {
        self.candidates.clear();
        self.current_page = 0;
        self.total_pages = 0;
        self.selected_index = None;
    }

    /// 根据索引选择候选词
    pub fn select_by_index(&mut self, global_index: usize) -> bool {
        if global_index < self.candidates.len() {
            self.selected_index = Some(global_index);
            self.current_page = global_index / self.config.page_size;
            return true;
        }
        false
    }

    /// 获取本地索引（在当前页中的位置）
    pub fn local_selected_index(&self) -> Option<usize> {
        if let Some(global_idx) = self.selected_index {
            let page_start = self.current_page * self.config.page_size;
            return Some(global_idx - page_start);
        }
        None
    }
}

/// IME 配置
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImeConfig {
    /// 是否启用 IME
    pub enabled: bool,
    /// 是否自动提交组合文本
    pub auto_commit: bool,
    /// 组合文本最小长度触发提交
    pub min_commit_length: usize,
    /// 是否显示候选词窗口
    pub show_candidate_window: bool,
    /// 候选词窗口配置
    pub candidate_window_config: CandidateWindowConfig,
    /// 是否在组合时显示下划线
    pub show_composition_underline: bool,
    /// 组合下划线样式
    pub underline_style: UnderlineStyle,
}

impl Default for ImeConfig {
    fn default() -> Self {
        ImeConfig {
            enabled: true,
            auto_commit: true,
            min_commit_length: 0,
            show_candidate_window: true,
            candidate_window_config: CandidateWindowConfig::default(),
            show_composition_underline: true,
            underline_style: UnderlineStyle::Solid,
        }
    }
}

/// 下划线样式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnderlineStyle {
    /// 实线
    Solid,
    /// 虚线
    Dashed,
    /// 点线
    Dotted,
    /// 波纹线
    Wavy,
    /// 无下划线
    None,
}

impl Default for UnderlineStyle {
    fn default() -> Self {
        UnderlineStyle::Solid
    }
}

/// IME 处理器
///
/// 负责处理 IME 事件并管理与 PieceTree 的交互
pub struct ImeHandler {
    /// 当前 IME 状态
    pub state: ImeState,
    /// 当前配置
    pub config: ImeConfig,
    /// 候选词窗口
    pub candidate_window: CandidateWindow,
    /// 当前输入法名称
    current_input_method: String,
    /// 事件回调列表
    event_callbacks: Vec<Box<dyn Fn(ImeEvent) + Send>>,
}

impl Default for ImeHandler {
    fn default() -> Self {
        ImeHandler::new()
    }
}

impl std::fmt::Debug for ImeHandler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ImeHandler")
            .field("state", &self.state)
            .field("config", &self.config)
            .field("candidate_window", &self.candidate_window)
            .field("current_input_method", &self.current_input_method)
            .finish()
    }
}

impl Clone for ImeHandler {
    fn clone(&self) -> Self {
        // 克隆时不包含回调函数
        ImeHandler {
            state: self.state.clone(),
            config: self.config.clone(),
            candidate_window: self.candidate_window.clone(),
            current_input_method: self.current_input_method.clone(),
            event_callbacks: Vec::new(),
        }
    }
}

impl PartialEq for ImeHandler {
    fn eq(&self, other: &Self) -> bool {
        self.state == other.state
            && self.config == other.config
            && self.candidate_window == other.candidate_window
            && self.current_input_method == other.current_input_method
    }
}

impl ImeHandler {
    /// 创建一个新的 IME 处理器
    pub fn new() -> Self {
        ImeHandler {
            state: ImeState::new(),
            config: ImeConfig::default(),
            candidate_window: CandidateWindow::new(),
            current_input_method: String::new(),
            event_callbacks: Vec::new(),
        }
    }

    /// 启用/禁用 IME
    pub fn set_enabled(&mut self, enabled: bool) {
        self.config.enabled = enabled;
    }

    /// 检查 IME 是否启用
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    /// 开始组合
    pub fn composition_start(&mut self, position: usize) -> Option<ImeEvent> {
        if !self.config.enabled {
            return None;
        }

        self.state.start_composition(position);
        self.candidate_window.clear();
        self.notify_event(ImeEvent::CompositionStart);
        Some(ImeEvent::CompositionStart)
    }

    /// 更新组合文本
    pub fn composition_update(&mut self, text: String) -> Option<ImePreEdit> {
        if !self.config.enabled || !self.state.is_composing {
            return None;
        }

        let cursor_pos = text.len();
        self.state.update_composition(text.clone(), cursor_pos);

        // 生成预编辑信息
        let pre_edit = ImePreEdit::new(text, cursor_pos);

        self.notify_event(ImeEvent::CompositionUpdate(pre_edit.text.clone()));

        Some(pre_edit)
    }

    /// 结束组合并提交
    pub fn composition_end(&mut self, committed_text: String) -> Option<ImeCommit> {
        if !self.config.enabled {
            return None;
        }

        let position = self.state.composing_range.0;
        self.state.reset();

        self.notify_event(ImeEvent::CompositionEnd(committed_text.clone()));

        if !committed_text.is_empty() {
            Some(ImeCommit::new(committed_text, position))
        } else {
            None
        }
    }

    /// 取消组合
    pub fn composition_cancel(&mut self) {
        self.state.reset();
        self.candidate_window.clear();
    }

    /// 直接提交文本（未经过组合）
    pub fn commit(&mut self, text: String, position: usize) -> Option<ImeCommit> {
        if !self.config.enabled || text.is_empty() {
            return None;
        }

        self.notify_event(ImeEvent::Commit(text.clone()));
        Some(ImeCommit::new(text, position))
    }

    /// 切换输入法
    pub fn change_input_method(&mut self, method_name: String) {
        self.current_input_method = method_name;
        self.notify_event(ImeEvent::InputMethodChanged(self.current_input_method.clone()));
    }

    /// 获取当前输入法名称
    pub fn current_input_method(&self) -> &str {
        &self.current_input_method
    }

    /// 更新选区
    pub fn update_selection(&mut self, start: usize, end: usize) {
        self.state.selection_range = (start, end);
    }

    /// 设置候选词列表
    pub fn set_candidates(&mut self, candidates: Vec<CandidateWord>) {
        self.candidate_window.set_candidates(candidates);
    }

    /// 添加候选词
    pub fn add_candidate(&mut self, candidate: CandidateWord) {
        let idx = self.candidate_window.total_count();
        let mut new_candidate = candidate;
        new_candidate.index = idx;
        self.candidate_window.candidates.push(new_candidate);
        self.candidate_window.total_pages =
            (self.candidate_window.candidates.len() + self.candidate_window.config.page_size - 1)
                / self.candidate_window.config.page_size;
    }

    /// 选择候选词
    pub fn select_candidate(&mut self, index: usize) -> bool {
        self.candidate_window.select_by_index(index)
    }

    /// 获取当前选中的候选词
    pub fn get_selected_candidate(&self) -> Option<&CandidateWord> {
        self.candidate_window.get_selected_candidate()
    }

    /// 候选词导航
    pub fn navigate_candidates(&mut self, direction: CandidateNavigation) -> bool {
        match direction {
            CandidateNavigation::Next => self.candidate_window.select_next(),
            CandidateNavigation::Prev => self.candidate_window.select_prev(),
            CandidateNavigation::NextPage => self.candidate_window.next_page(),
            CandidateNavigation::PrevPage => self.candidate_window.prev_page(),
        }
    }

    /// 清空候选词
    pub fn clear_candidates(&mut self) {
        self.candidate_window.clear();
    }

    /// 检查是否正在组合
    pub fn is_composing(&self) -> bool {
        self.state.is_composing
    }

    /// 获取组合文本
    pub fn composing_text(&self) -> &str {
        &self.state.composing_text
    }

    /// 获取组合文本范围
    pub fn composing_range(&self) -> (usize, usize) {
        self.state.composing_range
    }

    /// 注册事件回调
    pub fn on_event<F>(&mut self, callback: F)
    where
        F: Fn(ImeEvent) + Send + 'static,
    {
        self.event_callbacks.push(Box::new(callback));
    }

    /// 触发所有回调
    fn notify_event(&self, event: ImeEvent) {
        for callback in &self.event_callbacks {
            callback(event.clone());
        }
    }
}

/// 候选词导航方向
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CandidateNavigation {
    /// 下一个
    Next,
    /// 上一个
    Prev,
    /// 下一页
    NextPage,
    /// 上一页
    PrevPage,
}

/// 用于与 PieceTree 集成的 IME 适配器
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ImeAdapter {
    /// IME 处理器
    pub handler: ImeHandler,
}

impl ImeAdapter {
    /// 创建一个新的 IME 适配器
    pub fn new() -> Self {
        ImeAdapter {
            handler: ImeHandler::new(),
        }
    }

    /// 在指定位置插入组合文本
    ///
    /// 返回插入后的新位置
    pub fn insert_composing_text_at(
        &self,
        position: usize,
        text: &str,
    ) -> usize {
        position + text.len()
    }

    /// 获取组合文本的边界矩形信息
    ///
    /// 返回 (left, top, right, bottom) 相对于文档的位置
    pub fn get_composition_bounds(&self, position: usize) -> Option<(f64, f64, f64, f64)> {
        if !self.handler.is_composing() {
            return None;
        }

        let length = self.handler.composing_text().len();
        if length == 0 {
            return None;
        }

        // 返回大致位置，实际位置需要由前端根据字体计算
        Some((position as f64, 0.0, (position + length) as f64, 0.0))
    }

    /// 获取光标位置（相对于组合文本开头）
    pub fn get_cursor_offset(&self) -> usize {
        if let Some((start, end)) = self.get_selection_range() {
            if self.handler.is_composing() {
                return start.saturating_sub(self.handler.composing_range().0);
            }
        }
        0
    }

    /// 获取选区范围
    pub fn get_selection_range(&self) -> Option<(usize, usize)> {
        let (start, end) = self.handler.state.selection_range;
        if start != end {
            Some((start, end))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ime_state_new() {
        let state = ImeState::new();
        assert!(!state.is_composing);
        assert!(state.composing_text.is_empty());
        assert_eq!(state.composing_range, (0, 0));
    }

    #[test]
    fn test_ime_state_composition() {
        let mut state = ImeState::new();

        state.start_composition(5);
        assert!(state.is_composing);
        assert_eq!(state.composing_range, (5, 5));

        state.update_composition("你好".to_string(), 2);
        assert_eq!(state.composing_text, "你好");
        assert_eq!(state.composing_range, (5, 7));

        state.reset();
        assert!(!state.is_composing);
    }

    #[test]
    fn test_ime_commit() {
        let commit = ImeCommit::new("测试".to_string(), 10);
        assert_eq!(commit.text, "测试");
        assert_eq!(commit.position, 10);
        assert_eq!(commit.committed_length, 2);
    }

    #[test]
    fn test_ime_pre_edit() {
        let pre_edit = ImePreEdit::new("预编辑".to_string(), 2);
        assert_eq!(pre_edit.text, "预编辑");
        assert_eq!(pre_edit.cursor_position, 2);
        assert!(pre_edit.selection.is_none());

        let pre_edit_with_selection =
            ImePreEdit::with_selection("选择文本".to_string(), 2, (0, 2));
        assert_eq!(pre_edit_with_selection.selection, Some((0, 2)));
    }

    #[test]
    fn test_ime_event_display() {
        let event = ImeEvent::CompositionStart;
        assert_eq!(format!("{}", event), "CompositionStart");

        let event = ImeEvent::CompositionUpdate("测试".to_string());
        assert_eq!(format!("{}", event), "CompositionUpdate(测试)");

        let event = ImeEvent::Commit("提交".to_string());
        assert_eq!(format!("{}", event), "Commit(提交)");
    }

    #[test]
    fn test_candidate_window() {
        let mut window = CandidateWindow::new();

        // 添加候选词
        let candidates = vec![
            CandidateWord::new("你好".to_string(), 0),
            CandidateWord::new("您好".to_string(), 1),
            CandidateWord::new("哈喽".to_string(), 2),
        ];
        window.set_candidates(candidates);

        assert_eq!(window.total_count(), 3);
        assert_eq!(window.total_pages(), 1);

        // 选择候选词
        assert!(window.select_next());
        assert_eq!(window.local_selected_index(), Some(0));

        // 翻页测试（多页情况）
        let mut window2 = CandidateWindow::with_config(CandidateWindowConfig {
            page_size: 2,
            max_pages: 10,
            max_word_length: 32,
            show_page_number: true,
            enable_realtime_search: true,
        });

        let candidates = (0..5)
            .map(|i| CandidateWord::new(format!("词{}", i), i))
            .collect();
        window2.set_candidates(candidates);

        assert_eq!(window2.total_pages(), 3);
        assert_eq!(window2.get_current_page_candidates().len(), 2);
        assert_eq!(window2.current_page_number(), 1);

        window2.next_page();
        assert_eq!(window2.current_page_number(), 2);

        window2.prev_page();
        assert_eq!(window2.current_page_number(), 1);
    }

    #[test]
    fn test_ime_handler() {
        let mut handler = ImeHandler::new();

        // 开始组合
        let event = handler.composition_start(0);
        assert_eq!(event, Some(ImeEvent::CompositionStart));
        assert!(handler.is_composing());

        // 更新组合
        let pre_edit = handler.composition_update("你好".to_string());
        assert!(pre_edit.is_some());
        assert_eq!(pre_edit.unwrap().text, "你好");

        // 结束组合并提交
        let commit = handler.composition_end("你好".to_string());
        assert!(commit.is_some());
        assert_eq!(commit.unwrap().text, "你好");
        assert!(!handler.is_composing());
    }

    #[test]
    fn test_ime_handler_candidates() {
        let mut handler = ImeHandler::new();

        // 设置候选词
        let candidates = vec![
            CandidateWord::new("选项1".to_string(), 0),
            CandidateWord::new("选项2".to_string(), 1),
        ];
        handler.set_candidates(candidates);

        assert!(!handler.candidate_window.is_empty());
        assert_eq!(handler.candidate_window.total_count(), 2);

        // 导航
        assert!(handler.navigate_candidates(CandidateNavigation::Next));
        assert!(handler.navigate_candidates(CandidateNavigation::NextPage));
        assert!(handler.navigate_candidates(CandidateNavigation::Prev));
        assert!(handler.navigate_candidates(CandidateNavigation::PrevPage));

        // 选择
        assert!(handler.select_candidate(0));
        assert_eq!(handler.get_selected_candidate().unwrap().text, "选项1");
    }

    #[test]
    fn test_ime_handler_direct_commit() {
        let mut handler = ImeHandler::new();

        let commit = handler.commit("直接提交".to_string(), 5);
        assert!(commit.is_some());
        assert_eq!(commit.unwrap().text, "直接提交");
    }

    #[test]
    fn test_ime_handler_input_method() {
        let mut handler = ImeHandler::new();

        handler.change_input_method("Sogou Pinyin".to_string());
        assert_eq!(handler.current_input_method(), "Sogou Pinyin");

        handler.change_input_method("Microsoft Pinyin".to_string());
        assert_eq!(handler.current_input_method(), "Microsoft Pinyin");
    }

    #[test]
    fn test_ime_handler_disabled() {
        let mut handler = ImeHandler::new();
        handler.set_enabled(false);

        let event = handler.composition_start(0);
        assert!(event.is_none());
        assert!(!handler.is_composing());
    }

    #[test]
    fn test_candidate_word_types() {
        let normal = CandidateWord::new("普通词".to_string(), 0);
        assert_eq!(normal.word_type, CandidateWordType::Normal);

        let frequent = CandidateWord::with_type(
            "高频词".to_string(),
            0,
            CandidateWordType::Frequent,
        );
        assert_eq!(frequent.word_type, CandidateWordType::Frequent);

        let emoji = CandidateWord::with_type("表情".to_string(), 0, CandidateWordType::Emoji);
        assert_eq!(emoji.word_type, CandidateWordType::Emoji);
    }

    #[test]
    fn test_ime_config() {
        let config = ImeConfig::default();
        assert!(config.enabled);
        assert!(config.auto_commit);
        assert!(config.show_candidate_window);
    }

    #[test]
    fn test_underline_style() {
        assert_eq!(UnderlineStyle::Solid as u8, 0);
        assert_eq!(UnderlineStyle::Dashed as u8, 1);
        assert_eq!(UnderlineStyle::Dotted as u8, 2);
        assert_eq!(UnderlineStyle::Wavy as u8, 3);
        assert_eq!(UnderlineStyle::None as u8, 4);
    }
}
