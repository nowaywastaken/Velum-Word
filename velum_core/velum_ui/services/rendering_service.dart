// 渲染服务
// 管理文档渲染和文本测量

import 'dart:async';
import 'dart:math' as math;

import '../bindings/velum_core_bindings_generated.dart';
import 'document_service.dart';

/// 渲染服务
class RenderingService {
  // 单例模式
  static final RenderingService _instance = RenderingService._internal();
  factory RenderingService() => _instance;
  RenderingService._internal();

  final DocumentService _documentService = DocumentService();

  // 字体度量缓存
  final Map<String, FontMetrics> _fontMetricsCache = {};

  // 文本测量缓存
  final Map<String, double> _textWidthCache = {};

  // 最大缓存大小
  static const int _maxWidthCacheSize = 1000;

  // DPI 缩放因子
  double _dpiScale = 1.0;

  /// 初始化渲染服务
  Future<void> initialize({double dpiScale = 1.0}) async {
    await _documentService.initialize();
    _dpiScale = dpiScale;
  }

  // ==================== 文本测量 ====================

  /// 测量单个字符宽度
  Future<double> measureCharWidth(String char, {String fontFamily = 'Arial', double fontSize = 12.0}) async {
    final key = '$char:$fontFamily:$fontSize';
    if (_textWidthCache.containsKey(key)) {
      return _textWidthCache[key]!;
    }

    // 使用 Rust 核心测量
    final result = await _documentService.calculateTextWidth(char);
    final width = result / _dpiScale;

    _cacheTextWidth(key, width);
    return width;
  }

  /// 测量字符串宽度
  Future<double> measureTextWidth(String text, {String fontFamily = 'Arial', double fontSize = 12.0}) async {
    if (text.isEmpty) return 0.0;

    final key = '$text:$fontFamily:$fontSize';
    if (_textWidthCache.containsKey(key)) {
      return _textWidthCache[key]!;
    }

    // 使用 Rust 核心测量
    final result = await _documentService.calculateTextWidth(text);
    final width = result / _dpiScale;

    _cacheTextWidth(key, width);
    return width;
  }

  /// 测量多行文本
  Future<List<double>> measureTextLines(List<String> lines, {String fontFamily = 'Arial', double fontSize = 12.0}) async {
    final widths = <double>[];
    for (final line in lines) {
      widths.add(await measureTextWidth(line, fontFamily: fontFamily, fontSize: fontSize));
    }
    return widths;
  }

  /// 获取字符在行中的偏移位置
  Future<double> getCharacterPosition(String text, int charIndex, {String fontFamily = 'Arial', double fontSize = 12.0}) async {
    if (charIndex <= 0) return 0.0;
    if (charIndex >= text.length) return await measureTextWidth(text, fontFamily: fontFamily, fontSize: fontSize);

    final substring = text.substring(0, charIndex);
    return await measureTextWidth(substring, fontFamily: fontFamily, fontSize: fontSize);
  }

  // ==================== 布局计算 ====================

  /// 计算行数
  Future<int> calculateLineCount(String text, double width, {String fontFamily = 'Arial', double fontSize = 12.0, double lineHeight = 1.5}) async {
    return await _documentService.getLineCountForWidth(text, width * _dpiScale);
  }

  /// 计算文本高度
  Future<double> calculateTextHeight(String text, double width, {String fontFamily = 'Arial', double fontSize = 12.0, double lineHeight = 1.5}) async {
    return await _documentService.getTextHeight(text, width * _dpiScale, lineHeight, fontSize);
  }

  /// 布局文档
  Future<LayoutResult> layoutDocument(String text, double width, {String fontFamily = 'Arial', double fontSize = 12.0, double lineHeight = 1.5}) async {
    final layoutJson = await _documentService.layoutText(text, width * _dpiScale);
    return LayoutResult.fromJson(layoutJson);
  }

  // ==================== 字体度量 ====================

  /// 获取字体度量
  Future<FontMetrics> getFontMetrics(String fontFamily, double fontSize) async {
    final key = '$fontFamily:$fontSize';
    if (_fontMetricsCache.containsKey(key)) {
      return _fontMetricsCache[key]!;
    }

    // 默认度量（实际应该从系统获取）
    final metrics = FontMetrics(
      ascent: fontSize * 0.8,
      descent: fontSize * 0.2,
      leading: fontSize * 0.1,
      height: fontSize * lineHeightFactor(fontSize),
      avgCharWidth: fontSize * 0.5,
      maxCharWidth: fontSize * 0.8,
    );

    _fontMetricsCache[key] = metrics;
    return metrics;
  }

  double lineHeightFactor(double fontSize) {
    // 根据字体大小调整行高因子
    if (fontSize <= 12) return 1.2;
    if (fontSize <= 24) return 1.3;
    return 1.5;
  }

  // ==================== 缓存管理 ====================

  void _cacheTextWidth(String key, double width) {
    if (_textWidthCache.length >= _maxWidthCacheSize) {
      // 清除最早的 20% 缓存
      final keysToRemove = _textWidthCache.keys.take(_maxWidthCacheSize ~/ 5);
      for (final k in keysToRemove) {
        _textWidthCache.remove(k);
      }
    }
    _textWidthCache[key] = width;
  }

  /// 清除缓存
  void clearCache() {
    _textWidthCache.clear();
    _fontMetricsCache.clear();
  }

  /// 清除文本宽度缓存
  void clearTextWidthCache() {
    _textWidthCache.clear();
  }

  // ==================== DPI 管理 ====================

  /// 设置 DPI 缩放
  void setDpiScale(double scale) {
    if (_dpiScale != scale) {
      _dpiScale = scale;
      clearCache();
    }
  }

  /// 获取 DPI 缩放
  double get dpiScale => _dpiScale;
}

/// 字体度量
class FontMetrics {
  final double ascent;
  final double descent;
  final double leading;
  final double height;
  final double avgCharWidth;
  final double maxCharWidth;

  const FontMetrics({
    required this.ascent,
    required this.descent,
    required this.leading,
    required this.height,
    required this.avgCharWidth,
    required this.maxCharWidth,
  });

  double get baseline => ascent;
}

/// 布局结果
class LayoutResult {
  final List<LayoutLine> lines;
  final double totalWidth;
  final double totalHeight;

  LayoutResult({
    required this.lines,
    required this.totalWidth,
    required this.totalHeight,
  });

  factory LayoutResult.fromJson(String json) {
    // 解析 JSON 结果
    // 实际实现需要根据 Rust 返回的格式解析
    return LayoutResult(
      lines: [],
      totalWidth: 0,
      totalHeight: 0,
    );
  }

  factory LayoutResult.fromMap(Map<String, dynamic> map) {
    final lineList = (map['lines'] as List?) ?? [];
    final lines = lineList.map((l) => LayoutLine.fromMap(l)).toList();

    return LayoutResult(
      lines: lines,
      totalWidth: (map['total_width'] as num?)?.toDouble() ?? 0,
      totalHeight: (map['total_height'] as num?)?.toDouble() ?? 0,
    );
  }
}

/// 布局行
class LayoutLine {
  final int lineNumber;
  final int startOffset;
  final int endOffset;
  final double width;
  final double height;
  final List<LayoutSpan> spans;

  LayoutLine({
    required this.lineNumber,
    required this.startOffset,
    required this.endOffset,
    required this.width,
    required this.height,
    required this.spans,
  });

  factory LayoutLine.fromMap(Map<String, dynamic> map) {
    final spanList = (map['spans'] as List?) ?? [];
    final spans = spanList.map((s) => LayoutSpan.fromMap(s)).toList();

    return LayoutLine(
      lineNumber: (map['line_number'] as num?)?.toInt() ?? 0,
      startOffset: (map['start_offset'] as num?)?.toInt() ?? 0,
      endOffset: (map['end_offset'] as num?)?.toInt() ?? 0,
      width: (map['width'] as num?)?.toDouble() ?? 0,
      height: (map['height'] as num?)?.toDouble() ?? 0,
      spans: spans,
    );
  }
}

/// 布局片段
class LayoutSpan {
  final int startOffset;
  final int endOffset;
  final String text;
  final double width;
  final double height;
  final TextStyle style;

  LayoutSpan({
    required this.startOffset,
    required this.endOffset,
    required this.text,
    required this.width,
    required this.height,
    required this.style,
  });

  factory LayoutSpan.fromMap(Map<String, dynamic> map) {
    return LayoutSpan(
      startOffset: (map['start_offset'] as num?)?.toInt() ?? 0,
      endOffset: (map['end_offset'] as num?)?.toInt() ?? 0,
      text: map['text'] ?? '',
      width: (map['width'] as num?)?.toDouble() ?? 0,
      height: (map['height'] as num?)?.toDouble() ?? 0,
      style: TextStyle.fromMap(map['style'] ?? {}),
    );
  }
}

/// 文本样式
class TextStyle {
  final bool bold;
  final bool italic;
  final bool underline;
  final double? fontSize;
  final String? fontFamily;
  final String? foreground;
  final String? background;

  const TextStyle({
    this.bold = false,
    this.italic = false,
    this.underline = false,
    this.fontSize,
    this.fontFamily,
    this.foreground,
    this.background,
  });

  factory TextStyle.fromMap(Map<String, dynamic> map) {
    return TextStyle(
      bold: map['bold'] ?? false,
      italic: map['italic'] ?? false,
      underline: map['underline'] ?? false,
      fontSize: (map['font_size'] as num?)?.toDouble(),
      fontFamily: map['font_family'],
      foreground: map['foreground'],
      background: map['background'],
    );
  }
}
