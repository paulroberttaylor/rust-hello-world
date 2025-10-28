// Minimal tree-sitter bindings for parsing

use std::ffi::c_void;
use std::ptr;

// External C functions from libtree-sitter.so
extern "C" {
    fn ts_parser_new() -> *mut c_void;
    fn ts_parser_delete(parser: *mut c_void);
    fn ts_parser_set_language(parser: *mut c_void, language: *const c_void) -> bool;
    fn ts_parser_parse_string(
        parser: *mut c_void,
        old_tree: *const c_void,
        string: *const u8,
        length: u32,
    ) -> *mut c_void;
    fn ts_tree_delete(tree: *mut c_void);
    fn ts_tree_root_node(tree: *const c_void) -> TSNode;
    fn ts_node_string(node: TSNode) -> *mut i8;
    fn free(ptr: *mut c_void);
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct TSNode {
    context: [u32; 4],
    id: *const c_void,
    tree: *const c_void,
}

pub struct Parser {
    ptr: *mut c_void,
}

pub struct Tree {
    ptr: *mut c_void,
}

pub struct Node {
    inner: TSNode,
}

impl Parser {
    pub fn new() -> Self {
        unsafe {
            let ptr = ts_parser_new();
            Parser { ptr }
        }
    }

    pub fn set_language(&mut self, language: *const c_void) -> Result<(), String> {
        unsafe {
            if ts_parser_set_language(self.ptr, language) {
                Ok(())
            } else {
                Err("Failed to set language".to_string())
            }
        }
    }

    pub fn parse(&mut self, source: &str) -> Option<Tree> {
        unsafe {
            let ptr = ts_parser_parse_string(
                self.ptr,
                ptr::null(),
                source.as_ptr(),
                source.len() as u32,
            );
            if ptr.is_null() {
                None
            } else {
                Some(Tree { ptr })
            }
        }
    }
}

impl Drop for Parser {
    fn drop(&mut self) {
        unsafe {
            ts_parser_delete(self.ptr);
        }
    }
}

impl Tree {
    pub fn root_node(&self) -> Node {
        unsafe {
            let inner = ts_tree_root_node(self.ptr);
            Node { inner }
        }
    }
}

impl Drop for Tree {
    fn drop(&mut self) {
        unsafe {
            ts_tree_delete(self.ptr);
        }
    }
}

impl Node {
    pub fn to_sexp(&self) -> String {
        unsafe {
            let c_str = ts_node_string(self.inner);
            let rust_str = std::ffi::CStr::from_ptr(c_str)
                .to_string_lossy()
                .into_owned();
            free(c_str as *mut c_void);
            rust_str
        }
    }
}
