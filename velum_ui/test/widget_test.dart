// This is a basic Flutter widget test for Velum.
//
// To perform an interaction with a widget in your test, use the WidgetTester
// utility in the flutter_test package.

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:velum_ui/main.dart';

void main() {
  testWidgets('Velum app smoke test', (WidgetTester tester) async {
    // Build our app and trigger a frame.
    await tester.pumpWidget(const MyApp());

    // Verify that our app title is displayed
    expect(find.text('Velum Word Processor'), findsOneWidget);

    // Verify that the text field is present
    expect(find.byType(TextField), findsOneWidget);

    // Verify that undo/redo buttons are present
    expect(find.byIcon(Icons.undo), findsOneWidget);
    expect(find.byIcon(Icons.redo), findsOneWidget);
  });
}
