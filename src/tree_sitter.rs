// Minimal tree-sitter bindings for parsing and querying

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
    fn ts_node_start_byte(node: TSNode) -> u32;
    fn ts_node_end_byte(node: TSNode) -> u32;
    fn ts_node_child_count(node: TSNode) -> u32;
    fn ts_node_child(node: TSNode, index: u32) -> TSNode;
    fn ts_node_named_child_count(node: TSNode) -> u32;
    fn ts_node_named_child(node: TSNode, index: u32) -> TSNode;
    fn ts_node_type(node: TSNode) -> *const i8;
    fn ts_node_is_named(node: TSNode) -> bool;
    fn ts_query_new(
        language: *const c_void,
        source: *const u8,
        source_len: u32,
        error_offset: *mut u32,
        error_type: *mut u32,
    ) -> *mut c_void;
    fn ts_query_delete(query: *mut c_void);
    fn ts_query_cursor_new() -> *mut c_void;
    fn ts_query_cursor_delete(cursor: *mut c_void);
    fn ts_query_cursor_exec(cursor: *mut c_void, query: *const c_void, node: TSNode);
    fn ts_query_cursor_next_match(cursor: *mut c_void, match_: *mut TSQueryMatch) -> bool;
    fn ts_query_capture_name_for_id(query: *const c_void, id: u32, length: *mut u32) -> *const u8;
    fn free(ptr: *mut c_void);
}

#[repr(C)]
pub struct TSQueryMatch {
    pub id: u32,
    pub pattern_index: u16,
    pub capture_count: u16,
    pub captures: *const TSQueryCapture,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct TSQueryCapture {
    pub node: TSNode,
    pub index: u32,
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

    pub fn kind(&self) -> &str {
        unsafe {
            let c_str = ts_node_type(self.inner);
            std::ffi::CStr::from_ptr(c_str).to_str().unwrap_or("")
        }
    }

    pub fn start_byte(&self) -> u32 {
        unsafe { ts_node_start_byte(self.inner) }
    }

    pub fn end_byte(&self) -> u32 {
        unsafe { ts_node_end_byte(self.inner) }
    }

    pub fn child_count(&self) -> u32 {
        unsafe { ts_node_child_count(self.inner) }
    }

    pub fn child(&self, index: u32) -> Option<Node> {
        unsafe {
            let child = ts_node_child(self.inner, index);
            if child.id.is_null() {
                None
            } else {
                Some(Node { inner: child })
            }
        }
    }

    pub fn named_child_count(&self) -> u32 {
        unsafe { ts_node_named_child_count(self.inner) }
    }

    pub fn named_child(&self, index: u32) -> Option<Node> {
        unsafe {
            let child = ts_node_named_child(self.inner, index);
            if child.id.is_null() {
                None
            } else {
                Some(Node { inner: child })
            }
        }
    }

    pub fn is_named(&self) -> bool {
        unsafe { ts_node_is_named(self.inner) }
    }

    pub fn utf8_text<'a>(&self, source: &'a str) -> &'a str {
        let start = self.start_byte() as usize;
        let end = self.end_byte() as usize;
        &source[start..end]
    }

    pub(crate) fn inner(&self) -> TSNode {
        self.inner
    }
}

pub struct Query {
    ptr: *mut c_void,
    language: *const c_void,
}

impl Query {
    pub fn new(language: *const c_void, source: &str) -> Result<Self, String> {
        unsafe {
            let mut error_offset = 0u32;
            let mut error_type = 0u32;
            let ptr = ts_query_new(
                language,
                source.as_ptr(),
                source.len() as u32,
                &mut error_offset,
                &mut error_type,
            );
            if ptr.is_null() {
                Err(format!(
                    "Query error at offset {}: type {}",
                    error_offset, error_type
                ))
            } else {
                Ok(Query { ptr, language })
            }
        }
    }

    pub fn capture_name_for_id(&self, id: u32) -> String {
        unsafe {
            let mut length = 0u32;
            let name_ptr = ts_query_capture_name_for_id(self.ptr, id, &mut length);
            let bytes = std::slice::from_raw_parts(name_ptr, length as usize);
            String::from_utf8_lossy(bytes).to_string()
        }
    }
}

impl Drop for Query {
    fn drop(&mut self) {
        unsafe {
            ts_query_delete(self.ptr);
        }
    }
}

pub struct QueryCursor {
    ptr: *mut c_void,
}

impl QueryCursor {
    pub fn new() -> Self {
        unsafe {
            let ptr = ts_query_cursor_new();
            QueryCursor { ptr }
        }
    }

    pub fn exec(&mut self, query: &Query, node: &Node) {
        unsafe {
            ts_query_cursor_exec(self.ptr, query.ptr, node.inner());
        }
    }

    pub fn next_match(&mut self, query: &Query) -> Option<Vec<(String, Node)>> {
        unsafe {
            let mut ts_match = TSQueryMatch {
                id: 0,
                pattern_index: 0,
                capture_count: 0,
                captures: ptr::null(),
            };

            if ts_query_cursor_next_match(self.ptr, &mut ts_match) {
                let captures = std::slice::from_raw_parts(
                    ts_match.captures,
                    ts_match.capture_count as usize,
                );

                let result: Vec<(String, Node)> = captures
                    .iter()
                    .map(|cap| {
                        let name = query.capture_name_for_id(cap.index);
                        let node = Node { inner: cap.node };
                        (name, node)
                    })
                    .collect();

                Some(result)
            } else {
                None
            }
        }
    }
}

impl Drop for QueryCursor {
    fn drop(&mut self) {
        unsafe {
            ts_query_cursor_delete(self.ptr);
        }
    }
}
