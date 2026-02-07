//! # Keyboard Shortcut Support
//!
//! Provides keyboard shortcut definitions and handling support.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Virtual key codes
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VirtualKey {
    /// A-Z
    A, B, C, D, E, F, G, H, I, J, K, L, M,
    N, O, P, Q, R, S, T, U, V, W, X, Y, Z,
    /// 0-9
    Key0, Key1, Key2, Key3, Key4, Key5, Key6, Key7, Key8, Key9,
    /// Function keys
    F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12,
    /// Navigation
    Left, Right, Up, Down,
    Home, End, PageUp, PageDown,
    /// Editing
    Insert, Delete, Backspace, Tab, Enter, Escape,
    /// Special
    Space,
    Minus,      // -/_
    Equal,      // =/+
    /// Numpad
    Numpad0, Numpad1, Numpad2, Numpad3, Numpad4,
    Numpad5, Numpad6, Numpad7, Numpad8, Numpad9,
    NumpadMultiply, NumpadAdd, NumpadSubtract,
    NumpadDecimal, NumpadDivide,
    /// Unknown
    Unknown(u32),
}

impl Default for VirtualKey {
    fn default() -> Self {
        VirtualKey::Unknown(0)
    }
}

/// Modifier keys
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Modifiers {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub meta: bool, // Command on Mac, Win on Windows
}

impl Modifiers {
    /// Creates empty modifiers
    pub fn none() -> Self {
        Modifiers {
            ctrl: false,
            alt: false,
            shift: false,
            meta: false,
        }
    }

    /// Creates a single modifier
    pub fn ctrl() -> Self {
        Modifiers {
            ctrl: true,
            alt: false,
            shift: false,
            meta: false,
        }
    }

    /// Creates shift modifier
    pub fn shift() -> Self {
        Modifiers {
            ctrl: false,
            alt: false,
            shift: true,
            meta: false,
        }
    }

    /// Creates alt modifier
    pub fn alt() -> Self {
        Modifiers {
            ctrl: false,
            alt: true,
            shift: false,
            meta: false,
        }
    }

    /// Creates meta modifier (Command on Mac)
    pub fn meta() -> Self {
        Modifiers {
            ctrl: false,
            alt: false,
            shift: false,
            meta: true,
        }
    }

    /// Creates ctrl+shift modifier
    pub fn ctrl_shift() -> Self {
        Modifiers {
            ctrl: true,
            alt: false,
            shift: true,
            meta: false,
        }
    }

    /// Creates ctrl+alt modifier
    pub fn ctrl_alt() -> Self {
        Modifiers {
            ctrl: true,
            alt: true,
            shift: false,
            meta: false,
        }
    }

    /// Returns true if any modifier is pressed
    pub fn any(&self) -> bool {
        self.ctrl || self.alt || self.shift || self.meta
    }
}

/// Keyboard shortcut definition
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Shortcut {
    pub key: VirtualKey,
    pub modifiers: Modifiers,
    /// Command to execute
    pub command: CommandId,
    /// Display name for UI
    pub display_name: String,
    /// Category for menu organization
    pub category: Category,
    /// Whether this is a default shortcut
    pub is_default: bool,
}

impl Shortcut {
    /// Creates a new shortcut
    pub fn new(key: VirtualKey, modifiers: Modifiers, command: CommandId, display_name: impl Into<String>) -> Self {
        Shortcut {
            key,
            modifiers,
            command,
            display_name: display_name.into(),
            category: Category::Default,
            is_default: true,
        }
    }

    /// Returns the shortcut as a string (e.g., "Ctrl+B")
    pub fn to_string(&self) -> String {
        let mut parts = Vec::new();

        if self.modifiers.ctrl || self.modifiers.meta {
            #[cfg(target_os = "macos")]
            parts.push("Cmd");
            #[cfg(not(target_os = "macos"))]
            parts.push("Ctrl");
        }
        if self.modifiers.alt {
            parts.push("Alt");
        }
        if self.modifiers.shift {
            parts.push("Shift");
        }

        parts.push(self.key_name());

        parts.join("+")
    }

    /// Gets the key name
    fn key_name(&self) -> &str {
        match self.key {
            VirtualKey::A => "A",
            VirtualKey::B => "B",
            VirtualKey::C => "C",
            VirtualKey::D => "D",
            VirtualKey::E => "E",
            VirtualKey::F => "F",
            VirtualKey::G => "G",
            VirtualKey::H => "H",
            VirtualKey::I => "I",
            VirtualKey::J => "J",
            VirtualKey::K => "K",
            VirtualKey::L => "L",
            VirtualKey::M => "M",
            VirtualKey::N => "N",
            VirtualKey::O => "O",
            VirtualKey::P => "P",
            VirtualKey::Q => "Q",
            VirtualKey::R => "R",
            VirtualKey::S => "S",
            VirtualKey::T => "T",
            VirtualKey::U => "U",
            VirtualKey::V => "V",
            VirtualKey::W => "W",
            VirtualKey::X => "X",
            VirtualKey::Y => "Y",
            VirtualKey::Z => "Z",
            VirtualKey::Key0 => "0",
            VirtualKey::Key1 => "1",
            VirtualKey::Key2 => "2",
            VirtualKey::Key3 => "3",
            VirtualKey::Key4 => "4",
            VirtualKey::Key5 => "5",
            VirtualKey::Key6 => "6",
            VirtualKey::Key7 => "7",
            VirtualKey::Key8 => "8",
            VirtualKey::Key9 => "9",
            VirtualKey::F1 => "F1",
            VirtualKey::F2 => "F2",
            VirtualKey::F3 => "F3",
            VirtualKey::F4 => "F4",
            VirtualKey::F5 => "F5",
            VirtualKey::F6 => "F6",
            VirtualKey::F7 => "F7",
            VirtualKey::F8 => "F8",
            VirtualKey::F9 => "F9",
            VirtualKey::F10 => "F10",
            VirtualKey::F11 => "F11",
            VirtualKey::F12 => "F12",
            VirtualKey::Left => "Left",
            VirtualKey::Right => "Right",
            VirtualKey::Up => "Up",
            VirtualKey::Down => "Down",
            VirtualKey::Home => "Home",
            VirtualKey::End => "End",
            VirtualKey::PageUp => "PageUp",
            VirtualKey::PageDown => "PageDown",
            VirtualKey::Insert => "Insert",
            VirtualKey::Delete => "Delete",
            VirtualKey::Backspace => "Backspace",
            VirtualKey::Tab => "Tab",
            VirtualKey::Enter => "Enter",
            VirtualKey::Escape => "Escape",
            VirtualKey::Space => "Space",
            _ => "Unknown",
        }
    }
}

/// Command IDs for editor commands
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CommandId {
    // File commands
    New,
    Open,
    Save,
    SaveAs,
    Print,
    Close,

    // Edit commands
    Undo,
    Redo,
    Cut,
    Copy,
    Paste,
    SelectAll,
    Find,
    Replace,
    GoTo,

    // Format commands
    Bold,
    Italic,
    Underline,
    StrikeThrough,
    Subscript,
    Superscript,
    FontSize,
    FontName,
    IncreaseFontSize,
    DecreaseFontSize,
    ClearFormatting,
    AlignLeft,
    AlignCenter,
    AlignRight,
    AlignJustify,
    IncreaseIndent,
    DecreaseIndent,
    ParagraphSpacing,
    Bullets,
    Numbering,

    // Paragraph commands
    Heading1,
    Heading2,
    Heading3,
    Normal,
    LineSpacingSingle,
    LineSpacing15,
    LineSpacingDouble,

    // Navigation commands
    MoveLeft,
    MoveRight,
    MoveUp,
    MoveDown,
    MoveToLineStart,
    MoveToLineEnd,
    MoveToDocumentStart,
    MoveToDocumentEnd,
    NextPage,
    PreviousPage,

    // Selection commands
    SelectLeft,
    SelectRight,
    SelectUp,
    SelectDown,
    SelectToLineStart,
    SelectToLineEnd,
    SelectToDocumentStart,
    SelectToDocumentEnd,
    SelectWord,
    SelectParagraph,

    // Table commands (placeholders)
    TableInsertRow,
    TableInsertColumn,
    TableDeleteRow,
    TableDeleteColumn,
    TableMergeCells,
    TableSplitCell,

    // Custom
    Custom(String),
}

/// Menu category for organizing shortcuts
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Category {
    File,
    Edit,
    View,
    Insert,
    Format,
    Table,
    Tools,
    Window,
    Help,
    Default,
}

/// Shortcut map containing all keyboard shortcuts
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct ShortcutMap {
    /// Map from (key, modifiers) to command
    shortcuts: HashMap<(VirtualKey, Modifiers), CommandId>,
    /// Map from command ID to shortcut
    commands: HashMap<CommandId, Shortcut>,
    /// Platform-specific adjustments
    pub platform: PlatformConfig,
}

impl ShortcutMap {
    /// Creates a new empty shortcut map
    pub fn new() -> Self {
        let mut map = ShortcutMap {
            shortcuts: HashMap::new(),
            commands: HashMap::new(),
            platform: PlatformConfig::default(),
        };

        // Register default shortcuts
        map.register_defaults();

        map
    }

    /// Registers a shortcut
    pub fn register(&mut self, shortcut: Shortcut) {
        self.shortcuts.insert((shortcut.key, shortcut.modifiers), shortcut.command);
        self.commands.insert(shortcut.command, shortcut);
    }

    /// Gets the command for a key combination
    pub fn get_command(&self, key: VirtualKey, modifiers: Modifiers) -> Option<&CommandId> {
        self.shortcuts.get(&(key, modifiers))
    }

    /// Gets the shortcut for a command
    pub fn get_shortcut(&self, command: CommandId) -> Option<&Shortcut> {
        self.commands.get(&command)
    }

    /// Converts a key event to a command
    pub fn handle_key_event(&self, key: VirtualKey, modifiers: Modifiers) -> Option<&CommandId> {
        self.get_command(key, modifiers)
    }

    /// Registers all default shortcuts
    fn register_defaults(&mut self) {
        // File shortcuts
        self.register(Shortcut::new(VirtualKey::N, Modifiers::ctrl(), CommandId::New, "New"));
        self.register(Shortcut::new(VirtualKey::O, Modifiers::ctrl(), CommandId::Open, "Open"));
        self.register(Shortcut::new(VirtualKey::S, Modifiers::ctrl(), CommandId::Save, "Save"));
        self.register(Shortcut::new(VirtualKey::S, Modifiers::ctrl_shift(), CommandId::SaveAs, "Save As"));
        self.register(Shortcut::new(VirtualKey::P, Modifiers::ctrl(), CommandId::Print, "Print"));

        // Edit shortcuts
        self.register(Shortcut::new(VirtualKey::Z, Modifiers::ctrl(), CommandId::Undo, "Undo"));
        self.register(Shortcut::new(VirtualKey::Y, Modifiers::ctrl(), CommandId::Redo, "Redo"));
        self.register(Shortcut::new(VirtualKey::X, Modifiers::ctrl(), CommandId::Cut, "Cut"));
        self.register(Shortcut::new(VirtualKey::C, Modifiers::ctrl(), CommandId::Copy, "Copy"));
        self.register(Shortcut::new(VirtualKey::V, Modifiers::ctrl(), CommandId::Paste, "Paste"));
        self.register(Shortcut::new(VirtualKey::A, Modifiers::ctrl(), CommandId::SelectAll, "Select All"));
        self.register(Shortcut::new(VirtualKey::F, Modifiers::ctrl(), CommandId::Find, "Find"));
        self.register(Shortcut::new(VirtualKey::H, Modifiers::ctrl(), CommandId::Replace, "Replace"));

        // Format shortcuts
        self.register(Shortcut::new(VirtualKey::B, Modifiers::ctrl(), CommandId::Bold, "Bold"));
        self.register(Shortcut::new(VirtualKey::I, Modifiers::ctrl(), CommandId::Italic, "Italic"));
        self.register(Shortcut::new(VirtualKey::U, Modifiers::ctrl(), CommandId::Underline, "Underline"));
        self.register(Shortcut::new(VirtualKey::D, Modifiers::ctrl(), CommandId::StrikeThrough, "Strikethrough"));
        self.register(Shortcut::new(VirtualKey::Equal, Modifiers::ctrl(), CommandId::Superscript, "Superscript"));
        self.register(Shortcut::new(VirtualKey::Equal, Modifiers::ctrl_shift(), CommandId::Subscript, "Subscript"));

        // Paragraph shortcuts
        self.register(Shortcut::new(VirtualKey::Key1, Modifiers::ctrl(), CommandId::Heading1, "Heading 1"));
        self.register(Shortcut::new(VirtualKey::Key2, Modifiers::ctrl(), CommandId::Heading2, "Heading 2"));
        self.register(Shortcut::new(VirtualKey::Key3, Modifiers::ctrl(), CommandId::Heading3, "Heading 3"));
        self.register(Shortcut::new(VirtualKey::Key0, Modifiers::ctrl(), CommandId::Normal, "Normal"));

        // Navigation shortcuts
        self.register(Shortcut::new(VirtualKey::Home, Modifiers::none(), CommandId::MoveToLineStart, "Line Start"));
        self.register(Shortcut::new(VirtualKey::End, Modifiers::none(), CommandId::MoveToLineEnd, "Line End"));
        self.register(Shortcut::new(VirtualKey::Home, Modifiers::ctrl(), CommandId::MoveToDocumentStart, "Document Start"));
        self.register(Shortcut::new(VirtualKey::End, Modifiers::ctrl(), CommandId::MoveToDocumentEnd, "Document End"));

        // Selection shortcuts
        self.register(Shortcut::new(VirtualKey::A, Modifiers::ctrl_shift(), CommandId::SelectAll, "Select All"));
    }

    /// Gets all registered shortcuts
    pub fn all_shortcuts(&self) -> Vec<&Shortcut> {
        self.commands.values().collect()
    }

    /// Gets shortcuts by category
    pub fn shortcuts_by_category(&self, category: Category) -> Vec<&Shortcut> {
        self.commands
            .values()
            .filter(|s| s.category == category)
            .collect()
    }
}

/// Platform-specific configuration
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct PlatformConfig {
    /// Use Command key instead of Ctrl on macOS
    pub macos_uses_cmd: bool,
    /// Use Escape for cancel
    pub escape_cancels: bool,
    /// Enable native shortcuts
    pub enable_native_shortcuts: bool,
}

impl Default for PlatformConfig {
    fn default() -> Self {
        #[cfg(target_os = "macos")]
        let macos_uses_cmd = true;
        #[cfg(not(target_os = "macos"))]
        let macos_uses_cmd = false;

        PlatformConfig {
            macos_uses_cmd,
            escape_cancels: true,
            enable_native_shortcuts: true,
        }
    }
}

// Add missing VirtualKey variant
impl VirtualKey {
    #[cfg(not(target_os = "macos"))]
    fn equal_key() -> Self {
        VirtualKey::Unknown(0xBB) // VK_OEM_PLUS or VK_ADD
    }
}
