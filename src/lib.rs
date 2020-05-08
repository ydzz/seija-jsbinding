#![deny(warnings)]
pub mod core;
pub mod app;
pub mod g2d;
pub mod loader;
#[macro_use]
extern crate c_str_macro;
use std::os::raw::{c_int};
use qjs_rs::{JSContext,AutoDropJSValue,RawJsValue,q,JSClass};

pub fn binding_all(ctx:&mut JSContext) {
    unsafe {
        crate::core::WORLD_CLASS = Some(JSClass::new("World",ctx.c_rt()));
        q::JS_SetContextOpaque(ctx.c_ctx(),std::mem::transmute(ctx as *mut JSContext));
    };
    let m = ctx.new_c_module("seija",Some(on_module_init));
   unsafe {
       q::JS_AddModuleExport(ctx.c_ctx(),m,c_str!("version").as_ptr());
   };
   app::app_export(ctx.c_ctx(),m);
   g2d::g2d_export(ctx.c_ctx(),m);
   loader::loader_export(ctx.c_ctx(),m);
}

pub unsafe extern "C" fn on_module_init(ctx: *mut q::JSContext, m: *mut q::JSModuleDef) -> c_int {
    let ctx_ptr = q::JS_GetContextOpaque(ctx);
    let rs_ctx:&mut JSContext = std::mem::transmute(ctx_ptr);
    q::JS_SetModuleExport(ctx,m,c_str!("version").as_ptr(),RawJsValue::val_string(&String::from("1.0.0"), ctx));
    app::app_init(rs_ctx,m);
    g2d::g2d_init(rs_ctx,m);
    loader::loader_init(rs_ctx,m);
    0
  }

#[derive(Debug)]
pub enum BootStartError {
    LoadFileError
}

pub fn boot_start(ctx:&mut JSContext,main_js_name:&str) -> Result<AutoDropJSValue,BootStartError> {
    let file_string:String = std::fs::read_to_string(main_js_name).map_err(|_| BootStartError::LoadFileError)?;
    let ret_val = ctx.eval(file_string.as_str(), main_js_name,(q::JS_EVAL_TYPE_GLOBAL | q::JS_EVAL_TYPE_MODULE) as i32);
    Ok(ret_val)
}

#[cfg(test)]
mod tests {
    use qjs_rs::{JSRuntime,JSContext,init_internal};
    use crate::{binding_all,boot_start};
    #[test]
    fn test_run() {
        let mut runtime = JSRuntime::new().unwrap();
        let mut ctx = JSContext::new(&runtime).unwrap();
        init_internal(&mut ctx,&mut runtime);
        binding_all(&mut ctx);
        boot_start(&mut ctx,"./tests/test.js").unwrap();
    }
}
