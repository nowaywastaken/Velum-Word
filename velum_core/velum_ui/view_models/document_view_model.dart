// 文档视图模型 - 核心视图模型
// 管理文档内容、状态和业务逻辑

import 'dart:async';
import 'dart:convert';
import 'package:flutter/foundation.dart';
import 'package:rxdart/rxdart.dart';

import '../bindings/velum_core_bindings_generated.dart';
import 'document_service.dart';
import 'layout_view_model.dart';
import 'selection_view_model.dart';

/// 文档元数据模型
class DocumentMetadataModel {
  final String title;
  final String author;
  final int createdAt;
  final int modifiedAt;
  final int wordCount;
  final int charCount;

  DocumentMetadataModel({
    required this.title,
    required this.author,
    required this.createdAt,
    required this.modifiedAt,
    required this.wordCount,
    required this.charCount,
  });

  factory DocumentMetadataModel.fromJson(Map<String, dynamic> json) {
    return DocumentMetadataModel(
      title: json['title'] ?? '',
      author: json['author'] ?? '',
      createdAt: json['created_at'] ?? 0,
      modifiedAt: json['modified_at'] ?? 0,
      wordCount: json['word_count'] ?? 0,
      charCount: json['char_count'] ?? 0,
    );
  }

  Map<String, dynamic> toJson() => {
        'title': title,
        'author': author,
        'created_at': createdAt,
        'modified_at': modifiedAt,
        'word_count': wordCount,
        'char_count': charCount,
      };
}

/// 文本属性模型
class TextAttributesModel {
  final bool? bold;
  final bool? italic;
  final bool? underline;
  final int? fontSize;
  final String? fontFamily;
  final String? foreground;
  final String? background;

  TextAttributesModel({
    this.bold,
    this.italic,
    this.underline,
    this.fontSize,
    this.fontFamily,
    this.foreground,
    this.background,
  });

  factory TextAttributesModel.fromApiString(String apiString) {
    final parts = apiString.split(',');
    if (parts.length != 7) {
      return TextAttributesModel();
    }

    return TextAttributesModel(
      bold: parts[0] == 'true' ? true : parts[0] == 'false' ? false : null,
      italic: parts[1] == 'true' ? true : parts[1] == 'false' ? false : null,
      underline: parts[2] == 'true' ? true : parts[2] == 'false' ? false : null,
      fontSize: int.tryParse(parts[3]),
      fontFamily: parts[4] == 'None' ? null : parts[4],
      foreground: parts[5] == 'None' ? null : parts[5],
      background: parts[6] == 'None' ? null : parts[6],
    );
  }
}

/// 搜索结果模型
class SearchResultModel {
  final int start;
  final int end;
  final String text;
  final int lineNumber;
  final int columnNumber;

  SearchResultModel({
    required this.start,
    required this.end,
    required this.text,
    required this.lineNumber,
    required this.columnNumber,
  });

  factory SearchResultModel.fromJson(Map<String, dynamic> json) {
    return SearchResultModel(
      start: json['start'] ?? 0,
      end: json['end'] ?? 0,
      text: json['text'] ?? '',
      lineNumber: json['line_number'] ?? 0,
      columnNumber: json['column'] ?? 0,
    );
  }
}

/// 文档视图模型
class DocumentViewModel with ChangeNotifier {
  // 依赖服务
  final DocumentService _documentService;
  final SelectionViewModel _selectionViewModel;
  final LayoutViewModel _layoutViewModel;

  // 文档内容流
  final _contentController = BehaviorSubject<String>();
  Stream<String> get contentStream => _contentController.stream;

  // 元数据流
  final _metadataController = BehaviorSubject<DocumentMetadataModel>();
  Stream<DocumentMetadataModel> get metadataStream => _metadataController.stream;

  // 文档状态
  String _content = '';
  DocumentMetadataModel _metadata = DocumentMetadataModel(
    title: 'Untitled Document',
    author: '',
    createdAt: 0,
    modifiedAt: 0,
    wordCount: 0,
    charCount: 0,
  );

  // 修改状态
  bool _isModified = false;
  bool get isModified => _isModified;

  // 撤销/重做状态
  bool _canUndo = false;
  bool _canRedo = false;
  bool get canUndo => _canUndo;
  bool get canRedo => _canRedo;

  // 加载状态
  bool _isLoading = false;
  bool get isLoading => _isLoading;

  // 错误状态
  String? _error;
  String? get error => _error;

  // 当前光标位置
  int _cursorOffset = 0;
  int get cursorOffset => _cursorOffset;

  DocumentViewModel({
    required DocumentService documentService,
    required SelectionViewModel selectionViewModel,
    required LayoutViewModel layoutViewModel,
  })  : _documentService = documentService,
        _selectionViewModel = selectionViewModel,
        _layoutViewModel = layoutViewModel {
    _initialize();
  }

  Future<void> _initialize() async {
    await loadEmptyDocument();
  }

  // ==================== 文档操作 ====================

  /// 加载空文档
  Future<void> loadEmptyDocument() async {
    _isLoading = true;
    _error = null;
    notifyListeners();

    try {
      final content = await _documentService.createEmptyDocument();
      _content = content;
      _contentController.add(content);
      _isModified = false;

      await _refreshMetadata();
      await _refreshUndoRedoState();
    } catch (e) {
      _error = 'Failed to create empty document: $e';
    } finally {
      _isLoading = false;
      notifyListeners();
    }
  }

  /// 加载文档内容
  Future<void> loadContent(String content) async {
    _isLoading = true;
    _error = null;
    notifyListeners();

    try {
      _content = content;
      _contentController.add(content);
      _isModified = false;

      await _refreshMetadata();
      await _refreshUndoRedoState();

      // 触发布局更新
      _layoutViewModel.updateContent(content);
    } catch (e) {
      _error = 'Failed to load content: $e';
    } finally {
      _isLoading = false;
      notifyListeners();
    }
  }

  /// 加载本地文件
  Future<void> loadFromFile(String path) async {
    _isLoading = true;
    _error = null;
    notifyListeners();

    try {
      final content = await _documentService.loadFromFile(path);
      _content = content;
      _contentController.add(content);
      _isModified = false;

      await _refreshMetadata();
      await _refreshUndoRedoState();

      // 触发布局更新
      _layoutViewModel.updateContent(content);
    } catch (e) {
      _error = 'Failed to load file: $e';
    } finally {
      _isLoading = false;
      notifyListeners();
    }
  }

  /// 加载 DOCX 文件
  Future<void> loadDocxFile(String path) async {
    _isLoading = true;
    _error = null;
    notifyListeners();

    try {
      final result = await _documentService.loadOoxmlDocument(path);
      final parsed = jsonDecode(result);
      _content = parsed['text'] ?? '';
      _contentController.add(_content);
      _isModified = false;

      await _refreshMetadata();
      await _refreshUndoRedoState();

      // 触发布局更新
      _layoutViewModel.updateContent(_content);
    } catch (e) {
      _error = 'Failed to load DOCX: $e';
    } finally {
      _isLoading = false;
      notifyListeners();
    }
  }

  /// 保存文档
  Future<bool> saveToFile(String path) async {
    try {
      final result = await _documentService.saveToFile(path);
      if (!result.startsWith('Error')) {
        _isModified = false;
        notifyListeners();
        return true;
      }
      _error = result;
      return false;
    } catch (e) {
      _error = 'Failed to save: $e';
      return false;
    }
  }

  /// 导出为 DOCX
  Future<bool> exportToDocx(String path) async {
    try {
      final result = await _documentService.exportToOoxml(jsonEncode({
        'text': _content,
        'title': _metadata.title,
        'author': _metadata.author,
      }));
      if (!result.startsWith('Error')) {
        // 写入文件
        return true;
      }
      _error = result;
      return false;
    } catch (e) {
      _error = 'Failed to export: $e';
      return false;
    }
  }

  // ==================== 文本编辑 ====================

  /// 在指定位置插入文本
  Future<void> insertText(int offset, String text) async {
    try {
      final newContent = await _documentService.insertText(offset, text);
      _content = newContent;
      _contentController.add(newContent);
      _isModified = true;

      await _refreshUndoRedoState();

      // 更新光标位置
      _cursorOffset = offset + text.length;
      _selectionViewModel.setSelection(_cursorOffset, _cursorOffset);

      // 触发增量布局更新
      _layoutViewModel.updateContent(newContent);
      notifyListeners();
    } catch (e) {
      _error = 'Failed to insert text: $e';
      notifyListeners();
    }
  }

  /// 删除指定范围文本
  Future<void> deleteText(int offset, int length) async {
    try {
      final newContent = await _documentService.deleteText(offset, length);
      _content = newContent;
      _contentController.add(newContent);
      _isModified = true;

      await _refreshUndoRedoState();

      // 更新光标位置
      _cursorOffset = offset;
      _selectionViewModel.setSelection(_cursorOffset, _cursorOffset);

      // 触发布局更新
      _layoutViewModel.updateContent(newContent);
      notifyListeners();
    } catch (e) {
      _error = 'Failed to delete text: $e';
      notifyListeners();
    }
  }

  /// 替换选中文本
  Future<void> replaceSelection(String replacement) async {
    final selection = _selectionViewModel.selection;
    if (selection == null) return;

    final start = selection.start;
    final end = selection.end;

    try {
      // 删除选中内容
      if (start < end) {
        await deleteText(start, end - start);
      }

      // 插入新内容
      await insertText(start, replacement);

      // 清除选择
      _selectionViewModel.clearSelection();
    } catch (e) {
      _error = 'Failed to replace selection: $e';
    }
  }

  // ==================== 撤销/重做 ====================

  /// 撤销操作
  Future<void> undo() async {
    try {
      final newContent = await _documentService.undo();
      _content = newContent;
      _contentController.add(newContent);
      _isModified = true;

      await _refreshUndoRedoState();
      _layoutViewModel.updateContent(newContent);
      notifyListeners();
    } catch (e) {
      _error = 'Failed to undo: $e';
    }
  }

  /// 重做操作
  Future<void> redo() async {
    try {
      final newContent = await _documentService.redo();
      _content = newContent;
      _contentController.add(newContent);
      _isModified = true;

      await _refreshUndoRedoState();
      _layoutViewModel.updateContent(newContent);
      notifyListeners();
    } catch (e) {
      _error = 'Failed to redo: $e';
    }
  }

  // ==================== 选择操作 ====================

  /// 设置选择范围
  void setSelection(int anchor, int active) {
    _selectionViewModel.setSelection(anchor, active);
    notifyListeners();
  }

  /// 清除选择
  void clearSelection() {
    _selectionViewModel.clearSelection();
    notifyListeners();
  }

  /// 获取选中文本
  Future<String> getSelectedText() async {
    return await _documentService.getSelectionText();
  }

  // ==================== 搜索操作 ====================

  /// 搜索文本
  Future<List<SearchResultModel>> search(String query,
      {bool caseSensitive = true, bool wholeWord = false}) async {
    try {
      final options = jsonEncode({
        'query': query,
        'case_sensitive': caseSensitive,
        'whole_word': wholeWord,
      });
      final result = await _documentService.findWithOptions(options);
      final decoded = jsonDecode(result) as Map<String, dynamic>;

      if (decoded['results'] != null) {
        final results = (decoded['results'] as List)
            .map((r) => SearchResultModel.fromJson(r))
            .toList();
        return results;
      }
      return [];
    } catch (e) {
      _error = 'Search failed: $e';
      return [];
    }
  }

  /// 查找下一个
  Future<SearchResultModel?> findNext(String query) async {
    try {
      final result = await _documentService.findNext(query);
      final decoded = jsonDecode(result) as Map<String, dynamic>;
      if (decoded.isNotEmpty) {
        return SearchResultModel.fromJson(decoded);
      }
      return null;
    } catch (e) {
      return null;
    }
  }

  /// 替换文本
  Future<int> replace(String find, String replace, {bool replaceAll = false}) async {
    return await _documentService.replaceText(find, replace, replaceAll);
  }

  // ==================== 格式操作 ====================

  /// 获取位置处的文本属性
  Future<TextAttributesModel?> getAttributesAt(int offset) async {
    try {
      final result = await _documentService.getTextAttributesAt(offset);
      return TextAttributesModel.fromApiString(result);
    } catch (e) {
      return null;
    }
  }

  /// 应用文本属性
  Future<void> applyAttributes(int start, int end,
      {bool? bold, bool? italic, bool? underline, int? fontSize}) async {
    final attrs = {
      if (bold != null) 'bold': bold,
      if (italic != null) 'italic': italic,
      if (underline != null) 'underline': underline,
      if (fontSize != null) 'font_size': fontSize,
    };

    try {
      final newContent =
          await _documentService.applyTextAttributes(start, end, jsonEncode(attrs));
      _content = newContent;
      _contentController.add(newContent);
      _isModified = true;
      notifyListeners();
    } catch (e) {
      _error = 'Failed to apply attributes: $e';
    }
  }

  /// 设置文档标题
  void setTitle(String title) {
    _documentService.setDocumentTitle(title);
    _metadata = _metadata.copyWith(title: title);
    _metadataController.add(_metadata);
    notifyListeners();
  }

  // ==================== 辅助方法 ====================

  Future<void> _refreshMetadata() async {
    try {
      final title = await _documentService.getDocumentTitle();
      final author = await _documentService.getDocumentAuthor();
      final wordCount = await _documentService.getWordCount();
      final charCount = await _documentService.getCharCount();

      _metadata = DocumentMetadataModel(
        title: title,
        author: author,
        createdAt: _metadata.createdAt,
        modifiedAt: _metadata.modifiedAt,
        wordCount: wordCount,
        charCount: charCount,
      );
      _metadataController.add(_metadata);
    } catch (e) {
      debugPrint('Failed to refresh metadata: $e');
    }
  }

  Future<void> _refreshUndoRedoState() async {
    _canUndo = await _documentService.canUndo();
    _canRedo = await _documentService.canRedo();
  }

  /// 更新光标位置
  void updateCursorPosition(int offset) {
    _cursorOffset = offset;
    _selectionViewModel.moveSelectionTo(offset);
    notifyListeners();
  }

  /// 获取完整内容
  String get content => _content;

  /// 获取元数据
  DocumentMetadataModel get metadata => _metadata;

  @override
  void dispose() {
    _contentController.close();
    _metadataController.close();
    super.dispose();
  }
}

// 扩展元数据
extension on DocumentMetadataModel {
  DocumentMetadataModel copyWith({
    String? title,
    String? author,
    int? createdAt,
    int? modifiedAt,
    int? wordCount,
    int? charCount,
  }) {
    return DocumentMetadataModel(
      title: title ?? this.title,
      author: author ?? this.author,
      createdAt: createdAt ?? this.createdAt,
      modifiedAt: modifiedAt ?? this.modifiedAt,
      wordCount: wordCount ?? this.wordCount,
      charCount: charCount ?? this.charCount,
    );
  }
}
