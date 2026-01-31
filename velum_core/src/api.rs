use crate::piece_tree::PieceTree;

pub fn hello_velum() -> String {
    "Hello from Velum Core (Rust)!".to_string()
}

pub fn get_sample_document() -> String {
    let mut pt = PieceTree::new("Welcome to Velum.".to_string());
    pt.insert(16, " This is Microsoft Word 1:1 replica project.".to_string());
    pt.get_text()
}

pub fn multiply(a: i32, b: i32) -> i32 {
    a * b
}
