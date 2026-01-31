import 'package:flutter/material.dart';
import 'velum_core_wrapper.dart';

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
  bool _isUpdatingFromCore = false;
  String _previousText = "";
  
  // Metadata state
  int _wordCount = 0;
  int _charCount = 0;
  int _line = 1;
  int _column = 1;

  @override
  void initState() {
    super.initState();
    _initDocument();
    _controller.addListener(_updateCursorPosition);
  }

  @override
  void dispose() {
    _controller.removeListener(_updateCursorPosition);
    _controller.dispose();
    super.dispose();
  }

  void _updateCursorPosition() async {
    final offset = _controller.selection.baseOffset;
    if (offset >= 0) {
      final core = VelumCoreWrapper();
      final pos = await core.api.getCursorPosition(charOffset: offset);
      setState(() {
        _line = pos.field0;
        _column = pos.field1;
      });
    }
  }

  Future<void> _updateMetadata() async {
    final core = VelumCoreWrapper();
    final wc = await core.api.getWordCount();
    final cc = await core.api.getCharCount();
    setState(() {
      _wordCount = wc;
      _charCount = cc;
    });
  }

  Future<void> _initDocument() async {
    final core = VelumCoreWrapper();
    final content = await core.api.createEmptyDocument();
    setState(() {
      _isUpdatingFromCore = true;
      _controller.text = content;
      _previousText = content;
      _isUpdatingFromCore = false;
    });
    await _updateMetadata();
  }

  Future<void> _handleTextChanged(String text) async {
    if (_isUpdatingFromCore) return;

    final core = VelumCoreWrapper();
    // For a basic interactive editor, we'll just sync the whole text for now
    // In a real app, we'd use delta updates with insert_text/delete_text
    // But since we need to use the new API:
    
    // Simple heuristic: if text is longer, it's an insertion, if shorter, deletion.
    // However, for this task, let's just replace the whole content to keep it robust
    // but the instructions say "Connect the editor's changes to the Rust insert_text and delete_text functions".
    
    // To properly use insert/delete, we need to track the previous state.
  }

  // Improved sync logic
  String _previousText = "";

  Future<void> _onChanged(String currentText) async {
    if (_isUpdatingFromCore) {
      _previousText = currentText;
      return;
    }

    final core = VelumCoreWrapper();
    
    // Find the first difference from the start
    int start = 0;
    while (start < _previousText.length && start < currentText.length && _previousText[start] == currentText[start]) {
      start++;
    }

    // Find the first difference from the end
    int oldEnd = _previousText.length;
    int newEnd = currentText.length;
    while (oldEnd > start && newEnd > start && _previousText[oldEnd - 1] == currentText[newEnd - 1]) {
      oldEnd--;
      newEnd--;
    }

    // If oldEnd > start, something was deleted
    if (oldEnd > start) {
      await core.api.deleteText(offset: start, length: oldEnd - start);
    }

    // If newEnd > start, something was inserted
    if (newEnd > start) {
      String inserted = currentText.substring(start, newEnd);
      await core.api.insertText(offset: start, newText: inserted);
    }

    _previousText = currentText;
    await _updateMetadata();
  }

  Future<void> _saveDocument() async {
    final core = VelumCoreWrapper();
    // For now, save to a default location or we could use a file picker
    final result = await core.api.saveToFile(path: 'document.vlm');
    if (mounted) {
      ScaffoldMessenger.of(context).showSnackBar(SnackBar(content: Text(result)));
    }
  }

  Future<void> _undo() async {
    final core = VelumCoreWrapper();
    final newText = await core.api.undo();
    setState(() {
      _isUpdatingFromCore = true;
      _controller.text = newText;
      _previousText = newText;
      _isUpdatingFromCore = false;
    });
  }

  Future<void> _redo() async {
    final core = VelumCoreWrapper();
    final newText = await core.api.redo();
    setState(() {
      _isUpdatingFromCore = true;
      _controller.text = newText;
      _previousText = newText;
      _isUpdatingFromCore = false;
    });
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        backgroundColor: Theme.of(context).colorScheme.inversePrimary,
        title: Text(widget.title),
        actions: [
          IconButton(
            icon: const Icon(Icons.save),
            onPressed: _saveDocument,
            tooltip: 'Save',
          ),
          const VerticalDivider(width: 1, indent: 10, endIndent: 10),
          IconButton(
            icon: const Icon(Icons.undo),
            onPressed: _undo,
            tooltip: 'Undo',
          ),
          IconButton(
            icon: const Icon(Icons.redo),
            onPressed: _redo,
            tooltip: 'Redo',
          ),
        ],
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
          // Status Bar
          Container(
            padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 4),
            color: Colors.grey.shade200,
            child: Row(
              children: [
                Text('Line: $_line, Col: $_column'),
                const SizedBox(width: 20),
                Text('Words: $_wordCount'),
                const SizedBox(width: 20),
                Text('Chars: $_charCount'),
                const Spacer(),
                const Text('UTF-8'),
              ],
            ),
          ),
        ],
      ),
    );
  }
}
