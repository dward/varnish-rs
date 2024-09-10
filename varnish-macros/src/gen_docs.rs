//! Code to generate automated documentation into a file

use std::fmt::Write;
use std::fs;
use std::path::Path;

use crate::model::{FuncInfo, FuncType, ParamInfo, ParamType, ParamTypeInfo, VmodInfo};

// Small helpers to write to a string without checking the result

macro_rules! wrt {
    ($docs:expr, $($arg:tt)*) => {
        let _ = write!($docs, $($arg)*);
    };
}
macro_rules! ln {
    ($docs:expr, $($arg:tt)*) => {
        let _ = writeln!($docs, $($arg)*);
    };
}

/// Generate documentation for the VMOD and save it to a file
pub fn generate_docs(info: &VmodInfo) {
    let Some(ref doc_file) = info.params.docs else {
        return; // doc file is not set, skipping
    };
    let Ok(dir) = std::env::var("CARGO_MANIFEST_DIR") else {
        panic!("Unable to get the CARGO_MANIFEST_DIR env var to save documentation, you may need to remove the `docs` parameter from the `#[vmod]` attribute");
    };
    let doc_file = Path::new(&dir).join(doc_file);
    let docs = gen_doc_content(info);
    if let Err(e) = fs::write(doc_file.as_path(), docs) {
        panic!(
            "Unable to save documentation to file {}: {e}",
            doc_file.display()
        );
    }
}

/// Generate documentation for the VMOD as a single string
pub fn gen_doc_content(info: &VmodInfo) -> String {
    let mut docs = String::new();

    // Initial warning to not edit this file
    ln!(
        docs,
        r#"<!--

   !!!!!!  WARNING: DO NOT EDIT THIS FILE!

   This file was generated from the Varnish VMOD source code.
   It will be automatically updated on each build.

-->"#
    );

    // generate documentation
    ln!(docs, "# Varnish Module (VMOD) `{}`", info.ident);
    write_docs(&mut docs, &info.docs, "#");
    ln!(
        docs,
        r#"
```vcl
// Place import statement at the top of your VCL file
// This loads vmod from a standard location
import {ident};

// Or load vmod from a specific file
import {ident} from "path/to/lib{ident}.so";
```"#,
        ident = info.ident
    );

    for func in &info.funcs {
        if !matches!(func.func_type, FuncType::Function) {
            continue;
        }
        write_function(&mut docs, "###", "Function", func);
    }

    for obj in &info.objects {
        ln!(docs, "\n### Object `{}`", obj.ident);

        write_docs(&mut docs, &obj.docs, "###");
        write_function(&mut docs, "####", &obj.ident, &obj.constructor);
        for method in &obj.funcs {
            write_function(&mut docs, "####", "Method", method);
        }
    }

    docs
}

fn write_function(mut docs: &mut String, prefix: &str, obj_or_typ: &str, func: &FuncInfo) {
    let user_args = get_user_args(func);

    if matches!(func.func_type, FuncType::Constructor) {
        ln!(
            docs,
            r#"
```vcl
// Create a new instance of the object in your VCL init function
sub vcl_init {{
    new {ident} = {obj_or_typ}.{sig};
}}
```"#,
            ident = func.ident,
            sig = fn_sig(func, &user_args),
        );
    } else {
        wrt!(docs, "\n{prefix} {obj_or_typ} ");
        ln!(&mut docs, "{}", fn_sig(func, &user_args));
    }

    write_docs(docs, &func.docs, prefix);

    // List of arguments are printed only if any of them have documentation
    if user_args.iter().any(|(arg, _)| !arg.docs.is_empty()) {
        ln!(docs, "");
        for (arg, ty) in &user_args {
            wrt!(docs, "* `{}`:", bracketed_name(arg, ty));
            if arg.docs.is_empty() {
                ln!(docs, "");
            } else {
                write_docs(docs, &arg.docs, prefix);
            }
        }
    }
}

fn fn_sig(func: &FuncInfo, user_args: &Vec<(&ParamTypeInfo, &ParamInfo)>) -> String {
    let mut res = String::new();
    let is_md_txt = matches!(
        func.func_type,
        FuncType::Function | FuncType::Method | FuncType::Event
    );
    if is_md_txt {
        wrt!(res, "`");
    }
    if matches!(func.func_type, FuncType::Function | FuncType::Method) {
        let ret = func.returns.value_type().to_vcc_type();
        wrt!(res, "{ret} ");
    }
    wrt!(res, "{}(", func.ident);
    let mut first = true;
    for (arg, ty) in user_args {
        if first {
            first = false;
        } else {
            wrt!(res, ", ");
        }
        wrt!(res, "{}", bracketed_name(arg, ty));
    }
    wrt!(res, ")");
    if is_md_txt {
        wrt!(res, "`");
    }
    res
}

fn get_user_args(func: &FuncInfo) -> Vec<(&ParamTypeInfo, &ParamInfo)> {
    func.args
        .iter()
        .filter_map(|v| {
            if let ParamType::Value(ref val) = v.ty {
                Some((v, val))
            } else {
                None
            }
        })
        .collect()
}

fn bracketed_name(arg: &ParamTypeInfo, ty: &ParamInfo) -> String {
    let vcc = ty.ty_info.to_vcc_type();
    let ident = &arg.ident;
    if ty.is_optional {
        format!("[{vcc} {ident}]")
    } else {
        format!("{vcc} {ident}")
    }
}

/// Print user provided documentation, indenting the `## ...` subsections with a prefix
fn write_docs(docs: &mut String, user_doc: &str, prefix: &str) {
    if !user_doc.is_empty() {
        if user_doc.contains('#') {
            // If the user provided documentation contains a `#`, we need to indent the subsections
            let mut user_doc = user_doc.replace("\n#", &format!("\n#{prefix}"));
            if user_doc.starts_with('#') {
                user_doc.insert_str(0, prefix);
            }
            ln!(docs, "\n{user_doc}");
        } else {
            ln!(docs, "\n{user_doc}");
        }
    }
}
