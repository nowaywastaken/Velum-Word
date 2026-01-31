import 'package:flutter/material.dart';

class VelumToolbar extends StatelessWidget implements PreferredSizeWidget {
  final VoidCallback onUndo;
  final VoidCallback onRedo;
  final VoidCallback onSave;
  final VoidCallback onOpen;
  final bool canUndo;
  final bool canRedo;
  final bool isSaving;
  
  const VelumToolbar({
    super.key,
    required this.onUndo,
    required this.onRedo,
    required this.onSave,
    required this.onOpen,
    this.canUndo = true,
    this.canRedo = true,
    this.isSaving = false,
  });
  
  @override
  Size get preferredSize => const Size.fromHeight(kToolbarHeight);
  
  @override
  Widget build(BuildContext context) {
    return AppBar(
      title: const Text('Velum'),
      actions: [
        _ToolbarButton(
          icon: Icons.undo,
          tooltip: 'Undo (Cmd+Z)',
          onPressed: canUndo ? onUndo : null,
        ),
        _ToolbarButton(
          icon: Icons.redo,
          tooltip: 'Redo (Cmd+Shift+Z)',
          onPressed: canRedo ? onRedo : null,
        ),
        const SizedBox(width: 8),
        _ToolbarButton(
          icon: isSaving ? Icons.save_alt : Icons.save,
          tooltip: 'Save (Cmd+S)',
          onPressed: onSave,
        ),
        _ToolbarButton(
          icon: Icons.folder_open,
          tooltip: 'Open File',
          onPressed: onOpen,
        ),
      ],
    );
  }
}

class _ToolbarButton extends StatelessWidget {
  final IconData icon;
  final String tooltip;
  final VoidCallback? onPressed;
  
  const _ToolbarButton({
    required this.icon,
    required this.tooltip,
    required this.onPressed,
  });
  
  @override
  Widget build(BuildContext context) {
    return IconButton(
      icon: Icon(icon),
      tooltip: tooltip,
      onPressed: onPressed,
      color: onPressed != null
          ? Theme.of(context).brightness == Brightness.dark
              ? Colors.white
              : Colors.black87
          : Colors.grey[400],
    );
  }
}
