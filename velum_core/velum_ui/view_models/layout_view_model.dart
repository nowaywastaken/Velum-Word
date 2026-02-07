// 布局视图模型 - 管理文档布局状态
// 处理分页、行布局和渲染信息

import 'dart:async';
import 'package:flutter/foundation.dart';
import 'package:rxdart/rxdart.dart';

/// 渲染配置
class RenderConfig {
  final double width;
  final double height;
  final double fontSize;
  final String fontFamily;
  final double lineHeight;
  final double letterSpacing;
  final double wordSpacing;

  const RenderConfig({
    this.width = 612.0, // 默认 A4 宽度 (points)
    this.height = 792.0, // 默认 A4 高度 (points)
    this.fontSize = 12.0,
    this.fontFamily = 'Arial',
    this.lineHeight = 1.5,
    this.letterSpacing = 0.0,
    this.wordSpacing = 0.0,
  });

  RenderConfig copyWith({
    double? width,
    double? height,
    double? fontSize,
    String? fontFamily,
    double? lineHeight,
    double? letterSpacing,
    double? wordSpacing,
  }) {
    return RenderConfig(
      width: width ?? this.width,
      height: height ?? this.height,
      fontSize: fontSize ?? this.fontSize,
      fontFamily: fontFamily ?? this.fontFamily,
      lineHeight: lineHeight ?? this.lineHeight,
      letterSpacing: letterSpacing ?? this.letterSpacing,
      wordSpacing: wordSpacing ?? this.wordSpacing,
    );
  }
}

/// 渲染行信息
class RenderedLine {
  final int lineNumber;
  final int startOffset;
  final int endOffset;
  final double x;
  final double y;
  final double width;
  final double height;
  final List<RenderedSpan> spans;

  const RenderedLine({
    required this.lineNumber,
    required this.startOffset,
    required this.endOffset,
    required this.x,
    required this.y,
    required this.width,
    required this.height,
    required this.spans,
  });

  Map<String, dynamic> toJson() => {
        'line_number': lineNumber,
        'start_offset': startOffset,
        'end_offset': endOffset,
        'x': x,
        'y': y,
        'width': width,
        'height': height,
        'spans': spans.map((s) => s.toJson()).toList(),
      };
}

/// 渲染的文本片段
class RenderedSpan {
  final int startOffset;
  final int endOffset;
  final String text;
  final double x;
  final double y;
  final double width;
  final double height;
  final TextStyleInfo style;

  const RenderedSpan({
    required this.startOffset,
    required this.endOffset,
    required this.text,
    required this.x,
    required this.y,
    required this.width,
    required this.height,
    required this.style,
  });

  Map<String, dynamic> toJson() => {
        'start_offset': startOffset,
        'end_offset': endOffset,
        'text': text,
        'x': x,
        'y': y,
        'width': width,
        'height': height,
        'style': style.toJson(),
      };
}

/// 文本样式信息
class TextStyleInfo {
  final bool bold;
  final bool italic;
  final bool underline;
  final double? fontSize;
  final String? fontFamily;
  final String? foreground;
  final String? background;

  const TextStyleInfo({
    this.bold = false,
    this.italic = false,
    this.underline = false,
    this.fontSize,
    this.fontFamily,
    this.foreground,
    this.background,
  });

  factory TextStyleInfo.fromAttributes(Map<String, dynamic> attrs) {
    return TextStyleInfo(
      bold: attrs['bold'] ?? false,
      italic: attrs['italic'] ?? false,
      underline: attrs['underline'] ?? false,
      fontSize: attrs['font_size']?.toDouble(),
      fontFamily: attrs['font_family'],
      foreground: attrs['foreground'],
      background: attrs['background'],
    );
  }

  Map<String, dynamic> toJson() => {
        'bold': bold,
        'italic': italic,
        'underline': underline,
        'font_size': fontSize,
        'font_family': fontFamily,
        'foreground': foreground,
        'background': background,
      };
}

/// 渲染的页面
class RenderedPage {
  final int pageNumber;
  final double x;
  final double y;
  final double width;
  final double height;
  final List<RenderedLine> lines;
  final int startOffset;
  final int endOffset;

  const RenderedPage({
    required this.pageNumber,
    required this.x,
    required this.y,
    required this.width,
    required this.height,
    required this.lines,
    required this.startOffset,
    required this.endOffset,
  });

  bool containsOffset(int offset) => offset >= startOffset && offset < endOffset;

  /// 获取页面中的可见行
  List<RenderedLine> getVisibleLines(double visibleTop, double visibleBottom) {
    return lines
        .where((line) =>
            line.y + line.height > visibleTop && line.y < visibleBottom)
        .toList();
  }
}

/// 布局视图模型
class LayoutViewModel with ChangeNotifier {
  // 依赖服务
  // final LayoutService _layoutService;

  // 布局状态流
  final _linesController = BehaviorSubject<List<RenderedLine>>();
  Stream<List<RenderedLine>> get linesStream => _linesController.stream;

  final _pagesController = BehaviorSubject<List<RenderedPage>>();
  Stream<List<RenderedPage>> get pagesStream => _pagesController.stream;

  final _configController = BehaviorSubject<RenderConfig>.seeded(const RenderConfig());
  Stream<RenderConfig> get configStream => _configController.stream;

  final _visibleRangeController = BehaviorSubject<VisibleRange>();
  Stream<VisibleRange> get visibleRangeStream => _visibleRangeController.stream;

  // 状态
  List<RenderedLine> _lines = [];
  List<RenderedPage> _pages = [];
  RenderConfig _config = const RenderConfig();
  VisibleRange _visibleRange = const VisibleRange.empty();

  // 文档内容
  String _content = '';

  // 可见性信息
  final Map<int, Rect> _characterRects = {};

  // 布局请求
  Completer<void>? _layoutCompleter;

  LayoutViewModel();

  // ==================== 布局操作 ====================

  /// 更新文档内容
  void updateContent(String content) {
    _content = content;
    _requestLayout();
  }

  /// 设置渲染配置
  void setRenderConfig(RenderConfig config) {
    _config = config;
    _configController.add(config);
    _requestLayout();
  }

  /// 请求重新布局
  void _requestLayout() {
    if (_layoutCompleter == null || _layoutCompleter!.isCompleted) {
      _layoutCompleter = Completer<void>();
      _performLayout().then((_) {
        _layoutCompleter = null;
      });
    }
  }

  /// 执行布局计算
  Future<void> _performLayout() async {
    if (_content.isEmpty) {
      _lines = [];
      _pages = [];
      _notifyLayoutChanged();
      return;
    }

    // TODO: 调用 Rust 核心的布局服务
    // final layoutResult = await _layoutService.layoutText(_content, _config.width);
    // _lines = _parseLayoutResult(layoutResult);
    // _pages = _paginateLines();

    // 模拟布局结果
    _simulateLayout();

    _notifyLayoutChanged();
  }

  /// 模拟布局（实际应该调用 Rust 核心）
  void _simulateLayout() {
    final lines = <RenderedLine>[];
    final charsPerLine = (_config.width / _config.fontSize * 0.6).floor();
    final lineHeight = _config.fontSize * _config.lineHeight;

    final linesContent = _content.split('\n');
    var globalOffset = 0;

    var lineNum = 0;
    for (final lineContent in linesContent) {
      var lineText = lineContent;
      var lineStartOffset = globalOffset;

      while (lineText.isNotEmpty) {
        final chunk = lineText.length > charsPerLine
            ? lineText.substring(0, charsPerLine)
            : lineText;

        final chunkEndOffset = lineStartOffset + chunk.length;

        lines.add(RenderedLine(
          lineNumber: lineNum,
          startOffset: lineStartOffset,
          endOffset: chunkEndOffset,
          x: 0,
          y: lineNum * lineHeight,
          width: chunk.length * _config.fontSize * 0.6,
          height: lineHeight,
          spans: [
            RenderedSpan(
              startOffset: lineStartOffset,
              endOffset: chunkEndOffset,
              text: chunk,
              x: 0,
              y: lineNum * lineHeight,
              width: chunk.length * _config.fontSize * 0.6,
              height: lineHeight,
              style: const TextStyleInfo(),
            ),
          ],
        ));

        lineText = lineText.substring(chunk.length);
        lineStartOffset = chunkEndOffset;
        lineNum++;
      }

      globalOffset += lineContent.length + 1; // +1 for newline
    }

    _lines = lines;
    _pages = _paginateLines();
  }

  /// 生成分页
  List<RenderedPage> _paginateLines() {
    final pages = <RenderedPage>[];
    final pageHeight = _config.height;
    var currentPageLines = <RenderedLine>[];
    var currentPageStart = 0;
    var currentY = 0.0;

    for (final line in _lines) {
      if (currentY + line.height > pageHeight && currentPageLines.isNotEmpty) {
        // 新页面
        pages.add(RenderedPage(
          pageNumber: pages.length,
          x: 0,
          y: 0,
          width: _config.width,
          height: pageHeight,
          lines: List.from(currentPageLines),
          startOffset: currentPageStart,
          endOffset: currentPageLines.last.endOffset,
        ));

        currentPageLines = [];
        currentPageStart = line.startOffset;
        currentY = 0;
      }

      currentPageLines.add(line.copyWith(y: currentY));
      currentY += line.height;
    }

    // 最后一页
    if (currentPageLines.isNotEmpty) {
      pages.add(RenderedPage(
        pageNumber: pages.length,
        x: 0,
        y: 0,
        width: _config.width,
        height: pageHeight,
        lines: currentPageLines,
        startOffset: currentPageStart,
        endOffset: currentPageLines.last.endOffset,
      ));
    }

    return pages;
  }

  void _notifyLayoutChanged() {
    _linesController.add(_lines);
    _pagesController.add(_pages);
    notifyListeners();
  }

  // ==================== 可见性操作 ====================

  /// 更新可见范围
  void updateVisibleRange(VisibleRange range) {
    _visibleRange = range;
    _visibleRangeController.add(range);
    notifyListeners();
  }

  /// 获取可见页面
  List<RenderedPage> getVisiblePages(double viewportTop, double viewportBottom,
      {double cacheExtent = 200.0}) {
    final extendedTop = viewportTop - cacheExtent;
    final extendedBottom = viewportBottom + cacheExtent;

    return _pages.where((page) {
      final pageBottom = page.y + page.height;
      return pageBottom > extendedTop && page.y < extendedBottom;
    }).toList();
  }

  /// 获取可见行
  List<RenderedLine> getVisibleLines(double viewportTop, double viewportBottom,
      {double cacheExtent = 100.0}) {
    final extendedTop = viewportTop - cacheExtent;
    final extendedBottom = viewportBottom + cacheExtent;

    return _lines
        .where((line) => line.y + line.height > extendedTop && line.y < extendedBottom)
        .toList();
  }

  // ==================== 位置映射 ====================

  /// 将字符偏移量转换为屏幕坐标
  Offset? getOffsetForOffset(int offset) {
    for (final line in _lines) {
      if (offset >= line.startOffset && offset <= line.endOffset) {
        final charWidth = _config.fontSize * 0.6;
        final charIndex = offset - line.startOffset;
        return Offset(
          line.x + charIndex * charWidth,
          line.y,
        );
      }
    }
    return null;
  }

  /// 将屏幕坐标转换为字符偏移量
  int? getOffsetForPosition(Offset position) {
    for (final line in _lines) {
      if (position.dy >= line.y && position.dy <= line.y + line.height) {
        final charWidth = _config.fontSize * 0.6;
        final charIndex = ((position.dx - line.x) / charWidth).floor();
        return (line.startOffset + charIndex).clamp(line.startOffset, line.endOffset);
      }
    }
    return null;
  }

  /// 获取偏移量所在的行号
  int? getLineNumberForOffset(int offset) {
    for (final line in _lines) {
      if (offset >= line.startOffset && offset <= line.endOffset) {
        return line.lineNumber;
      }
    }
    return null;
  }

  /// 获取行首偏移量
  int? getLineStartOffset(int lineNumber) {
    final line = _lines.firstWhere(
      (l) => l.lineNumber == lineNumber,
      orElse: () => _lines.last,
    );
    return line.startOffset;
  }

  /// 获取行尾偏移量
  int? getLineEndOffset(int lineNumber) {
    final line = _lines.firstWhere(
      (l) => l.lineNumber == lineNumber,
      orElse: () => _lines.last,
    );
    return line.endOffset;
  }

  // ==================== Getters ====================

  List<RenderedLine> get lines => _lines;
  List<RenderedPage> get pages => _pages;
  RenderConfig get config => _config;
  VisibleRange get visibleRange => _visibleRange;

  int get lineCount => _lines.length;
  int get pageCount => _pages.length;

  double get documentHeight =>
      _lines.isNotEmpty ? _lines.last.y + _lines.last.height : 0;

  double get documentWidth => _config.width;

  // ==================== 滚动位置 ====================

  /// 计算滚动到指定偏移量的位置
  double getScrollOffsetForOffset(int offset) {
    for (final line in _lines) {
      if (offset >= line.startOffset && offset <= line.endOffset) {
        return line.y;
      }
    }
    return 0;
  }

  /// 获取第一行的偏移量
  double getFirstVisibleOffset(double viewportHeight) {
    return _lines.isNotEmpty ? _lines.first.y : 0;
  }

  /// 获取最后一行的偏移量
  double getLastVisibleOffset(double viewportHeight) {
    if (_lines.isEmpty) return 0;

    var accumulatedHeight = 0.0;
    for (final line in _lines.reversed) {
      accumulatedHeight += line.height;
      if (accumulatedHeight >= viewportHeight) {
        return line.y;
      }
    }
    return _lines.last.y;
  }

  @override
  void dispose() {
    _linesController.close();
    _pagesController.close();
    _configController.close();
    _visibleRangeController.close();
    super.dispose();
  }
}

/// 可见范围
class VisibleRange {
  final double start;
  final double end;

  const VisibleRange({required this.start, required this.end});

  const VisibleRange.empty() : start = 0, end = 0;

  bool get isEmpty => start == end;

  double get length => end - start;

  bool contains(double position) => position >= start && position < end;

  VisibleRange expand({double? start, double? end}) {
    return VisibleRange(
      start: start ?? this.start,
      end: end ?? this.end,
    );
  }

  VisibleRange translate(double delta) {
    return VisibleRange(start: start + delta, end: end + delta);
  }
}

/// 矩形区域
class Rect {
  final double x;
  final double y;
  final double width;
  final double height;

  const Rect({required this.x, required this.y, required this.width, required this.height});

  bool contains(Offset point) =>
      point.dx >= x && point.dx <= x + width && point.dy >= y && point.dy <= y + height;

  Rect expand({double? left, double? top, double? right, double? bottom}) {
    return Rect(
      x: x - (left ?? 0),
      y: y - (top ?? 0),
      width: width + (left ?? 0) + (right ?? 0),
      height: height + (top ?? 0) + (bottom ?? 0),
    );
  }

  Rect translate(double dx, double dy) {
    return Rect(x: x + dx, y: y + dy, width: width, height: height);
  }
}
