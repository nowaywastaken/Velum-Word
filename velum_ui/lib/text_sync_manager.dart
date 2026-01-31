import 'dart:async';
import 'bridge_generated.dart';

class TextSyncManager {
  static const Duration _debounceDuration = Duration(milliseconds: 50);
  static const int _maxPendingOperations = 10;
  
  final VelumCore _api;
  Timer? _debounceTimer;
  List<_PendingOperation> _pendingOperations = [];
  bool _isProcessing = false;
  
  TextSyncManager(this._api);
  
  /// 处理文本变更（带防抖）
  Future<void> onTextChanged(String oldText, String newText) async {
    // 1. 无变化检测
    if (oldText == newText) return;
    
    // 2. 添加到待处理队列
    _pendingOperations.add(_PendingOperation(oldText, newText));
    
    // 3. 队列溢出保护：合并操作
    if (_pendingOperations.length > _maxPendingOperations) {
      _pendingOperations = [_mergeOperations(_pendingOperations)];
    }
    
    // 4. 防抖处理
    _debounceTimer?.cancel();
    _debounceTimer = Timer(_debounceDuration, _processOperations);
  }
  
  /// 批量处理操作队列
  Future<void> _processOperations() async {
    if (_isProcessing) return;
    _isProcessing = true;
    
    try {
      while (_pendingOperations.isNotEmpty) {
        final operation = _pendingOperations.removeAt(0);
        await _applyDiff(operation.oldText, operation.newText);
      }
    } finally {
      _isProcessing = false;
    }
  }
  
  /// 应用差异到 Rust 端
  Future<void> _applyDiff(String oldText, String newText) async {
    if (oldText == newText) return;
    
    final diff = _computeDiff(oldText, newText);
    if (diff == null) return;
    
    final deleteOffset = diff.deleteOffset;
    final deleteLength = diff.deleteLength;
    final insertOffset = diff.insertOffset;
    final insertText = diff.insertText;
    
    // 先删除后插入
    if (deleteLength > 0) {
      await _api.deleteText(offset: deleteOffset, length: deleteLength);
    }
    if (insertText.isNotEmpty) {
      await _api.insertText(offset: insertOffset, newText: insertText);
    }
  }
  
  /// 计算差异：返回 (deleteOffset, deleteLength, insertOffset, insertText)
  _DiffResult? _computeDiff(String oldText, String newText) {
    // 查找公共前缀
    int prefixLen = 0;
    final minLen = oldText.length < newText.length ? oldText.length : newText.length;
    while (prefixLen < minLen && oldText[prefixLen] == newText[prefixLen]) {
      prefixLen++;
    }
    
    // 查找公共后缀（避免重叠前缀）
    int suffixLen = 0;
    final maxSuffixLen = oldText.length - prefixLen < newText.length - prefixLen 
        ? oldText.length - prefixLen 
        : newText.length - prefixLen;
    
    while (suffixLen < maxSuffixLen && 
           oldText[oldText.length - 1 - suffixLen] == newText[newText.length - 1 - suffixLen]) {
      suffixLen++;
    }
    
    final deleteOffset = prefixLen;
    final deleteLength = oldText.length - prefixLen - suffixLen;
    final insertOffset = prefixLen;
    final insertText = newText.substring(newText.length - suffixLen);
    
    return _DiffResult(
      deleteOffset: deleteOffset,
      deleteLength: deleteLength,
      insertOffset: insertOffset,
      insertText: insertText,
    );
  }
  
  /// 合并多个操作为单个操作
  _PendingOperation _mergeOperations(List<_PendingOperation> operations) {
    assert(operations.isNotEmpty);
    if (operations.length == 1) return operations.first;
    
    String merged = operations.first.oldText;
    for (var op in operations) {
      merged = op.newText;
    }
    
    return _PendingOperation(operations.first.oldText, merged);
  }
  
  /// 强制刷新所有待处理操作
  Future<void> flush() async {
    _debounceTimer?.cancel();
    await _processOperations();
  }
  
  /// 释放资源
  void dispose() {
    _debounceTimer?.cancel();
  }
}

class _PendingOperation {
  final String oldText;
  final String newText;
  
  _PendingOperation(this.oldText, this.newText);
}

class _DiffResult {
  final int deleteOffset;
  final int deleteLength;
  final int insertOffset;
  final String insertText;
  
  _DiffResult({
    required this.deleteOffset,
    required this.deleteLength,
    required this.insertOffset,
    required this.insertText,
  });
}
