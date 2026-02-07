// Velum Core Flutter UI Integration Layer
// Flutter UI 集成层，将 Rust 核心功能暴露给 Flutter

library velum_ui;

export 'bindings/velum_core_bindings.dart';
export 'view_models/view_models.dart';
export 'services/services.dart';
export 'widgets/widgets.dart';

// ==================== 初始化 ====================

import 'dart:async';
import 'services/document_service.dart';
import 'services/rendering_service.dart';
import 'services/file_service.dart';

/// 初始化所有服务
Future<void> initializeVelumServices() async {
  await DocumentService().initialize();
  await RenderingService().initialize();
  await FileService().initialize();
}

/// 服务容器
class VelumServiceContainer {
  // 单例
  static final VelumServiceContainer _instance = VelumServiceContainer._internal();
  factory VelumServiceContainer() => _instance;
  VelumServiceContainer._internal();

  // 服务实例
  final DocumentService documentService = DocumentService();
  final RenderingService renderingService = RenderingService();
  final FileService fileService = FileService();

  // 初始化状态
  bool _initialized = false;
  bool get isInitialized => _initialized;

  /// 初始化所有服务
  Future<void> initialize() async {
    if (_initialized) return;

    await documentService.initialize();
    await renderingService.initialize();
    await fileService.initialize();

    _initialized = true;
  }

  /// 确保已初始化
  Future<void> ensureInitialized() async {
    if (!_initialized) {
      await initialize();
    }
  }
}

/// 快速访问单例
VelumServiceContainer get velumServices => VelumServiceContainer();
