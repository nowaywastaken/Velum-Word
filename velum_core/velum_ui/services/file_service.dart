// 文件操作服务
// 处理文件读写和文档导入导出

import 'dart:async';
import 'dart:io';
import 'dart:typed_data';

import '../bindings/velum_core_bindings_generated.dart';
import 'document_service.dart';

/// 文件类型枚举
enum FileType {
  unknown,
  json, // 内部 JSON 格式
  text, // 纯文本
  docx, // Word 文档
  odt, // OpenDocument Text
  rtf, // Rich Text Format
  html, // HTML
  pdf, // PDF
}

/// 文件类型检测
FileType detectFileType(String path) {
  final ext = path.split('.').last.toLowerCase();

  switch (ext) {
    case 'json':
      return FileType.json;
    case 'txt':
      return FileType.text;
    case 'docx':
      return FileType.docx;
    case 'odt':
      return FileType.odt;
    case 'rtf':
      return FileType.rtf;
    case 'html':
    case 'htm':
      return FileType.html;
    case 'pdf':
      return FileType.pdf;
    default:
      return FileType.unknown;
  }
}

/// 导入结果
class ImportResult {
  final bool success;
  final String? content;
  final String? title;
  final String? author;
  final int? wordCount;
  final int? charCount;
  final String? error;

  const ImportResult({
    required this.success,
    this.content,
    this.title,
    this.author,
    this.wordCount,
    this.charCount,
    this.error,
  });

  factory ImportResult.success(String content, {String? title, String? author, int? wordCount, int? charCount}) {
    return ImportResult(
      success: true,
      content: content,
      title: title,
      author: author,
      wordCount: wordCount,
      charCount: charCount,
    );
  }

  factory ImportResult.failure(String error) {
    return ImportResult(
      success: false,
      error: error,
    );
  }
}

/// 导出结果
class ExportResult {
  final bool success;
  final String? filePath;
  final Uint8List? bytes;
  final String? error;

  const ExportResult({
    required this.success,
    this.filePath,
    this.bytes,
    this.error,
  });

  factory ExportResult.success(String filePath) {
    return ExportResult(success: true, filePath: filePath);
  }

  factory ExportResult.successBytes(Uint8List bytes) {
    return ExportResult(success: true, bytes: bytes);
  }

  factory ExportResult.failure(String error) {
    return ExportResult(success: false, error: error);
  }
}

/// 文件操作服务
class FileService {
  // 单例模式
  static final FileService _instance = FileService._internal();
  factory FileService() => _instance;
  FileService._internal();

  final DocumentService _documentService = DocumentService();

  /// 初始化服务
  Future<void> initialize() async {
    await _documentService.initialize();
  }

  // ==================== 导入操作 ====================

  /// 导入文件
  Future<ImportResult> importFile(String path) async {
    final fileType = detectFileType(path);

    switch (fileType) {
      case FileType.json:
        return await _importJson(path);
      case FileType.text:
        return await _importText(path);
      case FileType.docx:
        return await _importDocx(path);
      case FileType.html:
        return await _importHtml(path);
      default:
        return ImportResult.failure('Unsupported file type: ${path.split('.').last}');
    }
  }

  /// 导入 JSON 文件
  Future<ImportResult> _importJson(String path) async {
    try {
      final file = File(path);
      final content = await file.readAsString();
      final json = await _documentService.loadDocumentFromJson(content);

      if (json.startsWith('Error:')) {
        return ImportResult.failure(json);
      }

      return ImportResult.success(
        json,
        title: await _documentService.getDocumentTitle(),
        author: await _documentService.getDocumentAuthor(),
        wordCount: await _documentService.getWordCount(),
        charCount: await _documentService.getCharCount(),
      );
    } catch (e) {
      return ImportResult.failure('Failed to read JSON file: $e');
    }
  }

  /// 导入纯文本文件
  Future<ImportResult> _importText(String path) async {
    try {
      final file = File(path);
      final content = await file.readAsString();

      await _documentService.loadDocumentFromText(content);

      return ImportResult.success(
        content,
        title: File(path).uri.pathSegments.last,
        wordCount: content.split(RegExp(r'\s+')).where((w) => w.isNotEmpty).length,
        charCount: content.length,
      );
    } catch (e) {
      return ImportResult.failure('Failed to read text file: $e');
    }
  }

  /// 导入 DOCX 文件
  Future<ImportResult> _importDocx(String path) async {
    try {
      final result = await _documentService.loadOoxmlDocument(path);

      if (result.startsWith('Error:') || result.startsWith('File error:')) {
        return ImportResult.failure(result);
      }

      // 解析 OOXML 结果
      // final doc = jsonDecode(result);
      // final content = doc['text'] ?? '';

      return ImportResult.success(
        result,
        title: await _documentService.getDocumentTitle(),
        author: await _documentService.getDocumentAuthor(),
        wordCount: await _documentService.getWordCount(),
        charCount: await _documentService.getCharCount(),
      );
    } catch (e) {
      return ImportResult.failure('Failed to read DOCX file: $e');
    }
  }

  /// 导入 HTML 文件
  Future<ImportResult> _importHtml(String path) async {
    try {
      final file = File(path);
      final content = await file.readAsString();

      // 简单提取纯文本（实际应该使用 HTML 解析器）
      final textContent = _stripHtmlTags(content);

      await _documentService.loadDocumentFromText(textContent);

      return ImportResult.success(
        textContent,
        title: File(path).uri.pathSegments.last,
        wordCount: textContent.split(RegExp(r'\s+')).where((w) => w.isNotEmpty).length,
        charCount: textContent.length,
      );
    } catch (e) {
      return ImportResult.failure('Failed to read HTML file: $e');
    }
  }

  String _stripHtmlTags(String html) {
    return html.replaceAll(RegExp(r'<[^>]*>'), '').trim();
  }

  // ==================== 导出操作 ====================

  /// 导出文件
  Future<ExportResult> exportFile(String path, String content, {FileType? forceType}) async {
    final fileType = forceType ?? detectFileType(path);

    switch (fileType) {
      case FileType.json:
        return await _exportJson(path, content);
      case FileType.text:
        return await _exportText(path, content);
      case FileType.docx:
        return await _exportDocx(path, content);
      case FileType.html:
        return await _exportHtml(path, content);
      default:
        return ExportResult.failure('Unsupported file type for export');
    }
  }

  /// 导出为 JSON
  Future<ExportResult> _exportJson(String path, String content) async {
    try {
      // 获取当前文档的 JSON 表示
      final json = await _documentService.saveToFile(path);

      if (json.startsWith('Error:')) {
        return ExportResult.failure(json);
      }

      return ExportResult.success(path);
    } catch (e) {
      return ExportResult.failure('Failed to export JSON: $e');
    }
  }

  /// 导出为纯文本
  Future<ExportResult> _exportText(String path, String content) async {
    try {
      final result = await _documentService.exportToTxt(path);

      if (result.startsWith('Error:')) {
        return ExportResult.failure(result);
      }

      return ExportResult.success(path);
    } catch (e) {
      return ExportResult.failure('Failed to export text: $e');
    }
  }

  /// 导出为 DOCX
  Future<ExportResult> _exportDocx(String path, String content) async {
    try {
      // 构建文档 JSON
      final documentJson = await _documentService.saveToFile('');
      final result = await _documentService.exportToOoxml(documentJson);

      final bytes = Uint8List.fromList(result.codeUnits);
      final file = File(path);
      await file.writeAsBytes(bytes);

      return ExportResult.success(path);
    } catch (e) {
      return ExportResult.failure('Failed to export DOCX: $e');
    }
  }

  /// 导出为 HTML
  Future<ExportResult> _exportHtml(String path, String content) async {
    try {
      // 简单 HTML 包装
      final html = '''<!DOCTYPE html>
<html lang="zh-CN">
<head>
  <meta charset="UTF-8">
  <title>Exported Document</title>
  <style>
    body {
      font-family: Arial, sans-serif;
      font-size: 12pt;
      line-height: 1.5;
      margin: 1in;
    }
  </style>
</head>
<body>
<pre style="white-space: pre-wrap;">${_escapeHtml(content)}</pre>
</body>
</html>''';

      final file = File(path);
      await file.writeAsString(html);

      return ExportResult.success(path);
    } catch (e) {
      return ExportResult.failure('Failed to export HTML: $e');
    }
  }

  String _escapeHtml(String text) {
    return text
        .replaceAll('&', '&amp;')
        .replaceAll('<', '&lt;')
        .replaceAll('>', '&gt;')
        .replaceAll('"', '&quot;')
        .replaceAll("'", '&#39;');
  }

  // ==================== 文件信息 ====================

  /// 获取文件信息
  Future<FileInfo?> getFileInfo(String path) async {
    try {
      final file = File(path);
      final stat = await file.stat();

      final fileType = detectFileType(path);

      return FileInfo(
        path: path,
        name: file.uri.pathSegments.last,
        size: stat.size,
        created: stat.changed,
        modified: stat.modified,
        type: fileType,
        extension: path.split('.').last,
      );
    } catch (e) {
      return null;
    }
  }

  /// 检查文件是否存在
  Future<bool> fileExists(String path) async {
    return File(path).exists();
  }

  /// 获取文件字节
  Future<Uint8List?> getFileBytes(String path) async {
    try {
      final file = File(path);
      return await file.readAsBytes();
    } catch (e) {
      return null;
    }
  }
}

/// 文件信息
class FileInfo {
  final String path;
  final String name;
  final int size;
  final DateTime created;
  final DateTime modified;
  final FileType type;
  final String extension;

  const FileInfo({
    required this.path,
    required this.name,
    required this.size,
    required this.created,
    required this.modified,
    required this.type,
    required this.extension,
  });

  String get formattedSize {
    if (size < 1024) return '$size B';
    if (size < 1024 * 1024) return '${(size / 1024).toStringAsFixed(1)} KB';
    return '${(size / (1024 * 1024)).toStringAsFixed(1)} MB';
  }

  String get formattedModified {
    final formatter = DateFormat('yyyy-MM-dd HH:mm');
    return formatter.format(modified);
  }
}
