import 'package:flutter/material.dart';

class StatusBar extends StatelessWidget {
  final int wordCount;
  final int charCount;
  final int line;
  final int column;
  
  const StatusBar({
    super.key,
    required this.wordCount,
    required this.charCount,
    required this.line,
    required this.column,
  });
  
  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
      decoration: BoxDecoration(
        color: Theme.of(context).brightness == Brightness.dark
            ? Colors.grey[900]
            : Colors.grey[100],
        border: Border(
          top: BorderSide(
            color: Theme.of(context).brightness == Brightness.dark
                ? Colors.grey[700]!
                : Colors.grey[300]!,
          ),
        ),
      ),
      child: Row(
        mainAxisAlignment: MainAxisAlignment.end,
        children: [
          _StatusItem(label: 'Lines', value: line.toString()),
          const SizedBox(width: 24),
          _StatusItem(label: 'Cols', value: column.toString()),
          const SizedBox(width: 24),
          _StatusItem(label: 'Words', value: wordCount.toString()),
          const SizedBox(width: 24),
          _StatusItem(label: 'Chars', value: charCount.toString()),
        ],
      ),
    );
  }
}

class _StatusItem extends StatelessWidget {
  final String label;
  final String value;
  
  const _StatusItem({required this.label, required this.value});
  
  @override
  Widget build(BuildContext context) {
    return Text.rich(
      TextSpan(
        children: [
          TextSpan(
            text: '$label: ',
            style: TextStyle(
              color: Theme.of(context).brightness == Brightness.dark
                  ? Colors.grey[400]
                  : Colors.grey[600],
              fontSize: 12,
            ),
          ),
          TextSpan(
            text: value,
            style: TextStyle(
              color: Theme.of(context).brightness == Brightness.dark
                  ? Colors.white
                  : Colors.black87,
              fontSize: 12,
              fontWeight: FontWeight.w500,
            ),
          ),
        ],
      ),
    );
  }
}
