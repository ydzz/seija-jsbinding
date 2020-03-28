use qjs_rs::{JSContext,q,JSPropertyItem,RawJsValue};
use std::os::raw::{c_int};
use seija::assets::{Loader};

pub unsafe fn loader_init(ctx: &mut JSContext, m: *mut q::JSModuleDef) {
    let app_obj = q::JS_NewObject(ctx.c_ctx());
    let app_attrs = vec![
        JSPropertyItem::func(c_str!("loadSync"), Some(c_load_sync), 1),
        //JSPropertyItem::func(c_str!("NewApp"), Some(c_new_app), 1),
        //JSPropertyItem::func(c_str!("CloseApp"), Some(c_close_app), 1),
    ];
    ctx.set_property_function_list(app_obj, &app_attrs);
    q::JS_SetModuleExport(ctx.c_ctx(), m, c_str!("loader").as_ptr(), app_obj);
}

pub fn loader_export(ctx: *mut q::JSContext, m: *mut q::JSModuleDef) {
    unsafe {
        q::JS_AddModuleExport(ctx, m, c_str!("loader").as_ptr());
    };
}

pub unsafe extern "C" fn c_load_sync(ctx: *mut q::JSContext,_: q::JSValue, count:c_int, argv: *mut q::JSValue) -> q::JSValue {
    RawJsValue::val_null()
}