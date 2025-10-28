// Salesforce Apex/SOQL/SOSL Tree-sitter bindings

use std::ffi::c_void;

// External C functions from the compiled grammars
extern "C" {
    fn tree_sitter_apex() -> *const c_void;
    fn tree_sitter_soql() -> *const c_void;
    fn tree_sitter_sosl() -> *const c_void;
}

pub mod apex {
    use super::*;

    pub fn language() -> *const c_void {
        unsafe { tree_sitter_apex() }
    }
}

pub mod soql {
    use super::*;

    pub fn language() -> *const c_void {
        unsafe { tree_sitter_soql() }
    }
}

pub mod sosl {
    use super::*;

    pub fn language() -> *const c_void {
        unsafe { tree_sitter_sosl() }
    }
}
