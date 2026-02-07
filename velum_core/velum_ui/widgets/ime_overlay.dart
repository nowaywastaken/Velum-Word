// IME 候选词窗口
// 处理输入法编辑器候选词显示

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import '../view_models/selection_view_model.dart';
import '../view_models/layout_view_model.dart';

/// 候选词类型
enum CandidateType {
  none,
  composing, // 正在输入
  candidates, // 候选词列表
  auto_correct, // 自动更正
  next_word, // 下一个单词建议
}

/// 候选词窗口配置
class CandidateWindowConfig {
  final double maxHeight;
  final double itemHeight;
  final int maxVisibleItems;
  final Color backgroundColor;
  final Color highlightColor;
  final Color selectedItemColor;
  final TextStyle itemTextStyle;
  final TextStyle selectedItemTextStyle;
  final BorderRadius borderRadius;
  final EdgeInsets padding;
  final double elevation;

  const CandidateWindowConfig({
    this.maxHeight = 200,
    this.itemHeight = 40,
    this.maxVisibleItems = 5,
    this.backgroundColor = Colors.white,
    this.highighlightColor = Colors.blue,
    this.selectedItemColor = Colors.blue,
    this.itemTextStyle = const TextStyle(fontSize: 16),
    this.selectedItemTextStyle = const TextStyle(fontSize: 16, color: Colors.white),
    this.borderRadius = const BorderRadius.all(Radius.circular(8)),
    this.padding = EdgeInsets.zero,
    this.elevation = 4,
  });
}

/// 候选词
class CandidateWord {
  final String text;
  final String? label;
  final CandidateType type;
  final Map<String, dynamic>? metadata;

  const CandidateWord({
    required this.text,
    this.label,
    this.type = CandidateType.candidates,
    this.metadata,
  });

  factory CandidateWord.composing(String text) {
    return CandidateWord(
      text: text,
      type: CandidateType.composing,
    );
  }

  factory CandidateWord.candidate(String text, {String? label}) {
    return CandidateWord(
      text: text,
      label: label,
      type: CandidateType.candidates,
    );
  }
}

/// 候选词窗口状态
class CandidateWindowState {
  final bool isVisible;
  final List<CandidateWord> candidates;
  final int selectedIndex;
  final Offset position;
  final CandidateType type;
  final String composingText;

  const CandidateWindowState({
    this.isVisible = false,
    this.candidates = const [],
    this.selectedIndex = 0,
    this.position = Offset.zero,
    this.type = CandidateType.none,
    this.composingText = '',
  });

  CandidateWindowState copyWith({
    bool? isVisible,
    List<CandidateWord>? candidates,
    int? selectedIndex,
    Offset? position,
    CandidateType? type,
    String? composingText,
  }) {
    return CandidateWindowState(
      isVisible: isVisible ?? this.isVisible,
      candidates: candidates ?? this.candidates,
      selectedIndex: selectedIndex ?? this.selectedIndex,
      position: position ?? this.position,
      type: type ?? this.type,
      composingText: composingText ?? this.composingText,
    );
  }

  CandidateWord? get selectedCandidate =>
      (selectedIndex >= 0 && selectedIndex < candidates.length)
          ? candidates[selectedIndex]
          : null;
}

/// IME 叠加层 Widget
class ImeOverlay extends StatefulWidget {
  final SelectionViewModel viewModel;
  final LayoutViewModel layoutViewModel;
  final CandidateWindowConfig config;

  const ImeOverlay({
    super.key,
    required this.viewModel,
    required this.layoutViewModel,
    this.config = const CandidateWindowConfig(),
  });

  @override
  State<ImeOverlay> createState() => _ImeOverlayState();
}

class _ImeOverlayState extends State<ImeOverlay> {
  // 候选词窗口状态
  final _state = ValueNotifier<CandidateWindowState>(const CandidateWindowState());

  // 焦点节点
  final FocusNode _focusNode = FocusNode();

  // IME 输入控制器
  final TextEditingController _imeController = TextEditingController();

  @override
  void initState() {
    super.initState();

    _focusNode.addListener(_onFocusChange);

    // 监听原始文本字段
    // 实际应该与 DocumentWidget 的文本控制器同步
  }

  @override
  void dispose() {
    _focusNode.removeListener(_onFocusChange);
    _focusNode.dispose();
    _imeController.dispose();
    super.dispose();
  }

  void _onFocusChange() {
    if (!_focusNode.hasFocus) {
      _hideCandidateWindow();
    }
  }

  // ==================== 公开方法 ====================

  /// 显示候选词窗口
  void showCandidates(List<CandidateWord> candidates, Offset position) {
    _state.value = _state.value.copyWith(
      isVisible: true,
      candidates: candidates,
      selectedIndex: 0,
      position: position,
      type: CandidateType.candidates,
    );
  }

  /// 更新候选词列表
  void updateCandidates(List<CandidateWord> candidates) {
    if (_state.value.isVisible) {
      _state.value = _state.value.copyWith(candidates: candidates);
    }
  }

  /// 更新组合文字
  void updateComposingText(String text, Offset position) {
    _state.value = _state.value.copyWith(
      composingText: text,
      position: position,
      type: CandidateType.composing,
    );
  }

  /// 隐藏候选词窗口
  void hide() {
    _hideCandidateWindow();
  }

  void _hideCandidateWindow() {
    _state.value = const CandidateWindowState();
  }

  /// 选择候选词
  void selectCandidate(int index) {
    if (index >= 0 && index < _state.value.candidates.length) {
      _state.value = _state.value.copyWith(selectedIndex: index);
    }
  }

  /// 确认选中的候选词
  String? confirmSelection() {
    final selected = _state.value.selectedCandidate;
    if (selected != null) {
      _hideCandidateWindow();
      return selected.text;
    }
    return null;
  }

  /// 选择上一个候选词
  void selectPrevious() {
    final newIndex = (_state.value.selectedIndex - 1).clamp(0, _state.value.candidates.length - 1);
    _state.value = _state.value.copyWith(selectedIndex: newIndex);
  }

  /// 选择下一个候选词
  void selectNext() {
    final newIndex = (_state.value.selectedIndex + 1).clamp(0, _state.value.candidates.length - 1);
    _state.value = _state.value.copyWith(selectedIndex: newIndex);
  }

  // ==================== 构建 ====================

  @override
  Widget build(BuildContext context) {
    return ValueListenableBuilder<CandidateWindowState>(
      valueListenable: _state,
      builder: (context, state, child) {
        if (!state.isVisible) {
          return const SizedBox.shrink();
        }

        return Positioned(
          left: state.position.dx,
          top: state.position.dy,
          child: _buildCandidateWindow(state),
        );
      },
    );
  }

  Widget _buildCandidateWindow(CandidateWindowState state) {
    final height = state.candidates.length * widget.config.itemHeight;

    return Material(
      elevation: widget.config.elevation,
      borderRadius: widget.config.borderRadius,
      child: Container(
        width: 300,
        height: height.clamp(0, widget.config.maxHeight),
        decoration: BoxDecoration(
          color: widget.config.backgroundColor,
          borderRadius: widget.config.borderRadius,
        ),
        child: ListView.builder(
          itemCount: state.candidates.length,
          itemExtent: widget.config.itemHeight,
          padding: widget.config.padding,
          itemBuilder: (context, index) {
            return _buildCandidateItem(state.candidates[index], index == state.selectedIndex);
          },
        ),
      ),
    );
  }

  Widget _buildCandidateItem(CandidateWord candidate, bool isSelected) {
    return InkWell(
      onTap: () {
        _confirmCandidate(candidate);
      },
      child: Container(
        alignment: Alignment.centerLeft,
        padding: const EdgeInsets.symmetric(horizontal: 16),
        color: isSelected ? widget.config.selectedItemColor : Colors.transparent,
        child: Row(
          children: [
            // 候选词文本
            Expanded(
              child: Text(
                candidate.text,
                style: isSelected
                    ? widget.config.selectedItemTextStyle
                    : widget.config.itemTextStyle,
              ),
            ),

            // 标签
            if (candidate.label != null)
              Padding(
                padding: const EdgeInsets.only(left: 8),
                child: Text(
                  candidate.label!,
                  style: TextStyle(
                    fontSize: 12,
                    color: isSelected ? Colors.white70 : Colors.grey,
                  ),
                ),
              ),
          ],
        ),
      ),
    );
  }

  void _confirmCandidate(CandidateWord candidate) {
    // TODO: 将候选词插入到文档中
    hide();
  }
}

/// IME 处理服务
class ImeHandler {
  // 单例
  static final ImeHandler _instance = ImeHandler._internal();
  factory ImeHandler() => _instance;
  ImeHandler._internal();

  // 候选词窗口引用
  ImeOverlayState? _overlayState;

  // 状态监听
  final List<VoidCallback> _stateListeners = [];

  // ==================== 注册/注销 ====================

  /// 注册候选词窗口
  void registerOverlay(ImeOverlayState overlayState) {
    _overlayState = overlayState;
  }

  /// 注销候选词窗口
  void unregisterOverlay() {
    _overlayState = null;
  }

  // ==================== IME 事件处理 ====================

  /// 处理文本输入
  void handleTextInput(String text) {
    _overlayState?.updateComposingText(text, Offset.zero);
  }

  /// 处理提交文本
  void handleSubmit(String text) {
    _overlayState?.hide();
  }

  /// 处理删除
  void handleDelete() {
    if (_overlayState != null) {
      final currentText = _overlayState!._state.value.composingText;
      if (currentText.length > 1) {
        _overlayState?.updateComposingText(
          currentText.substring(0, currentText.length - 1),
          Offset.zero,
        );
      } else {
        _overlayState?.hide();
      }
    }
  }

  /// 处理候选词选择
  void handleSelection(int index) {
    _overlayState?.selectCandidate(index);
  }

  /// 处理候选词确认
  void handleConfirmation() {
    final text = _overlayState?.confirmSelection();
    if (text != null) {
      _notifyStateChanged();
    }
  }

  /// 处理上一个候选词
  void handlePreviousCandidate() {
    _overlayState?.selectPrevious();
  }

  /// 处理下一个候选词
  void handleNextCandidate() {
    _overlayState?.selectNext();
  }

  // ==================== 状态监听 ====================

  void addStateListener(VoidCallback callback) {
    _stateListeners.add(callback);
  }

  void removeStateListener(VoidCallback callback) {
    _stateListeners.remove(callback);
  }

  void _notifyStateChanged() {
    for (final callback in _stateListeners) {
      callback();
    }
  }
}

/// 快捷键处理
class ImeShortcutHandler {
  // 快捷键映射
  static const Map<LogicalKeyboardKey, void Function(ImeHandler)> _shortcuts = {
    LogicalKeyboardKey.arrowLeft: _handleArrowLeft,
    LogicalKeyboardKey.arrowRight: _handleArrowRight,
    LogicalKeyboardKey.arrowUp: _handleArrowUp,
    LogicalKeyboardKey.arrowDown: _handleArrowDown,
    LogicalKeyboardKey.enter: _handleEnter,
    LogicalKeyboardKey.tab: _handleTab,
    LogicalKeyboardKey.escape: _handleEscape,
    LogicalKeyboardKey.delete: _handleDelete,
    LogicalKeyboardKey.backspace: _handleBackspace,
  };

  static void _handleArrowLeft(ImeHandler handler) {
    handler.handlePreviousCandidate();
  }

  static void _handleArrowRight(ImeHandler handler) {
    handler.handleNextCandidate();
  }

  static void _handleArrowUp(ImeHandler handler) {
    handler.handlePreviousCandidate();
  }

  static void _handleArrowDown(ImeHandler handler) {
    handler.handleNextCandidate();
  }

  static void _handleEnter(ImeHandler handler) {
    handler.handleConfirmation();
  }

  static void _handleTab(ImeHandler handler) {
    handler.handleConfirmation();
  }

  static void _handleEscape(ImeHandler handler) {
    handler.hide();
  }

  static void _handleDelete(ImeHandler handler) {
    handler.handleDelete();
  }

  static void _handleBackspace(ImeHandler handler) {
    handler.handleDelete();
  }

  /// 处理键盘事件
  static bool handleKeyEvent(KeyEvent event, ImeHandler handler) {
    if (_shortcuts.containsKey(event.logicalKey)) {
      _shortcuts[event.logicalKey]!(handler);
      return true;
    }
    return false;
  }
}
