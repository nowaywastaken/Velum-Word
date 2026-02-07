// 选择区域叠加层
// 绘制选中文本的视觉高亮

import 'package:flutter/material.dart';

import '../view_models/selection_view_model.dart';
import '../view_models/layout_view_model.dart';

/// 选择叠加层配置
class SelectionOverlayConfig {
  final Color selectionColor;
  final double selectionOpacity;
  final bool showCaret;
  final Color caretColor;
  final double caretWidth;
  final BorderRadius selectionBorderRadius;

  const SelectionOverlayConfig({
    this.selectionColor = Colors.blue,
    this.selectionOpacity = 0.3,
    this.showCaret = true,
    this.caretColor = Colors.blue,
    this.caretWidth = 2.0,
    this.selectionBorderRadius = BorderRadius.zero,
  });
}

/// 选择叠加层 Widget
class SelectionOverlay extends StatefulWidget {
  final SelectionViewModel viewModel;
  final LayoutViewModel layoutViewModel;
  final SelectionOverlayConfig config;

  const SelectionOverlay({
    super.key,
    required this.viewModel,
    required this.layoutViewModel,
    this.config = const SelectionOverlayConfig(),
  });

  @override
  State<SelectionOverlay> createState() => _SelectionOverlayState();
}

class _SelectionOverlayState extends State<SelectionOverlay> {
  @override
  void initState() {
    super.initState();
    widget.viewModel.addListener(_onSelectionChanged);
  }

  @override
  void dispose() {
    widget.viewModel.removeListener(_onSelectionChanged);
    super.dispose();
  }

  void _onSelectionChanged() {
    setState(() {
      // 触发重绘
    });
  }

  @override
  Widget build(BuildContext context) {
    return CustomPaint(
      size: Size(
        widget.layoutViewModel.config.width,
        widget.layoutViewModel.documentHeight,
      ),
      painter: SelectionOverlayPainter(
        viewModel: widget.viewModel,
        layoutViewModel: widget.layoutViewModel,
        config: widget.config,
      ),
    );
  }
}

/// 选择叠加层绘制器
class SelectionOverlayPainter extends CustomPainter {
  final SelectionViewModel viewModel;
  final LayoutViewModel layoutViewModel;
  final SelectionOverlayConfig config;

  SelectionOverlayPainter({
    required this.viewModel,
    required this.layoutViewModel,
    required this.config,
  });

  @override
  void paint(Canvas canvas, Size size) {
    // 绘制选择背景
    _drawSelectionBackground(canvas, size);

    // 绘制光标
    if (config.showCaret) {
      _drawCaret(canvas);
    }
  }

  /// 绘制选择背景
  void _drawSelectionBackground(Canvas canvas, Size size) {
    final selection = viewModel.selection;
    if (selection == null || selection.isEmpty) return;

    // 获取选择的矩形区域
    final selectionRects = _getSelectionRects(selection);
    final paint = Paint()
      ..color = config.selectionColor.withOpacity(config.selectionOpacity)
      ..style = PaintingStyle.fill;

    for (final rect in selectionRects) {
      canvas.drawRRect(
        RRect.fromRectAndRadius(
          Rect.fromLTWH(rect.x, rect.y, rect.width, rect.height),
          config.selectionBorderRadius,
        ),
        paint,
      );
    }
  }

  /// 绘制光标
  void _drawCaret(Canvas canvas) {
    final selection = viewModel.selection;
    final position = viewModel.active;

    // 获取光标位置
    final offset = layoutViewModel.getOffsetForOffset(position);
    if (offset == null) return;

    final lineNumber = layoutViewModel.getLineNumberForOffset(position);
    if (lineNumber == null) return;

    final line = layoutViewModel.lines
        .firstWhere((l) => l.lineNumber == lineNumber, orElse: () => layoutViewModel.lines.first);

    final cursorX = offset.dx;
    final cursorY = offset.dy;
    final cursorHeight = line.height;

    final paint = Paint()
      ..color = config.caretColor
      ..style = PaintingStyle.stroke
      ..strokeWidth = config.caretWidth;

    canvas.drawLine(
      Offset(cursorX, cursorY),
      Offset(cursorX, cursorY + cursorHeight),
      paint,
    );
  }

  /// 获取选择的矩形区域列表
  List<Rect> _getSelectionRects(SelectionRange selection) {
    final rects = <Rect>[];

    final startLine = layoutViewModel.getLineNumberForOffset(selection.start);
    final endLine = layoutViewModel.getLineNumberForOffset(selection.end);

    if (startLine == null || endLine == null) return rects;

    for (int lineNum = startLine; lineNum <= endLine; lineNum++) {
      final line = layoutViewModel.lines.firstWhere(
        (l) => l.lineNumber == lineNum,
        orElse: () => layoutViewModel.lines.first,
      );

      int startOffset, endOffset;

      if (lineNum == startLine) {
        startOffset = selection.start;
      } else {
        startOffset = line.startOffset;
      }

      if (lineNum == endLine) {
        endOffset = selection.end;
      } else {
        endOffset = line.endOffset;
      }

      final charWidth = layoutViewModel.config.fontSize * 0.6;
      final startX = (startOffset - line.startOffset) * charWidth + line.x;
      final width = (endOffset - startOffset) * charWidth;

      rects.add(Rect(
        x: startX,
        y: line.y,
        width: width,
        height: line.height,
      ));
    }

    return rects;
  }

  @override
  bool shouldRepaint(covariant SelectionOverlayPainter oldDelegate) {
    return oldDelegate.viewModel != viewModel ||
        oldDelegate.layoutViewModel != layoutViewModel;
  }
}
