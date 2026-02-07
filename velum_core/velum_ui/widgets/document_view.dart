// DocumentView - 基于新 Rust API 的文档视图组件
// 使用 VelumApi 进行文档操作

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'dart:async';
import '../bindings/velum_core_bindings.dart';

/// 文档视图配置
class DocumentViewConfig {
  final bool enableKeyboard;
  final bool enableSelection;
  final bool enableIME;
  final bool enablePageView;
  final Color? backgroundColor;
  final EdgeInsets padding;
  final double cursorBlinkDuration;
  final Duration cursorBlinkInterval;

  const DocumentViewConfig({
    this.enableKeyboard = true,
    this.enableSelection = true,
    this.enableIME = true,
    this.enablePageView = true,
    this.backgroundColor,
    this.padding = const EdgeInsets.all(16),
    this.cursorBlinkDuration = const Duration(milliseconds: 700),
    this.cursorBlinkInterval = const Duration(milliseconds: 500),
  });
}

/// 文档视图状态
class DocumentViewState {
  String content;
  int cursorOffset;
  int selectionStart;
  int selectionEnd;
  bool hasSelection;
  int pageCount;
  int currentPage;
  Map<String, dynamic> metadata;

  DocumentViewState({
    this.content = '',
    this.cursorOffset = 0,
    this.selectionStart = 0,
    this.selectionEnd = 0,
    this.hasSelection = false,
    this.pageCount = 1,
    this.currentPage = 0,
    this.metadata = const {},
  });

  DocumentViewState copyWith({
    String? content,
    int? cursorOffset,
    int? selectionStart,
    int? selectionEnd,
    bool? hasSelection,
    int? pageCount,
    int? currentPage,
    Map<String, dynamic>? metadata,
  }) {
    return DocumentViewState(
      content: content ?? this.content,
      cursorOffset: cursorOffset ?? this.cursorOffset,
      selectionStart: selectionStart ?? this.selectionStart,
      selectionEnd: selectionEnd ?? this.selectionEnd,
      hasSelection: hasSelection ?? this.hasSelection,
      pageCount: pageCount ?? this.pageCount,
      currentPage: currentPage ?? this.currentPage,
      metadata: metadata ?? this.metadata,
    );
  }
}

/// 文档视图控制器
class DocumentViewController extends ChangeNotifier {
  DocumentViewState _state = DocumentViewState();
  final VelumApi _api;

  DocumentViewController({VelumApi? api}) : _api = api ?? VelumApi();

  DocumentViewState get state => _state;

  // ==================== 文档操作 ====================

  /// 加载文档
  Future<void> loadDocument(String text) async {
    _state = _state.copyWith(content: text);
    await _refreshFromApi();
    notifyListeners();
  }

  /// 从文件加载文档
  Future<void> loadFromFile(String path) async {
    final text = await _api.document.getTextRange(0, 1000000);
    _state = _state.copyWith(content: text);
    notifyListeners();
  }

  /// 插入文本
  void insertText(int offset, String text) {
    _api.document.insert(offset, text);
    _state = _state.copyWith(content: _api.document.getTextRange(0, 1000000));
    notifyListeners();
  }

  /// 删除文本
  void deleteText(int offset, int length) {
    _api.document.delete(offset, length);
    _state = _state.copyWith(content: _api.document.getTextRange(0, 1000000));
    notifyListeners();
  }

  /// 替换文本
  void replaceText(String find, String replace, {bool all = false}) {
    _api.document.replaceText(find, replace, all);
    _state = _state.copyWith(content: _api.document.getTextRange(0, 1000000));
    notifyListeners();
  }

  // ==================== 选择操作 ====================

  /// 移动光标
  void moveCursor(int offset) {
    final clampedOffset = offset.clamp(0, _state.content.length);
    _api.document.setSelection(clampedOffset, clampedOffset);
    _state = _state.copyWith(
      cursorOffset: clampedOffset,
      selectionStart: clampedOffset,
      selectionEnd: clampedOffset,
      hasSelection: false,
    );
    notifyListeners();
  }

  /// 设置选择
  void setSelection(int start, int end) {
    final clampedStart = start.clamp(0, _state.content.length);
    final clampedEnd = end.clamp(0, _state.content.length);
    _api.document.setSelection(clampedStart, clampedEnd);
    _state = _state.copyWith(
      cursorOffset: clampedEnd,
      selectionStart: clampedStart,
      selectionEnd: clampedEnd,
      hasSelection: clampedStart != clampedEnd,
    );
    notifyListeners();
  }

  /// 选择全部
  void selectAll() {
    setSelection(0, _state.content.length);
  }

  /// 清除选择
  void clearSelection() {
    _api.document.clearSelection();
    _state = _state.copyWith(
      cursorOffset: _state.selectionEnd,
      hasSelection: false,
    );
    notifyListeners();
  }

  // ==================== 撤销/重做 ====================

  void undo() {
    _api.document.undo();
    _refreshFromApi();
    notifyListeners();
  }

  void redo() {
    _api.document.redo();
    _refreshFromApi();
    notifyListeners();
  }

  bool canUndo() => _api.document.canUndo();
  bool canRedo() => _api.document.canRedo();

  // ==================== 搜索 ====================

  List<SearchResult> find(String query, {bool caseSensitive = false}) {
    final options = SearchOptions(
      query: query,
      caseSensitive: caseSensitive,
    );
    return _api.document.find(query, options);
  }

  // ==================== 布局信息 ====================

  int get pageCount => _api.layout.pageCount();

  int getPageForOffset(int offset) => _api.layout.getPageForOffset(offset);

  LineInfo getLineInfo(int offset) => _api.layout.getLineInfo(offset);

  // ==================== 内部方法 ====================

  Future<void> _refreshFromApi() async {
    _state = _state.copyWith(
      content: _api.document.getTextRange(0, 1000000),
      pageCount: _api.layout.pageCount(),
    );
  }
}

/// 文档视图 Widget
class DocumentView extends StatefulWidget {
  final DocumentViewController controller;
  final DocumentViewConfig config;

  const DocumentView({
    super.key,
    required this.controller,
    this.config = const DocumentViewConfig(),
  });

  @override
  State<DocumentView> createState() => _DocumentViewState();
}

class _DocumentViewState extends State<DocumentView>
    with SingleTickerProviderStateMixin {
  final FocusNode _focusNode = FocusNode();
  late AnimationController _cursorController;
  late ScrollController _scrollController;

  @override
  void initState() {
    super.initState();
    _cursorController = AnimationController(
      vsync: this,
      duration: widget.config.cursorBlinkDuration,
    )..repeat(
        period: widget.config.cursorBlinkInterval,
        min: 0.5,
        max: 1.0,
      );

    _scrollController = ScrollController();

    _focusNode.addListener(_onFocusChange);
    widget.controller.addListener(_onControllerChanged);
  }

  @override
  void dispose() {
    _focusNode.removeListener(_onFocusChange);
    _focusNode.dispose();
    _scrollController.dispose();
    _cursorController.dispose();
    widget.controller.removeListener(_onControllerChanged);
    super.dispose();
  }

  void _onFocusChange() {
    if (_focusNode.hasFocus) {
      _cursorController.forward();
    } else {
      _cursorController.stop();
    }
  }

  void _onControllerChanged() {
    setState(() {});
  }

  // ==================== 键盘处理 ====================

  void _handleKeyEvent(KeyEvent event) {
    if (!widget.config.enableKeyboard) return;

    if (event is KeyDownEvent || event is KeyRepeatEvent) {
      switch (event.logicalKey) {
        case LogicalKeyboardKey.backspace:
          _handleBackspace();
          break;
        case LogicalKeyboardKey.delete:
          _handleDelete();
          break;
        case LogicalKeyboardKey.enter:
          _handleEnter();
          break;
        case LogicalKeyboardKey.tab:
          _handleTab();
          break;
        case LogicalKeyboardKey.arrowLeft:
          _moveCursor(_controller.cursorOffset - 1, extend: event.isShiftPressed);
          break;
        case LogicalKeyboardKey.arrowRight:
          _moveCursor(_controller.cursorOffset + 1, extend: event.isShiftPressed);
          break;
        case LogicalKeyboardKey.keyA:
          if (event.isControlPressed || event.isMetaPressed) {
            _controller.selectAll();
          }
          break;
        case LogicalKeyboardKey.keyC:
          if (event.isControlPressed || event.isMetaPressed) {
            _copySelection();
          }
          break;
        case LogicalKeyboardKey.keyX:
          if (event.isControlPressed || event.isMetaPressed) {
            _cutSelection();
          }
          break;
        case LogicalKeyboardKey.keyV:
          if (event.isControlPressed || event.isMetaPressed) {
            _pasteSelection();
          }
          break;
        case LogicalKeyboardKey.keyZ:
          if (event.isControlPressed || event.isMetaPressed) {
            if (event.isShiftPressed) {
              _controller.redo();
            } else {
              _controller.undo();
            }
          }
          break;
        default:
          if (event.character != null && !event.isControlPressed) {
            _insertCharacter(event.character!);
          }
          break;
      }
    }
  }

  void _handleBackspace() {
    if (_controller.state.hasSelection) {
      _controller.deleteText(
        _controller.state.selectionStart,
        _controller.state.selectionEnd - _controller.state.selectionStart,
      );
    } else if (_controller.state.cursorOffset > 0) {
      _controller.deleteText(_controller.state.cursorOffset - 1, 1);
    }
  }

  void _handleDelete() {
    if (_controller.state.hasSelection) {
      _controller.deleteText(
        _controller.state.selectionStart,
        _controller.state.selectionEnd - _controller.state.selectionStart,
      );
    } else if (_controller.state.cursorOffset < _controller.state.content.length) {
      _controller.deleteText(_controller.state.cursorOffset, 1);
    }
  }

  void _handleEnter() {
    _controller.insertText(_controller.state.cursorOffset, '\n');
  }

  void _handleTab() {
    _controller.insertText(_controller.state.cursorOffset, '\t');
  }

  void _insertCharacter(String char) {
    if (_controller.state.hasSelection) {
      _controller.deleteText(
        _controller.state.selectionStart,
        _controller.state.selectionEnd - _controller.state.selectionStart,
      );
    }
    _controller.insertText(_controller.state.cursorOffset, char);
  }

  void _moveCursor(int offset, {bool extend = false}) {
    final clampedOffset = offset.clamp(0, _controller.state.content.length);
    if (extend && _controller.state.hasSelection) {
      _controller.setSelection(_controller.state.selectionStart, clampedOffset);
    } else {
      _controller.moveCursor(clampedOffset);
    }
  }

  void _copySelection() {
    if (_controller.state.hasSelection) {
      final text = _controller.state.content.substring(
        _controller.state.selectionStart,
        _controller.state.selectionEnd,
      );
      Clipboard.setData(ClipboardData(text: text));
    }
  }

  void _cutSelection() {
    _copySelection();
    if (_controller.state.hasSelection) {
      _controller.deleteText(
        _controller.state.selectionStart,
        _controller.state.selectionEnd - _controller.state.selectionStart,
      );
    }
  }

  Future<void> _pasteSelection() async {
    final data = await Clipboard.getData(Clipboard.kTextPlain);
    if (data?.text != null) {
      if (_controller.state.hasSelection) {
        _controller.deleteText(
          _controller.state.selectionStart,
          _controller.state.selectionEnd - _controller.state.selectionStart,
        );
      }
      _controller.insertText(_controller.state.cursorOffset, data!.text!);
    }
  }

  DocumentViewController get _controller => widget.controller;

  // ==================== 构建 ====================

  @override
  Widget build(BuildContext context) {
    return RawKeyboardListener(
      focusNode: _focusNode,
      onKeyEvent: _handleKeyEvent,
      child: GestureDetector(
        onTap: () => _focusNode.requestFocus(),
        child: Scrollable(
          controller: _scrollController,
          viewportBuilder: (context, viewportOffset) {
            return Viewport(
              offset: viewportOffset,
              center: UniqueKey(),
              children: [
                SizedBox(
                  width: double.infinity,
                  height: _calculateDocumentHeight(),
                  child: CustomPaint(
                    painter: _DocumentContentPainter(
                      controller: _controller,
                      cursorAnimation: _cursorController,
                    ),
                  ),
                ),
              ],
            );
          },
        ),
      ),
    );
  }

  double _calculateDocumentHeight() {
    // 估算文档高度
    final lines = _controller.state.content.split('\n');
    return lines.length * 24.0 + 100; // 估算每行约24像素
  }
}

/// 文档内容绘制器
class _DocumentContentPainter extends CustomPainter {
  final DocumentViewController controller;
  final Animation<double> cursorAnimation;

  _DocumentContentPainter({
    required this.controller,
    required this.cursorAnimation,
  });

  @override
  void paint(Canvas canvas, Size size) {
    final content = controller.state.content;
    final cursorOffset = controller.state.cursorOffset;
    final selectionStart = controller.state.selectionStart;
    final selectionEnd = controller.state.selectionEnd;

    double y = 20;
    double x = 20;
    const lineHeight = 24.0;
    const charWidth = 10.0;

    int currentOffset = 0;

    for (final line in content.split('\n')) {
      // 绘制选择背景
      if (selectionStart != selectionEnd) {
        final startInLine = selectionStart.clamp(currentOffset, currentOffset + line.length);
        final endInLine = selectionEnd.clamp(currentOffset, currentOffset + line.length);

        if (startInLine < endInLine) {
          final selectionRect = Rect.fromLTWH(
            x + (startInLine - currentOffset) * charWidth,
            y,
            (endInLine - startInLine) * charWidth,
            lineHeight,
          );
          canvas.drawRect(
            selectionRect,
            Paint()..color = Colors.blue.withOpacity(0.3),
          );
        }
      }

      // 绘制文本
      final textPainter = TextPainter(
        text: TextSpan(
          text: line,
          style: const TextStyle(
            fontSize: 14,
            fontFamily: 'SF Pro Display',
            color: Colors.black,
          ),
        ),
        textDirection: TextDirection.ltr,
      );
      textPainter.layout();
      textPainter.paint(canvas, Offset(x, y));

      // 绘制光标
      if (cursorOffset >= currentOffset &&
          cursorOffset <= currentOffset + line.length &&
          !controller.state.hasSelection) {
        final cursorX = x + (cursorOffset - currentOffset) * charWidth;
        final cursorRect = Rect.fromLTWH(cursorX, y, 1.5, lineHeight);
        final cursorAlpha = (0.5 + 0.5 * cursorAnimation.value).clamp(0.0, 1.0);
        canvas.drawRect(
          cursorRect,
          Paint()..color = Colors.black.withOpacity(cursorAlpha),
        );
      }

      y += lineHeight;
      currentOffset += line.length + 1; // +1 for newline
    }
  }

  @override
  bool shouldRepaint(covariant _DocumentContentPainter oldDelegate) {
    return oldDelegate.controller != controller ||
        oldDelegate.cursorAnimation != cursorAnimation;
  }
}
