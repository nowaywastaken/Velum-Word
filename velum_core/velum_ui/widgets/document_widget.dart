// 文档渲染 Widget
// 主文档编辑和显示 Widget

import 'package:flutter/gestures.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import '../view_models/document_view_model.dart';
import '../view_models/selection_view_model.dart';
import '../view_models/layout_view_model.dart';
import 'selection_overlay.dart';
import 'cursor_widget.dart';
import 'ime_overlay.dart';

/// 文档编辑配置
class DocumentEditorConfig {
  final bool enableKeyboard;
  final bool enableScroll;
  final bool enableSelection;
  final bool enableIME;
  final Color? backgroundColor;
  final EdgeInsets padding;
  final double cursorBlinkDuration;
  final Duration cursorBlinkInterval;

  const DocumentEditorConfig({
    this.enableKeyboard = true,
    this.enableScroll = true,
    this.enableSelection = true,
    this.enableIME = true,
    this.backgroundColor,
    this.padding = EdgeInsets.zero,
    this.cursorBlinkDuration = const Duration(milliseconds: 700),
    this.cursorBlinkInterval = const Duration(milliseconds: 500),
  });
}

/// 文档 Widget
class DocumentWidget extends StatefulWidget {
  final DocumentViewModel viewModel;
  final SelectionViewModel selectionViewModel;
  final LayoutViewModel layoutViewModel;
  final DocumentEditorConfig config;
  final Widget? header;
  final Widget? footer;
  final List<Widget>? sideWidgets;

  const DocumentWidget({
    super.key,
    required this.viewModel,
    required this.selectionViewModel,
    required this.layoutViewModel,
    this.config = const DocumentEditorConfig(),
    this.header,
    this.footer,
    this.sideWidgets,
  });

  @override
  State<DocumentWidget> createState() => _DocumentWidgetState();
}

class _DocumentWidgetState extends State<DocumentWidget>
    with SingleTickerProviderStateMixin {
  // 焦点节点
  final FocusNode _focusNode = FocusNode();

  // 滚动控制器
  final ScrollController _scrollController = ScrollController();

  // 文本控制器
  late TextEditingController _textController;

  // 动画控制器（用于光标闪烁）
  late AnimationController _cursorController;

  // 选中文本跨度识别器
  late TapGestureRecognizer _doubleTapRecognizer;
  late PanGestureRecognizer _dragSelectionRecognizer;

  // 内容渲染
  final GlobalKey _documentKey = GlobalKey();

  // 最后一次触摸位置
  Offset? _lastTapPosition;

  @override
  void initState() {
    super.initState();

    // 初始化文本控制器
    _textController = TextEditingController(text: widget.viewModel.content);

    // 初始化光标动画
    _cursorController = AnimationController(
      vsync: this,
      duration: widget.config.cursorBlinkDuration,
    )..repeat(
        period: widget.config.cursorBlinkInterval,
        min: 0.5,
        max: 1.0,
      );

    // 双击选择单词
    _doubleTapRecognizer = TapGestureRecognizer()
      ..onDoubleTap = _handleDoubleTap;

    // 拖拽选择
    _dragSelectionRecognizer = PanGestureRecognizer()
      ..onPanStart = _handleDragStart
      ..onPanUpdate = _handleDragUpdate
      ..onPanEnd = _handleDragEnd;

    // 监听焦点变化
    _focusNode.addListener(_onFocusChange);

    // 监听视图模型变化
    widget.viewModel.addListener(_onViewModelChanged);
  }

  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    _textController.text = widget.viewModel.content;
  }

  @override
  void dispose() {
    _focusNode.removeListener(_onFocusChange);
    _focusNode.dispose();
    _scrollController.dispose();
    _textController.dispose();
    _cursorController.dispose();
    _doubleTapRecognizer.dispose();
    _dragSelectionRecognizer.dispose();
    widget.viewModel.removeListener(_onViewModelChanged);
    super.dispose();
  }

  // ==================== 事件处理 ====================

  void _onFocusChange() {
    if (_focusNode.hasFocus) {
      _cursorController.forward();
    } else {
      _cursorController.stop();
    }
  }

  void _onViewModelChanged() {
    if (_textController.text != widget.viewModel.content) {
      // 保存当前选择位置
      final oldSelection = _textController.selection;

      // 更新文本
      _textController.value = TextEditingValue(
        text: widget.viewModel.content,
        selection: oldSelection,
      );
    }
  }

  // ==================== 手势处理 ====================

  void _handleTapDown(TapDownDetails details) {
    _lastTapPosition = details.globalPosition;
  }

  void _handleTapUp(TapUpDetails details) {
    final position = details.globalPosition;
    final offset = _getOffsetFromPosition(position);

    if (offset != null) {
      widget.selectionViewModel.moveSelectionTo(offset);
      widget.viewModel.updateCursorPosition(offset);
    }
  }

  void _handleDoubleTap() {
    if (_lastTapPosition == null) return;

    final offset = _getOffsetFromPosition(_lastTapPosition!);
    if (offset == null) return;

    // 选中单词
    final content = widget.viewModel.content;
    widget.selectionViewModel.selectWord(offset, content);
  }

  void _handleDragStart(DragStartDetails details) {
    final offset = _getOffsetFromPosition(details.globalPosition);
    if (offset != null) {
      widget.selectionViewModel.startDrag(offset);
    }
  }

  void _handleDragUpdate(DragUpdateDetails details) {
    final offset = _getOffsetFromPosition(details.globalPosition);
    if (offset != null) {
      widget.selectionViewModel.updateDrag(offset);
    }
  }

  void _handleDragEnd(DragEndDetails details) {
    widget.selectionViewModel.endDrag();
  }

  // ==================== 键盘处理 ====================

  void _handleKeyEvent(KeyEvent event) {
    if (!widget.config.enableKeyboard) return;

    if (event is KeyDownEvent || event is KeyRepeatEvent) {
      final selection = widget.selectionViewModel;

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
          _moveCursor(selection.anchor - 1, extend: event.isShiftPressed);
          break;

        case LogicalKeyboardKey.arrowRight:
          _moveCursor(selection.anchor + 1, extend: event.isShiftPressed);
          break;

        case LogicalKeyboardKey.arrowUp:
          _moveToPreviousLine(extend: event.isShiftPressed);
          break;

        case LogicalKeyboardKey.arrowDown:
          _moveToNextLine(extend: event.isShiftPressed);
          break;

        case LogicalKeyboardKey.home:
          _moveToLineStart(extend: event.isShiftPressed);
          break;

        case LogicalKeyboardKey.end:
          _moveToLineEnd(extend: event.isShiftPressed);
          break;

        case LogicalKeyboardKey.keyA:
          if (event.isControlPressed || event.isMetaPressed) {
            widget.selectionViewModel.selectAll();
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
              widget.viewModel.redo();
            } else {
              widget.viewModel.undo();
            }
          }
          break;

        case LogicalKeyboardKey.keyY:
          if (event.isControlPressed || event.isMetaPressed) {
            widget.viewModel.redo();
          }
          break;

        default:
          // 输入普通字符
          if (event.character != null && !event.isControlPressed) {
            _insertCharacter(event.character!);
          }
          break;
      }
    }
  }

  // ==================== 文本操作 ====================

  void _handleBackspace() {
    final selection = widget.selectionViewModel.selection;
    if (selection != null && !selection.isEmpty) {
      // 删除选中文本
      widget.viewModel.deleteText(selection.start, selection.length);
    } else if (selection != null && selection.start > 0) {
      // 删除光标前一个字符
      widget.viewModel.deleteText(selection.start - 1, 1);
    }
  }

  void _handleDelete() {
    final selection = widget.selectionViewModel.selection;
    if (selection != null && !selection.isEmpty) {
      widget.viewModel.deleteText(selection.start, selection.length);
    } else if (selection != null && selection.start < widget.viewModel.content.length) {
      widget.viewModel.deleteText(selection.start, 1);
    }
  }

  void _handleEnter() {
    final selection = widget.selectionViewModel.selection;
    final offset = selection?.start ?? widget.viewModel.cursorOffset;
    widget.viewModel.insertText(offset, '\n');
  }

  void _handleTab() {
    final selection = widget.selectionViewModel.selection;
    final offset = selection?.start ?? widget.viewModel.cursorOffset;
    widget.viewModel.insertText(offset, '\t');
  }

  void _insertCharacter(String char) {
    final selection = widget.selectionViewModel.selection;
    final offset = selection?.start ?? widget.viewModel.cursorOffset;

    // 如果有选中文本，先删除
    if (selection != null && !selection.isEmpty) {
      widget.viewModel.deleteText(selection.start, selection.length);
    }

    widget.viewModel.insertText(offset, char);
  }

  void _moveCursor(int offset, {bool extend = false}) {
    final clampedOffset = offset.clamp(0, widget.viewModel.content.length);
    widget.selectionViewModel.moveSelectionTo(clampedOffset, extend: extend);
    widget.viewModel.updateCursorPosition(clampedOffset);
    _scrollToCursor();
  }

  void _moveToPreviousLine({bool extend = false}) {
    // TODO: 实现向上移动一行
    _moveCursor(widget.selectionViewModel.anchor, extend: extend);
  }

  void _moveToNextLine({bool extend = false}) {
    // TODO: 实现向下移动一行
    _moveCursor(widget.selectionViewModel.anchor, extend: extend);
  }

  void _moveToLineStart({bool extend = false}) {
    // TODO: 实现移动到行首
    _moveCursor(widget.selectionViewModel.anchor, extend: extend);
  }

  void _moveToLineEnd({bool extend = false}) {
    // TODO: 实现移动到行尾
    _moveCursor(widget.selectionViewModel.anchor, extend: extend);
  }

  // ==================== 剪贴板操作 ====================

  void _copySelection() {
    final text = widget.viewModel.getSelectedText();
    Clipboard.setData(ClipboardData(text: text));
  }

  void _cutSelection() {
    _copySelection();
    widget.viewModel.deleteText(
      widget.selectionViewModel.selection!.start,
      widget.selectionViewModel.selection!.length,
    );
  }

  Future<void> _pasteSelection() async {
    final data = await Clipboard.getData(Clipboard.kTextPlain);
    if (data?.text != null) {
      widget.viewModel.replaceSelection(data!.text!);
    }
  }

  // ==================== 滚动 ====================

  void _scrollToCursor() {
    final offset = widget.layoutViewModel.getScrollOffsetForOffset(
      widget.viewModel.cursorOffset,
    );

    _scrollController.animateTo(
      offset,
      duration: const Duration(milliseconds: 100),
      curve: Curves.easeOut,
    );
  }

  // ==================== 位置转换 ====================

  int? _getOffsetFromPosition(Offset globalPosition) {
    // 转换全局位置到偏移量
    return widget.layoutViewModel.getOffsetForPosition(
      _localToDocumentPosition(globalPosition),
    );
  }

  Offset _localToDocumentPosition(Offset local) {
    final renderBox = _documentKey.currentContext?.findRenderObject() as RenderBox?;
    if (renderBox == null) return local;

    return local - renderBox.localToGlobal(Offset.zero);
  }

  // ==================== 构建 ====================

  @override
  Widget build(BuildContext context) {
    return RawKeyboardListener(
      focusNode: _focusNode,
      onKeyEvent: _handleKeyEvent,
      child: Stack(
        children: [
          // 主内容区域
          _buildDocumentArea(),

          // 选择叠加层
          if (widget.config.enableSelection)
            SelectionOverlay(
              viewModel: widget.selectionViewModel,
              layoutViewModel: widget.layoutViewModel,
            ),

          // 光标 Widget
          if (widget.config.enableKeyboard)
            CursorWidget(
              viewModel: widget.selectionViewModel,
              layoutViewModel: widget.layoutViewModel,
              animation: _cursorController,
            ),

          // IME 叠加层
          if (widget.config.enableIME)
            ImeOverlay(
              viewModel: widget.selectionViewModel,
              layoutViewModel: widget.layoutViewModel,
            ),
        ],
      ),
    );
  }

  Widget _buildDocumentArea() {
    return GestureDetector(
      onTapUp: _handleTapUp,
      onTapDown: (_) => _focusNode.requestFocus(),
      onDoubleTap: _doubleTapRecognizer.onDoubleTap,
      child: MouseRegion(
        cursor: SystemMouseCursors.text,
        child: Listener(
          pointer: PointerDeviceKind.mouse,
          child: HorizontalDragGestureRecognizer(
            onHorizontalDragStart: _handleDragStart,
            onHorizontalDragUpdate: _handleDragUpdate,
            onHorizontalDragEnd: _handleDragEnd,
            child: Scrollable(
              controller: _scrollController,
              viewportBuilder: (context, viewportOffset) {
                return Viewport(
                  offset: viewportOffset,
                  center: _documentKey,
                  children: [
                    SizedBox(
                      width: widget.layoutViewModel.config.width,
                      height: widget.layoutViewModel.documentHeight,
                      child: _buildDocumentContent(),
                    ),
                  ],
                );
              },
            ),
          ),
        ),
      ),
    );
  }

  Widget _buildDocumentContent() {
    return CustomPaint(
      key: _documentKey,
      size: Size(
        widget.layoutViewModel.config.width,
        widget.layoutViewModel.documentHeight,
      ),
      painter: DocumentPainter(
        viewModel: widget.viewModel,
        layoutViewModel: widget.layoutViewModel,
      ),
    );
  }
}

/// 文档绘制器
class DocumentPainter extends CustomPainter {
  final DocumentViewModel viewModel;
  final LayoutViewModel layoutViewModel;

  DocumentPainter({
    required this.viewModel,
    required this.layoutViewModel,
  });

  @override
  void paint(Canvas canvas, Size size) {
    final lines = layoutViewModel.lines;

    for (final line in lines) {
      for (final span in line.spans) {
        _drawTextSpan(canvas, span, line.y);
      }
    }
  }

  void _drawTextSpan(Canvas canvas, RenderedSpan span, double baseline) {
    final textPainter = TextPainter(
      text: TextSpan(
        text: span.text,
        style: TextStyle(
          fontSize: span.style.fontSize ?? 12,
          fontFamily: span.style.fontFamily,
          fontWeight: span.style.bold ? FontWeight.bold : FontWeight.normal,
          fontStyle: span.style.italic ? FontStyle.italic : FontStyle.normal,
          decoration: span.style.underline ? TextDecoration.underline : null,
          color: _parseColor(span.style.foreground),
          backgroundColor: _parseColor(span.style.background),
        ),
      ),
      textDirection: TextDirection.ltr,
    );

    textPainter.layout(minWidth: span.width, maxWidth: span.width);

    canvas.save();
    canvas.translate(span.x, baseline);
    textPainter.paint(canvas, Offset.zero);
    canvas.restore();
  }

  Color? _parseColor(String? colorString) {
    if (colorString == null) return null;
    try {
      return Color(int.parse(colorString.replaceFirst('#', '0xFF')));
    } catch (e) {
      return null;
    }
  }

  @override
  bool shouldRepaint(covariant DocumentPainter oldDelegate) {
    return oldDelegate.viewModel != viewModel ||
        oldDelegate.layoutViewModel != layoutViewModel;
  }
}

/// 水平拖拽手势识别器
class HorizontalDragGestureRecognizer extends GestureRecognizer {
  HorizontalDragGestureRecognizer({
    super.debugOwner,
    super.supportedDevices,
  });

  final _startDetails = <int, DragStartDetails>{};
  final _updateDetails = <int, DragUpdateDetails>{};

  @override
  void addPointer(PointerDownEvent event) {
    _startDetails[event.pointer] = DragStartDetails(
      globalPosition: event.position,
      localPosition: event.localPosition,
      sourceTimeStamp: event.timeStamp,
    );
  }

  @override
  void addPointerPanZoom(PointerPanZoomStartEvent event) {
    // 处理触摸板手势
  }

  void Function(DragStartDetails)? onHorizontalDragStart;
  void Function(DragUpdateDetails)? onHorizontalDragUpdate;
  void Function(DragEndDetails)? onHorizontalDragEnd;

  @override
  String get debugDescription => 'horizontalDrag';

  @override
  void didStopTrackingLastPointer(int pointer) {
    onHorizontalDragEnd?.call(DragEndDetails(
      velocity: Velocity.zero,
      primaryVelocity: 0,
    ));
  }

  @override
  void handleEvent(PointerEvent event) {
    if (event is PointerMoveEvent) {
      final details = DragUpdateDetails(
        sourceTimeStamp: event.timeStamp,
        globalPosition: event.position,
        localPosition: event.localPosition,
        delta: event.delta,
        primaryDelta: event.delta.dx,
      );
      _updateDetails[event.pointer] = details;
      onHorizontalDragUpdate?.call(details);
    } else if (event is PointerUpEvent) {
      _startDetails.remove(event.pointer);
      _updateDetails.remove(event.pointer);
    } else if (event is PointerCancelEvent) {
      _startDetails.remove(event.pointer);
      _updateDetails.remove(event.pointer);
    }
  }
}
