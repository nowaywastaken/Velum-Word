// 选择视图模型 - 管理选择状态
// 处理选择、拖拽选择和光标位置

import 'package:flutter/foundation.dart';
import 'package:rxdart/rxdart.dart';

/// 选择类型
enum SelectionMode {
  none, // 无选择
  caret, // 光标（折叠选择）
  character, // 字符选择
  word, // 单词选择
  line, // 行选择
  block, // 块选择（列选择）
  all, // 全选
}

/// 选择范围
class SelectionRange {
  final int start;
  final int end;

  SelectionRange({required this.start, required this.end}) : assert(start <= end);

  int get length => end - start;

  bool get isEmpty => start == end;

  bool contains(int offset) => offset >= start && offset <= end;

  SelectionRange expandTo(int offset) {
    if (offset < start) {
      return SelectionRange(start: offset, end: end);
    } else if (offset > end) {
      return SelectionRange(start: start, end: offset);
    }
    return this;
  }

  SelectionRange collapseTo(int offset) {
    return SelectionRange(start: offset, end: offset);
  }

  SelectionRange shift(int offset) {
    return SelectionRange(start: start + offset, end: end + offset);
  }

  Map<String, dynamic> toJson() => {'start': start, 'end': end};

  factory SelectionRange.fromJson(Map<String, dynamic> json) {
    return SelectionRange(
      start: json['start'] ?? 0,
      end: json['end'] ?? 0,
    );
  }

  @override
  bool operator ==(Object other) {
    if (identical(this, other)) return true;
    if (other is! SelectionRange) return false;
    return other.start == start && other.end == end;
  }

  @override
  int get hashCode => start.hashCode ^ end.hashCode;

  @override
  String toString() => 'SelectionRange($start, $end)';
}

/// 视觉反馈样式
enum VisualFeedbackStyle {
  none,
  highlight, // 高亮背景
  caret, // 光标线
  outline, // 边框轮廓
  dotted, // 虚线边框
}

/// 拖拽选择配置
class DragSelectionConfig {
  final SelectionMode mode;
  final VisualFeedbackStyle feedbackStyle;
  final bool scrollOnEdge;
  final double edgeScrollThreshold;
  final double edgeScrollSpeed;

  const DragSelectionConfig({
    this.mode = SelectionMode.character,
    this.feedbackStyle = VisualFeedbackStyle.highlight,
    this.scrollOnEdge = true,
    this.edgeScrollThreshold = 20.0,
    this.edgeScrollSpeed = 8.0,
  });
}

/// 拖拽阶段
enum DragPhaseType {
  idle, // 空闲
  preparing, // 准备中（按下但未移动足够距离）
  selecting, // 选择中
  extending, // 扩展中（按住 Shift）
  canceling, // 取消中
}

/// 拖拽目标
enum DragTarget {
  none,
  text, // 文本区域
  selection, // 已选择区域
  gutter, // 行号区域
  scrollbar, // 滚动条
  widgetBoundary, // Widget 边界
}

/// 拖拽阶段信息
class DragPhase {
  final DragPhaseType type;
  final int? anchor;
  final int? active;

  const DragPhase({
    required this.type,
    this.anchor,
    this.active,
  });

  bool get isActive => type != DragPhaseType.idle && type != DragPhaseType.canceling;
}

/// 拖拽选择状态
class DragSelectionStateModel {
  final bool isDragging;
  final bool isSelecting;
  final int? startOffset;
  final int? currentOffset;
  final SelectionRange? currentSelection;
  final DragPhaseType phase;

  const DragSelectionStateModel({
    this.isDragging = false,
    this.isSelecting = false,
    this.startOffset,
    this.currentOffset,
    this.currentSelection,
    this.phase = DragPhaseType.idle,
  });

  factory DragSelectionStateModel.initial() => DragSelectionStateModel();

  DragSelectionStateModel copyWith({
    bool? isDragging,
    bool? isSelecting,
    ValueGetter<int?>? getStartOffset,
    int? startOffset,
    ValueGetter<int?>? getCurrentOffset,
    int? currentOffset,
    ValueGetter<SelectionRange?>? getCurrentSelection,
    SelectionRange? currentSelection,
    DragPhaseType? phase,
  }) {
    return DragSelectionStateModel(
      isDragging: isDragging ?? this.isDragging,
      isSelecting: isSelecting ?? this.isSelecting,
      startOffset: getStartOffset != null ? getStartOffset() : (startOffset ?? this.startOffset),
      currentOffset: getCurrentOffset != null ? getCurrentOffset() : (currentOffset ?? this.currentOffset),
      currentSelection: getCurrentSelection != null ? getCurrentSelection() : (currentSelection ?? this.currentSelection),
      phase: phase ?? this.phase,
    );
  }
}

/// 选择视图模型
class SelectionViewModel with ChangeNotifier {
  // 选择状态流
  final _selectionController = BehaviorSubject<SelectionRange?>();
  Stream<SelectionRange?> get selectionStream => _selectionController.stream;

  final _modeController = BehaviorSubject<SelectionMode>.seeded(SelectionMode.none);
  Stream<SelectionMode> get modeStream => _modeController.stream;

  final _dragStateController = BehaviorSubject<DragSelectionStateModel>.seeded(
    DragSelectionStateModel.initial(),
  );
  Stream<DragSelectionStateModel> get dragStateStream =>
      _dragStateController.stream;

  // 状态
  SelectionRange? _selection;
  SelectionMode _mode = SelectionMode.none;
  int _anchor = 0;
  int _active = 0;
  DragSelectionStateModel _dragState = DragSelectionStateModel.initial();

  // 配置
  final DragSelectionConfig _config;

  SelectionViewModel({DragSelectionConfig? config})
      : _config = config ?? const DragSelectionConfig() {
    _initialize();
  }

  void _initialize() {
    // 初始化为空选择
    _selection = null;
    _mode = SelectionMode.none;
    _selectionController.add(null);
  }

  // ==================== 基础选择操作 ====================

  /// 设置选择范围
  void setSelection(int anchor, int active, {SelectionMode? mode}) {
    final start = anchor < active ? anchor : active;
    final end = anchor < active ? active : anchor;

    _anchor = anchor;
    _active = active;
    _selection = SelectionRange(start: start, end: end);

    if (mode != null) {
      _mode = mode;
      _modeController.add(mode);
    } else if (anchor != active) {
      _mode = _determineSelectionMode();
      _modeController.add(_mode);
    } else {
      _mode = SelectionMode.caret;
      _modeController.add(_mode);
    }

    _selectionController.add(_selection);
    notifyListeners();
  }

  /// 清除选择
  void clearSelection() {
    _selection = null;
    _anchor = _active;
    _mode = SelectionMode.none;
    _selectionController.add(null);
    _modeController.add(SelectionMode.none);
    notifyListeners();
  }

  /// 将选择折叠到位置
  void collapseTo(int offset) {
    _anchor = offset;
    _active = offset;
    _selection = null;
    _mode = SelectionMode.caret;
    _selectionController.add(null);
    _modeController.add(SelectionMode.caret);
    notifyListeners();
  }

  /// 移动选择到位置
  void moveSelectionTo(int offset, {bool extend = false}) {
    if (extend && _selection != null) {
      // 扩展选择
      _active = offset;
      setSelection(_anchor, _active);
    } else {
      // 移动光标（折叠选择）
      collapseTo(offset);
    }
  }

  /// 全选
  void selectAll() {
    _anchor = 0;
    _active = 0; // 这里应该传入文档总长度
    _selection = SelectionRange(start: 0, end: _active);
    _mode = SelectionMode.all;
    _selectionController.add(_selection);
    _modeController.add(SelectionMode.all);
    notifyListeners();
  }

  // ==================== 单词选择 ====================

  /// 选中单词
  void selectWord(int offset, String text) {
    final wordRange = _findWordBounds(offset, text);
    _anchor = wordRange.start;
    _active = wordRange.end;
    _selection = wordRange;
    _mode = SelectionMode.word;
    _selectionController.add(_selection);
    _modeController.add(SelectionMode.word);
    notifyListeners();
  }

  /// 查找单词边界
  SelectionRange _findWordBounds(int offset, String text) {
    // 简单的单词边界检测
    // 实际实现应该考虑 Unicode 字符类
    final runes = text.runes.toList();

    if (offset < 0 || offset >= runes.length) {
      return SelectionRange(start: offset, end: offset);
    }

    int start = offset;
    int end = offset + 1;

    // 向前查找非单词字符
    while (start > 0 && _isWordCharacter(runes[start - 1])) {
      start--;
    }

    // 向后查找非单词字符
    while (end < runes.length && _isWordCharacter(runes[end])) {
      end++;
    }

    return SelectionRange(start: start, end: end);
  }

  bool _isWordCharacter(int rune) {
    // 简单判断：字母、数字、下划线为单词字符
    return (rune >= 0x41 && rune <= 0x5A) || // A-Z
        (rune >= 0x61 && rune <= 0x7A) || // a-z
        (rune >= 0x30 && rune <= 0x39) || // 0-9
        rune == 0x5F; // _
  }

  // ==================== 行选择 ====================

  /// 选中行
  void selectLine(int offset, int lineStart, int lineEnd) {
    _anchor = lineStart;
    _active = lineEnd;
    _selection = SelectionRange(start: lineStart, end: lineEnd);
    _mode = SelectionMode.line;
    _selectionController.add(_selection);
    _modeController.add(SelectionMode.line);
    notifyListeners();
  }

  // ==================== 拖拽选择 ====================

  /// 开始拖拽选择
  void startDrag(int offset, {bool extend = false}) {
    _dragState = _dragState.copyWith(
      isDragging: true,
      isSelecting: true,
      startOffset: offset,
      currentOffset: offset,
      phase: DragPhaseType.selecting,
    );

    if (extend) {
      // 带 Shift 的拖拽，扩展现有选择
      _dragState = _dragState.copyWith(phase: DragPhaseType.extending);
    } else {
      // 新拖拽
      if (_selection != null) {
        // 清除现有选择
        clearSelection();
      }
      _anchor = offset;
      _active = offset;
      _dragState = _dragState.copyWith(currentSelection: null);
    }

    _dragStateController.add(_dragState);
    notifyListeners();
  }

  /// 更新拖拽位置
  void updateDrag(int offset) {
    if (!_dragState.isDragging) return;

    _active = offset;
    final start = _anchor < _active ? _anchor : _active;
    final end = _anchor < _active ? _active : _anchor;

    _selection = SelectionRange(start: start, end: end);
    _mode = _determineSelectionMode();

    _dragState = _dragState.copyWith(
      currentOffset: offset,
      currentSelection: _selection,
      phase: DragPhaseType.selecting,
    );

    _selectionController.add(_selection);
    _modeController.add(_mode);
    _dragStateController.add(_dragState);
    notifyListeners();
  }

  /// 结束拖拽选择
  void endDrag() {
    if (_dragState.isDragging) {
      _dragState = _dragState.copyWith(
        isDragging: false,
        isSelecting: false,
        phase: DragPhaseType.idle,
      );
      _dragStateController.add(_dragState);
      notifyListeners();
    }
  }

  /// 取消拖拽
  void cancelDrag() {
    _dragState = _dragState.copyWith(
      isDragging: false,
      isSelecting: false,
      startOffset: null,
      currentOffset: null,
      currentSelection: null,
      phase: DragPhaseType.canceling,
    );
    _dragStateController.add(_dragState);
    notifyListeners();
  }

  // ==================== 选择扩展 ====================

  /// 扩展选择到单词边界
  void extendSelectionToWord(int offset, String text) {
    final wordRange = _findWordBounds(offset, text);
    setSelection(wordRange.start, wordRange.end, mode: SelectionMode.word);
  }

  /// 扩展选择到行边界
  void extendSelectionToLine(int lineStart, int lineEnd) {
    setSelection(lineStart, lineEnd, mode: SelectionMode.line);
  }

  /// 扩展选择到文档开始
  void extendSelectionToDocumentStart() {
    setSelection(0, _anchor, mode: SelectionMode.character);
  }

  /// 扩展选择到文档结束
  void extendSelectionToDocumentEnd(int documentLength) {
    setSelection(documentLength, _anchor, mode: SelectionMode.character);
  }

  // ==================== 选择变换 ====================

  /// 收缩选择（减少一个字符）
  void shrinkSelection({bool fromStart = true}) {
    if (_selection == null || _selection!.length <= 1) {
      clearSelection();
      return;
    }

    if (fromStart) {
      setSelection(_selection!.start + 1, _selection!.end);
    } else {
      setSelection(_selection!.start, _selection!.end - 1);
    }
  }

  /// 扩展选择（增加一个字符）
  void expandSelection({bool atStart = true}) {
    if (_selection == null) {
      collapseTo(_anchor);
      return;
    }

    if (atStart) {
      setSelection(_selection!.start - 1, _selection!.end);
    } else {
      setSelection(_selection!.start, _selection!.end + 1);
    }
  }

  // ==================== Getters ====================

  SelectionRange? get selection => _selection;
  SelectionMode get mode => _mode;
  int get anchor => _anchor;
  int get active => _active;
  DragSelectionStateModel get dragState => _dragState;

  bool get hasSelection => _selection != null && !_selection!.isEmpty;
  bool get isDragging => _dragState.isDragging;

  // ==================== 辅助方法 ====================

  SelectionMode _determineSelectionMode() {
    if (_selection == null || _selection!.length <= 1) {
      return SelectionMode.caret;
    }
    return SelectionMode.character;
  }

  /// 获取选择文本在原始字符串中的范围
  SelectionRange? getSelectionRangeForOffset(int documentOffset) {
    if (_selection == null) return null;

    if (_selection!.contains(documentOffset)) {
      return _selection;
    }
    return null;
  }

  @override
  void dispose() {
    _selectionController.close();
    _modeController.close();
    _dragStateController.close();
    super.dispose();
  }
}
