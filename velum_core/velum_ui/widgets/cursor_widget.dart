// 光标 Widget
// 绘制和动画显示光标

import 'package:flutter/material.dart';
import 'package:flutter/scheduler.dart';

import '../view_models/selection_view_model.dart';
import '../view_models/layout_view_model.dart';

/// 光标配置
class CursorConfig {
  final Color color;
  final double width;
  final Duration blinkDuration;
  final Duration blinkInterval;
  final bool enableBlink;
  final Duration fadeDuration;

  const CursorConfig({
    this.color = Colors.black,
    this.width = 2.0,
    this.blinkDuration = const Duration(milliseconds: 700),
    this.blinkInterval = const Duration(milliseconds: 500),
    this.enableBlink = true,
    this.fadeDuration = const Duration(milliseconds: 150),
  });
}

/// 光标 Widget
class CursorWidget extends StatefulWidget {
  final SelectionViewModel viewModel;
  final LayoutViewModel layoutViewModel;
  final CursorConfig config;
  final Animation<double>? animation;

  const CursorWidget({
    super.key,
    required this.viewModel,
    required this.layoutViewModel,
    this.config = const CursorConfig(),
    this.animation,
  });

  @override
  State<CursorWidget> createState() => _CursorWidgetState();
}

class _CursorWidgetState extends State<CursorWidget>
    with SingleTickerProviderStateMixin {
  // 动画控制器
  late AnimationController _blinkController;

  // Ticker 回调
  Ticker? _ticker;

  // 可见性状态
  bool _isVisible = true;

  // 最后一次位置
  Offset? _lastPosition;

  @override
  void initState() {
    super.initState();

    if (widget.animation != null) {
      // 使用外部提供的动画
      widget.animation!.addListener(_onAnimationTick);
      _isVisible = widget.animation!.value >= 0.5;
    } else {
      // 创建内部动画
      _blinkController = AnimationController(
        vsync: this,
        duration: widget.config.blinkDuration,
      )..repeat(
          period: widget.config.blinkInterval,
          min: 0.0,
          max: 1.0,
        );

      _blinkController.addListener(_onAnimationTick);
    }

    widget.viewModel.addListener(_onSelectionChanged);
  }

  @override
  void didChangeDependencies() {
    super.didChangeDependencies();

    // 请求焦点以显示光标
    SchedulerBinding.instance.addPersistentFrameCallback((_) {
      if (mounted) {
        _updateCursorPosition();
      }
    });
  }

  @override
  void dispose() {
    if (widget.animation != null) {
      widget.animation!.removeListener(_onAnimationTick);
    } else {
      _blinkController.removeListener(_onAnimationTick);
      _blinkController.dispose();
    }

    widget.viewModel.removeListener(_onSelectionChanged);
    _ticker?.dispose();
    super.dispose();
  }

  // ==================== 事件处理 ====================

  void _onAnimationTick() {
    if (widget.animation != null) {
      _isVisible = widget.animation!.value >= 0.5;
    } else {
      _isVisible = _blinkController.value < 0.5;
    }

    if (mounted) {
      setState(() {});
    }
  }

  void _onSelectionChanged() {
    _updateCursorPosition();
  }

  void _updateCursorPosition() {
    final position = widget.viewModel.active;
    final offset = widget.layoutViewModel.getOffsetForOffset(position);

    if (offset != null) {
      setState(() {
        _lastPosition = offset;
      });
    }
  }

  // ==================== 构建 ====================

  @override
  Widget build(BuildContext context) {
    // 检查是否应该显示光标
    final hasFocus = Focus.of(context).hasFocus;
    final hasSelection = widget.viewModel.hasSelection;

    // 有选择时不显示光标，或者无焦点时隐藏
    if (hasSelection || !hasFocus) {
      return const SizedBox.shrink();
    }

    if (!_isVisible) {
      return const SizedBox.shrink();
    }

    // 计算光标位置
    final cursorRect = _calculateCursorRect();
    if (cursorRect == null) {
      return const SizedBox.shrink();
    }

    return Positioned(
      left: cursorRect!.left,
      top: cursorRect.top,
      width: widget.config.width,
      height: cursorRect.height,
      child: Container(
        decoration: BoxDecoration(
          color: widget.config.color,
          borderRadius: BorderRadius.circular(widget.config.width / 2),
        ),
      ),
    );
  }

  /// 计算光标矩形
  Rect? _calculateCursorRect() {
    final position = widget.viewModel.active;
    final offset = widget.layoutViewModel.getOffsetForOffset(position);

    if (offset == null) return null;

    final lineNumber = widget.layoutViewModel.getLineNumberForOffset(position);
    if (lineNumber == null) return null;

    final line = widget.layoutViewModel.lines.firstWhere(
      (l) => l.lineNumber == lineNumber,
      orElse: () => widget.layoutViewModel.lines.first,
    );

    return Rect.fromLTWH(
      offset.dx - widget.config.width / 2,
      offset.dy,
      widget.config.width,
      line.height,
    );
  }
}

/// 光标位置管理器
class CursorPositionManager {
  // 单例
  static final CursorPositionManager _instance =
      CursorPositionManager._internal();
  factory CursorPositionManager() => _instance;
  CursorPositionManager._internal();

  // 光标位置
  Offset? _position;
  Offset? get position => _position;

  // 可见性
  bool _isVisible = true;
  bool get isVisible => _isVisible;

  // 焦点状态
  bool _hasFocus = false;
  bool get hasFocus => _hasFocus;

  // 监听器
  final List<VoidCallback> _listeners = [];

  // ==================== 位置操作 ====================

  /// 设置光标位置
  void setPosition(Offset position) {
    _position = position;
    _notifyListeners();
  }

  /// 清除光标位置
  void clearPosition() {
    _position = null;
    _notifyListeners();
  }

  // ==================== 可见性操作 ====================

  /// 设置可见性
  void setVisible(bool visible) {
    if (_isVisible != visible) {
      _isVisible = visible;
      _notifyListeners();
    }
  }

  /// 显示光标
  void show() {
    setVisible(true);
  }

  /// 隐藏光标
  void hide() {
    setVisible(false);
  }

  /// 闪烁光标
  void blink({Duration? duration, Duration? interval}) {
    _isVisible = true;
    _notifyListeners();

    // 延迟后隐藏
    Future.delayed(duration ?? const Duration(milliseconds: 700), () {
      if (mounted) {
        _isVisible = false;
        _notifyListeners();
      }
    });
  }

  // ==================== 焦点操作 ====================

  /// 设置焦点状态
  void setFocus(bool focus) {
    if (_hasFocus != focus) {
      _hasFocus = focus;
      _notifyListeners();
    }
  }

  // ==================== 监听器 ====================

  void addListener(VoidCallback callback) {
    _listeners.add(callback);
  }

  void removeListener(VoidCallback callback) {
    _listeners.remove(callback);
  }

  void _notifyListeners() {
    for (final callback in _listeners) {
      callback();
    }
  }
}

/// 光标矩形
class CursorRect {
  final double x;
  final double y;
  final double width;
  final double height;

  const CursorRect({
    required this.x,
    required this.y,
    required this.width,
    required this.height,
  });

  Rect toRect() => Rect.fromLTWH(x, y, width, height);

  CursorRect shift(Offset offset) => CursorRect(
        x: x + offset.dx,
        y: y + offset.dy,
        width: width,
        height: height,
      );

  bool contains(Offset point) =>
      point.dx >= x &&
      point.dx <= x + width &&
      point.dy >= y &&
      point.dy <= y + height;
}
