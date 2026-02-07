// 文档操作服务
// 封装所有与 Rust 核心交互的文档操作

import 'dart:async';
import 'dart:io';
import 'dart:typed_data';

import '../bindings/velum_core_bindings_generated.dart';

/// 文档操作服务
class DocumentService {
  // 单例模式
  static final DocumentService _instance = DocumentService._internal();
  factory DocumentService() => _instance;
  DocumentService._internal();

  // Rust 核心 API 实例
  VelumCore? _api;

  /// 初始化服务
  Future<void> initialize() async {
    if (_api == null) {
      _api = await VelumCore.init();
    }
  }

  /// 获取 API 实例
  VelumCore get api {
    if (_api == null) {
      throw StateError('DocumentService not initialized. Call initialize() first.');
    }
    return _api!;
  }

  // ==================== 基础文档操作 ====================

  /// 创建空文档
  Future<String> createEmptyDocument() async {
    return api.createEmptyDocument();
  }

  /// 获取完整文本
  Future<String> getFullText() async {
    return api.getFullText();
  }

  /// 在指定位置插入文本
  Future<String> insertText(int offset, String text) async {
    return api.insertText(offset, text);
  }

  /// 删除指定范围文本
  Future<String> deleteText(int offset, int length) async {
    return api.deleteText(offset, length);
  }

  /// 获取文本范围
  Future<String> getTextRange(int offset, int length) async {
    return api.getTextRange(offset, length);
  }

  /// 获取行数
  Future<int> getLineCount() async {
    return api.getLineCount();
  }

  /// 获取指定行内容
  Future<String?> getLineContent(int lineNumber) async {
    final result = await api.getLineContent(lineNumber);
    return result.isEmpty ? null : result;
  }

  /// 获取指定行的字符偏移量
  Future<int> getOffsetAtLine(int lineNumber) async {
    return api.getOffsetAtLine(lineNumber);
  }

  // ==================== 撤销/重做 ====================

  /// 撤销
  Future<String> undo() async {
    return api.undo();
  }

  /// 重做
  Future<String> redo() async {
    return api.redo();
  }

  /// 检查是否可以撤销
  Future<bool> canUndo() async {
    return api.canUndo();
  }

  /// 检查是否可以重做
  Future<bool> canRedo() async {
    return api.canRedo();
  }

  // ==================== 选择操作 ====================

  /// 获取选择锚点位置
  Future<int> getSelectionAnchor() async {
    return api.getSelectionAnchor();
  }

  /// 获取选择活动位置
  Future<int> getSelectionActive() async {
    return api.getSelectionActive();
  }

  /// 设置选择
  Future<void> setSelection(int anchor, int active) async {
    api.setSelection(anchor, active);
  }

  /// 获取选中文本
  Future<String> getSelectionText() async {
    return api.getSelectionText();
  }

  /// 移动选择到位置
  Future<void> moveSelectionTo(int offset) async {
    api.moveSelectionTo(offset);
  }

  /// 清除选择
  Future<void> clearSelection() async {
    api.clearSelection();
  }

  /// 检查是否有选择
  Future<bool> hasSelection() async {
    return api.hasSelection();
  }

  /// 获取选择范围
  Future<(int, int)> getSelectionRange() async {
    return api.getSelectionRange();
  }

  // ==================== 搜索操作 ====================

  /// 搜索文本
  Future<String> findText(String query, String options) async {
    return api.findText(query, options);
  }

  /// 高级搜索
  Future<String> findWithOptions(String optionsJson) async {
    return api.findWithOptions(optionsJson);
  }

  /// 查找下一个
  Future<String> findNext(String query) async {
    return api.findNext(query);
  }

  /// 查找上一个
  Future<String> findPrevious(String query) async {
    return api.findPrevious(query);
  }

  /// 替换文本
  Future<int> replaceText(String find, String replace, bool all) async {
    return api.replaceText(find, replace, all);
  }

  /// 获取匹配数量
  Future<int> getMatchCount(String query) async {
    return api.getMatchCount(query);
  }

  // ==================== 文件操作 ====================

  /// 保存文档到文件
  Future<String> saveToFile(String path) async {
    return api.saveToFile(path);
  }

  /// 从文件加载文档
  Future<String> loadFromFile(String path) async {
    return api.loadFromFile(path);
  }

  /// 获取文档纯文本
  Future<String> getDocumentAsText() async {
    return api.getDocumentAsText();
  }

  /// 从文本加载文档
  Future<String> loadDocumentFromText(String text) async {
    return api.loadDocumentFromText(text);
  }

  /// 导出为纯文本
  Future<String> exportToTxt(String path) async {
    return api.exportToTxt(path);
  }

  // ==================== OOXML 操作 ====================

  /// 加载 OOXML 文档
  Future<String> loadOoxmlDocument(String path) async {
    return api.loadOoxmlDocument(path);
  }

  /// 从字节加载 OOXML
  Future<String> loadOoxmlFromBytes(Uint8List data) async {
    return api.loadOoxmlFromBytes(data);
  }

  /// 导出为 OOXML
  Future<String> exportToOoxml(String documentJson) async {
    final result = await api.exportToOoxml(documentJson);
    return String.fromCharCodes(result);
  }

  /// 提取 OOXML 文本
  Future<String> extractOoxmlText(String path) async {
    return api.extractOoxmlText(path);
  }

  /// 获取 OOXML 统计信息
  Future<String> getOoxmlStats(String path) async {
    return api.getOoxmlStats(path);
  }

  // ==================== 元数据操作 ====================

  /// 获取文档标题
  Future<String> getDocumentTitle() async {
    return api.getDocumentTitle();
  }

  /// 设置文档标题
  Future<void> setDocumentTitle(String title) async {
    api.setDocumentTitle(title);
  }

  /// 获取文档作者
  Future<String> getDocumentAuthor() async {
    return api.getDocumentAuthor();
  }

  /// 设置文档作者
  Future<void> setDocumentAuthor(String author) async {
    api.setDocumentAuthor(author);
  }

  /// 获取创建时间
  Future<int> getDocumentCreatedAt() async {
    return api.getDocumentCreatedAt();
  }

  /// 获取修改时间
  Future<int> getDocumentModifiedAt() async {
    return api.getDocumentModifiedAt();
  }

  /// 获取字数统计
  Future<int> getWordCount() async {
    return api.getWordCount();
  }

  /// 获取字符统计
  Future<int> getCharCount() async {
    return api.getCharCount();
  }

  // ==================== 格式操作 ====================

  /// 获取位置处的文本属性
  Future<String> getTextAttributesAt(int offset) async {
    return api.getTextAttributesAt(offset);
  }

  /// 应用文本属性
  Future<String> applyTextAttributes(int start, int end, String attributesJson) async {
    return api.applyTextAttributes(start, end, attributesJson);
  }

  /// 移除文本属性
  Future<String> removeTextAttributes(int start, int end) async {
    return api.removeTextAttributes(start, end);
  }

  /// 获取带属性的文本
  Future<String> getTextWithAttributes() async {
    return api.getTextWithAttributes();
  }

  // ==================== 布局操作 ====================

  /// 布局文本
  Future<String> layoutText(String text, double width) async {
    return api.layoutText(text, width);
  }

  /// 计算文本宽度
  Future<double> calculateTextWidth(String text) async {
    return api.calculateTextWidth(text);
  }

  /// 获取指定宽度的行数
  Future<int> getLineCountForWidth(String text, double width) async {
    return api.getLineCountForWidth(text, width);
  }

  /// 获取文本高度
  Future<double> getTextHeight(String text, double width, double lineHeight, double fontSize) async {
    return api.getTextHeight(text, width, lineHeight, fontSize);
  }

  /// 布局当前文档
  Future<String> layoutCurrentDocument(double width) async {
    return api.layoutCurrentDocument(width);
  }

  // ==================== 光标位置 ====================

  /// 获取光标位置
  Future<(int, int)> getCursorPosition(int charOffset) async {
    return api.getCursorPosition(charOffset);
  }

  // ==================== 测试函数 ====================

  /// 测试连接
  Future<String> helloVelum() async {
    return api.helloVelum();
  }

  /// 获取示例文档
  Future<String> getSampleDocument() async {
    return api.getSampleDocument();
  }

  /// 乘法测试
  Future<int> multiply(int a, int b) async {
    return api.multiply(a, b);
  }
}
