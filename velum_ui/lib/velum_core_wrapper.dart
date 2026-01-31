import 'dart:ffi';
import 'dart:io';
import 'bridge_generated.dart';

class VelumCoreWrapper {
  static final VelumCoreWrapper _instance = VelumCoreWrapper._internal();
  late final VelumCore api;

  factory VelumCoreWrapper() {
    return _instance;
  }

  VelumCoreWrapper._internal() {
    final String path;
    if (Platform.isMacOS) {
      path = '../velum_core/target/debug/libvelum_core.dylib';
    } else if (Platform.isWindows) {
      path = '../velum_core/target/debug/velum_core.dll';
    } else {
      throw UnsupportedError('Unsupported platform');
    }
    
    final dylib = DynamicLibrary.open(path);
    api = VelumCoreImpl(dylib);
  }
}
