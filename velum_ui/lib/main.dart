import 'package:flutter/material.dart';
import 'dart:io';
import 'velum_core_wrapper.dart';
import 'text_sync_manager.dart';
import 'status_bar.dart';
import 'toolbar.dart';
import 'package:file_picker/file_picker.dart';

void main() {
  runApp(const MyApp());
}

class MyApp extends StatelessWidget {
  const MyApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Velum - Word 1:1 Replica',
      theme: ThemeData(
        colorScheme: ColorScheme.fromSeed(seedColor: Colors.blue),
        useMaterial3: true,
      ),
      home: const MyHomePage(title: 'Velum Word Processor'),
    );
  }
}

class MyHomePage extends StatefulWidget {
  const MyHomePage({super.key, required this.title});

  final String title;

  @override
  State<MyHomePage> createState() => _MyHomePageState();
}

class _MyHomePageState extends State<MyHomePage> {
  final TextEditingController _controller = TextEditingController();
  bool _isSaving = false;
  String _previousText = "";
  bool _canUndo = true;
  bool _canRedo = true;
  
  // TextSyncManager for optimized text sync
  late final TextSyncManager _syncManager;
  bool _isInitialized = false;
  
  // Metadata state
  int _wordCount = 0;
  int _charCount = 0;
  int _line = 1;
  int _column = 1;

  @override
  void initState() {
    super.initState();
    _syncManager = TextSyncManager(VelumCoreWrapper().api);
    _controller.addListener(_updateCursorPosition);
    _initializeDocument();
  }

  Future<void> _initializeDocument() async {
    final core = VelumCoreWrapper();
    final content = await core.api.createEmptyDocument();
    if (mounted) {
      setState(() {
        _controller.text = content;
        _previousText = content;
        _isInitialized = true;
      });
      _updateStats(content);
    }
  }

  @override
  void dispose() {
    _syncManager.flush();
    _syncManager.dispose();
    _controller.dispose();
    super.dispose();
  }

  void _updateCursorPosition() {
    final offset = _controller.selection.baseOffset;
    if (offset >= 0) {
      final text = _controller.text;
      // Calculate line and column from offset
      int line = 1;
      int column = 1;
      for (int i = 0; i < offset && i < text.length; i++) {
        if (text[i] == '\n') {
          line++;
          column = 1;
        } else {
          column++;
        }
      }
      setState(() {
        _line = line;
        _column = column;
      });
    }
  }

  void _updateStats([String? text]) {
    final content = text ?? _controller.text;
    // Count characters
    _charCount = content.length;
    // Count words (split by whitespace and filter empty)
    _wordCount = content.isEmpty ? 0 : content.split(RegExp(r'\s+')).where((w) => w.isNotEmpty).length;
  }

  Future<void> _onChanged(String newText) async {
    if (!_isInitialized) return;
    
    final oldText = _previousText;
    _previousText = newText;
    
    await _syncManager.onTextChanged(oldText, newText);
    
    if (mounted) {
      setState(() {
        _updateStats(newText);
        _updateCursorPosition();
      });
    }
  }

  Future<void> _syncFromCore() async {
    final core = VelumCoreWrapper();
    final content = await core.api.getFullText();
    
    if (mounted && _controller.text != content) {
      setState(() {
        final cursorPos = _controller.selection.base.offset;
        _controller.text = content;
        _previousText = content;
        
        // Restore cursor position
        if (cursorPos >= 0 && cursorPos <= content.length) {
          _controller.selection = TextSelection.collapsed(offset: cursorPos);
        }
      });
      _updateStats(content);
      _updateCursorPosition();
    }
  }

  Future<void> _performUndo() async {
    final core = VelumCoreWrapper();
    await core.api.undo();
    await _syncFromCore();
  }

  Future<void> _performRedo() async {
    final core = VelumCoreWrapper();
    await core.api.redo();
    await _syncFromCore();
  }

  Future<void> _saveFile() async {
    setState(() => _isSaving = true);
    try {
      String? outputFile = await FilePicker.platform.saveFile(
        dialogTitle: 'Save Document',
        fileName: 'untitled.vlm',
        allowedExtensions: ['vlm', 'json'],
        lockParentWindow: true,
      );
      
      if (outputFile != null) {
        if (!outputFile.endsWith('.vlm') && !outputFile.endsWith('.json')) {
          outputFile = '$outputFile.vlm';
        }
        final core = VelumCoreWrapper();
        final content = await core.api.getFullText();
        final file = File(outputFile);
        await file.writeAsString(content);
        if (mounted) {
          ScaffoldMessenger.of(context).showSnackBar(
            SnackBar(content: Text('Document saved to $outputFile')),
          );
        }
      }
    } finally {
      if (mounted) setState(() => _isSaving = false);
    }
  }

  Future<void> _openFile() async {
    try {
      FilePickerResult? result = await FilePicker.platform.pickFiles(
        dialogTitle: 'Open Document',
        allowedExtensions: ['vlm', 'json'],
        lockParentWindow: true,
      );
      
      if (result != null) {
        final file = File(result.files.single.path!);
        final content = await file.readAsString();
        setState(() {
          _controller.text = content;
          _previousText = content;
        });
        _updateStats();
        if (mounted) {
          ScaffoldMessenger.of(context).showSnackBar(
            const SnackBar(content: Text('File opened successfully')),
          );
        }
      }
    } catch (e) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text('Error opening file: $e')),
        );
      }
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: VelumToolbar(
        onUndo: _performUndo,
        onRedo: _performRedo,
        onSave: _saveFile,
        onOpen: _openFile,
        canUndo: _canUndo,
        canRedo: _canRedo,
        isSaving: _isSaving,
      ),
      body: Column(
        children: [
          Expanded(
            child: Padding(
              padding: const EdgeInsets.all(16.0),
              child: Container(
                decoration: BoxDecoration(
                  border: Border.all(color: Colors.grey.shade300),
                  borderRadius: BorderRadius.circular(8),
                ),
                child: TextField(
                  controller: _controller,
                  maxLines: null,
                  expands: true,
                  onChanged: _onChanged,
                  decoration: const InputDecoration(
                    hintText: 'Start typing...',
                    contentPadding: EdgeInsets.all(12),
                    border: InputBorder.none,
                  ),
                  style: const TextStyle(fontFamily: 'Courier', fontSize: 16),
                ),
              ),
            ),
          ),
        ],
      ),
      bottomNavigationBar: StatusBar(
        wordCount: _wordCount,
        charCount: _charCount,
        line: _line,
        column: _column,
      ),
    );
  }
}
