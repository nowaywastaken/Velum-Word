import 'package:flutter/material.dart';

class TextLayoutPainter extends CustomPainter {
  final String text;
  final TextStyle style;
  final double width;

  TextLayoutPainter({
    required this.text,
    required this.style,
    required this.width,
  });

  @override
  void paint(Canvas canvas, Size size) {
    // This is a prototype of manual text painting.
    // In the future, this will consume the layout data from the Rust core.
    
    final textSpan = TextSpan(
      text: text,
      style: style,
    );
    
    final textPainter = TextPainter(
      text: textSpan,
      textDirection: TextDirection.ltr,
    );
    
    textPainter.layout(
      minWidth: 0,
      maxWidth: width,
    );
    
    // Manual painting allows us to intervene in the rendering process
    // e.g. drawing custom selections, carets, or debug overlays
    textPainter.paint(canvas, Offset.zero);
    
    // Example: Draw a debug border around the text
    final paint = Paint()
      ..color = Colors.red.withOpacity(0.3)
      ..style = PaintingStyle.stroke
      ..strokeWidth = 1.0;
      
    canvas.drawRect(
      Rect.fromLTWH(0, 0, textPainter.width, textPainter.height),
      paint,
    );
  }

  @override
  bool shouldRepaint(covariant CustomPainter oldDelegate) {
    return true;
  }
}

class VelumProtoRenderer extends StatelessWidget {
  final String text;
  
  const VelumProtoRenderer({super.key, required this.text});

  @override
  Widget build(BuildContext context) {
    return LayoutBuilder(
      builder: (context, constraints) {
        return CustomPaint(
          painter: TextLayoutPainter(
            text: text,
            style: const TextStyle(fontSize: 16, color: Colors.black, fontFamily: 'Courier'),
            width: constraints.maxWidth,
          ),
          size: Size.infinite,
        );
      },
    );
  }
}
