use qjs_rs::{q, JSContext, JSPropertyItem,RawJsValue,JSClass};
use std::os::raw::{c_int};
use seija::app::{AppBuilder,App};
use seija::win::{dpi::{LogicalSize}};
use seija::module_bundle::{Simple2d};
use seija::core::{LimitSetting};
use crate::g2d::{SIMPLE2D_CLASS};
use crate::core::{JSGame};
use std::path::{PathBuf};

static mut APP_CLASS:Option<JSClass> = None;

pub unsafe fn app_init(ctx: &mut JSContext, m: *mut q::JSModuleDef) {
    APP_CLASS = Some(JSClass::new("App",ctx.c_rt()));
    let app_obj = q::JS_NewObject(ctx.c_ctx());
    let app_attrs = vec![
        JSPropertyItem::func(c_str!("runApp"), Some(c_run_app), 1),
        JSPropertyItem::func(c_str!("newApp"), Some(c_new_app), 1),
        JSPropertyItem::func(c_str!("closeApp"), Some(c_close_app), 1),
        JSPropertyItem::func(c_str!("exePath"), Some(c_exe_path), 1),
    ];
    ctx.set_property_function_list(app_obj, &app_attrs);
    q::JS_SetModuleExport(ctx.c_ctx(), m, c_str!("app").as_ptr(), app_obj);
}

pub fn app_export(ctx: *mut q::JSContext, m: *mut q::JSModuleDef) {
    unsafe {
        q::JS_AddModuleExport(ctx, m, c_str!("app").as_ptr());
    };
}

pub unsafe extern "C" fn c_new_app(ctx: *mut q::JSContext,_: q::JSValue, count:c_int, argv: *mut q::JSValue) -> q::JSValue {
    let args = std::slice::from_raw_parts(argv,count as usize);
    let class_id = SIMPLE2D_CLASS.as_ref().unwrap().class_id();
    let s2d_ptr:&mut Simple2d = std::mem::transmute(q::JS_GetOpaque(args[0],class_id));
    let s2d:Simple2d = *Box::from_raw(s2d_ptr);
    let game_func = RawJsValue::deserialize_value(args[1],ctx).unwrap();
    let js_game = JSGame::new(game_func,ctx);

    let app_class:&JSClass = APP_CLASS.as_ref().unwrap();
    let mut app_object = app_class.new_object(ctx);
    let app = Box::new(AppBuilder::new().with_update_limiter(LimitSetting::Sleep(30)).build(s2d, js_game));
    app_object.set_opaque(Box::into_raw(app));

    app_object.value()
}

pub unsafe extern "C" fn c_run_app(_ctx: *mut q::JSContext,_: q::JSValue, count:c_int, argv: *mut q::JSValue) -> q::JSValue {
    let args = std::slice::from_raw_parts(argv,count as usize);
    let class_id = APP_CLASS.as_ref().unwrap().class_id();
    let mut app:Box<App<JSGame,Simple2d>> = Box::from_raw(std::mem::transmute(q::JS_GetOpaque(args[0],class_id))) ;
    app.run();
    RawJsValue::val_null()
}

pub unsafe extern "C" fn c_close_app(_ctx: *mut q::JSContext,_: q::JSValue, count:c_int, argv: *mut q::JSValue) -> q::JSValue {
    let args = std::slice::from_raw_parts(argv,count as usize);
    let class_id = APP_CLASS.as_ref().unwrap().class_id();
    let app:&mut App<JSGame,Simple2d> = std::mem::transmute(q::JS_GetOpaque(args[0],class_id));
   
    app.close();
    RawJsValue::val_null()
}

pub unsafe extern "C" fn c_exe_path(ctx: *mut q::JSContext,_: q::JSValue, _count:c_int, _argv: *mut q::JSValue) -> q::JSValue {
    let cur_dir = std::env::current_dir().ok().and_then(|path:PathBuf| path.to_str().map(|s| String::from(s))).unwrap_or(String::from("./"));
    RawJsValue::val_string(&cur_dir, ctx)
}

