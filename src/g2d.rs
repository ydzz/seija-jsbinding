use crate::core::WORLD_CLASS;
use qjs_rs::{
    q, JSClass, JSClassOject, JSContext, JSPropertyItem, JSValue, RawJsValue,AutoDropJSValue,
};
use seija::assets::{errors::AssetLoadError, Handle,AssetStorage};
use seija::common::{Rect2D, Transform,transform::{Parent},AnchorAlign};
use seija::frp::{Event,Behavior};
use seija::json::Value;
use seija::math::Vector3;
use seija::module_bundle::{DefaultBackend, S2DLoader, Simple2d};
use seija::render::{
    Transparent,
    components::{ImageRender,SpriteRender,LineMode,TextRender,SpriteSheet,ImageType,Mesh2D,ImageFilledType,Sprite},
    types, FontAsset,
};
use seija::window::{ViewPortSize};
use seija::event::{cb_event::{CABEventRoot},EventNode,GameEventType};
use seija::specs::{shred::FetchMut,Entity,World,WorldExt,world::{Builder}};
use seija::win::{dpi::LogicalSize, WindowBuilder};
use std::collections::HashMap;
use std::os::raw::c_int;
use std::sync::{Arc};
use crate::core::{JSFRPNode,QJSValue,QJSContext,QJSWorld};

pub static mut SIMPLE2D_CLASS: Option<JSClass> = None;
pub static mut LOADER_CLASS: Option<JSClass> = None;
pub static mut EVENT_CLASS: Option<JSClass> = None;
pub static mut BEHAVIOR_CLASS: Option<JSClass> = None;
static mut LOADER_REF_COUNT: u32 = 0;



pub unsafe fn g2d_init(ctx: &mut JSContext, m: *mut q::JSModuleDef) {
    SIMPLE2D_CLASS = Some(JSClass::new_full("Simple2d", ctx.c_rt(),Some(simple2d_finalizer),None,None));
    LOADER_CLASS = Some(JSClass::new_full("Loader",ctx.c_rt(),Some(loader_finalizer),None,None));
    EVENT_CLASS = Some(JSClass::new_full("Event",ctx.c_rt(),Some(event_finalizer),Some(event_gc),None));
    BEHAVIOR_CLASS = Some(JSClass::new_full("Behavior",ctx.c_rt(),Some(behavior_finalizer),None,None));

    let g2d_obj = q::JS_NewObject(ctx.c_ctx());
    let g2d_attrs = vec![
        JSPropertyItem::func(c_str!("newSimple2d"), Some(c_new_simple2d), 1),
        JSPropertyItem::func(c_str!("fetchLoader"), Some(c_fetch_loader), 1),
        JSPropertyItem::func(c_str!("loadSync"), Some(c_load_sync), 1),
        JSPropertyItem::func(c_str!("getEvent"),Some(c_get_event) ,1),
        JSPropertyItem::func(c_str!("mergeEvent"),Some(c_merge_event),1),
        JSPropertyItem::func(c_str!("chainEvent"),Some(c_chain_event) ,1),
        JSPropertyItem::func(c_str!("newEntity"),Some(c_new_entity) ,1),
        JSPropertyItem::func(c_str!("refCount"),Some(c_ref_count) ,1),
        JSPropertyItem::func(c_str!("destoryEntity"),Some(c_destory_entity) ,1),
        JSPropertyItem::func(c_str!("newBehavior"),Some(c_new_behavior) ,1),
        JSPropertyItem::func(c_str!("getBehaviorValue"),Some(c_get_behavior_value), 1),
        JSPropertyItem::func(c_str!("setBehaviorCallback"),Some(c_set_behavior_callback), 1),
        JSPropertyItem::func(c_str!("attachBehavior"),Some(c_attach_behavior) ,1),
        JSPropertyItem::func(c_str!("behaviorSetFoldFunc"),Some(behavior_set_fold_func), 1),
        JSPropertyItem::func(c_str!("getViewPortSize"),Some(c_get_view_port_size), 1),
        JSPropertyItem::func(c_str!("getTextureSize"),Some(c_get_texture_size), 1),
        JSPropertyItem::func(c_str!("setParent"),Some(c_set_parent), 1),
        JSPropertyItem::func(c_str!("getSpriteRectInfo"),Some(c_get_sprite_rect_info), 1),
        //component
        JSPropertyItem::func(c_str!("addCABEventRoot"),Some(c_add_cab_event_root) ,1),
        JSPropertyItem::func(c_str!("addRect2d"), Some(c_add_rect_2d), 1),
        JSPropertyItem::func(c_str!("addTransform"),Some(c_add_transform), 1),
        JSPropertyItem::func(c_str!("addImageRender"),Some(c_add_image_render), 1),
        JSPropertyItem::func(c_str!("addSpriteRender"),Some(c_add_sprite_render), 1),
        JSPropertyItem::func(c_str!("addTextRender"),Some(c_add_text_render), 1),
        JSPropertyItem::func(c_str!("addTransparent"),Some(c_add_transparent), 1),
        //component attr
        JSPropertyItem::func(c_str!("setTransformBehavior"),Some(set_transform_behavior),1),
        JSPropertyItem::func(c_str!("setRect2dBehavior"),Some(set_rect2d_behavior),1),
        JSPropertyItem::func(c_str!("setImageRenderBehavior"),Some(set_image_render_behavior),1),
        JSPropertyItem::func(c_str!("setSpriteRenderBehavior"),Some(set_sprite_render_behavior),1),
        JSPropertyItem::func(c_str!("setTextRenderBehavior"),Some(set_text_render_behavior),1)
    ];
    ctx.set_property_function_list(g2d_obj, &g2d_attrs);
    q::JS_SetModuleExport(ctx.c_ctx(), m, c_str!("g2d").as_ptr(), g2d_obj);
}

/* #region Main */
pub fn g2d_export(ctx: *mut q::JSContext, m: *mut q::JSModuleDef) {
    unsafe {
        q::JS_AddModuleExport(ctx, m, c_str!("g2d").as_ptr());
    };
}

unsafe extern "C" fn loader_finalizer(_rt: *mut q::JSRuntime, val: q::JSValue) {
    let loader_class: &JSClass = LOADER_CLASS.as_ref().unwrap();
    let ptr: *mut FetchMut<'_, S2DLoader> = std::mem::transmute(q::JS_GetOpaque(val, loader_class.class_id()));
    let _box_loader: Box<FetchMut<'_, S2DLoader>> = Box::from_raw(ptr);
    LOADER_REF_COUNT -= 1;
}

unsafe extern "C" fn simple2d_finalizer(_rt: *mut q::JSRuntime, _val: q::JSValue) {
    /*
    let simple_class: &JSClass = SIMPLE2D_CLASS.as_ref().unwrap();
    let ptr: *mut Simple2d = std::mem::transmute(q::JS_GetOpaque(val, simple_class.class_id()));
    Box::from_raw(ptr);*/
}

unsafe extern "C" fn event_finalizer(_rt: *mut q::JSRuntime, _val: q::JSValue) {
    dbg!("event finalizer");
}

unsafe extern "C" fn behavior_finalizer(_rt: *mut q::JSRuntime, val: q::JSValue) {
    println!("behavior finalizer");
    let behavior: *mut Behavior<QJSValue> = std::mem::transmute(q::JS_GetOpaque(val,BEHAVIOR_CLASS.as_ref().unwrap().class_id()));
    Box::from_raw(behavior);
    //println!("ref:{:?}",RawJsValue(val).ref_count());
}

unsafe extern "C" fn event_gc(_rt: *mut q::JSRuntime, _val: q::JSValue,_mark_func: q::JS_MarkFunc) {
    //let raw_js = RawJsValue(val);
    //raw_js.add_ref_count(-1);
}

pub unsafe extern "C" fn c_new_simple2d(ctx: *mut q::JSContext,_: q::JSValue,count: c_int,argv: *mut q::JSValue) -> q::JSValue {
    let args = std::slice::from_raw_parts(argv, count as usize);
    let simple2d_class: &mut JSClass = SIMPLE2D_CLASS.as_mut().unwrap();
    let mut class_obj = simple2d_class.new_object(ctx);
    let mut s2d = Box::new(Simple2d::new());

    let mut may_width = None;
    let mut may_height = None;
    let attr_dic = RawJsValue::deserialize_value(args[0], ctx).unwrap();
    
    let js_win_attr = attr_dic.as_object().and_then(|m| m.get(&String::from("window")));
    if let Some(js_val) = js_win_attr {
        let win_map_val = RawJsValue::deserialize_value(js_val.inner().0, ctx).unwrap();
        let may_map = win_map_val.as_object();
        if let Some(Some(js_width)) = may_map.map(|m| m.get(&String::from("width"))) {
            let js_num = RawJsValue::deserialize_value(js_width.inner().0, ctx).unwrap();
            may_width = js_num.as_number();
        }
        if let Some(Some(js_height)) = may_map.map(|m| m.get(&String::from("height"))) {
            let js_num = RawJsValue::deserialize_value(js_height.inner().0, ctx).unwrap();
            may_height = js_num.as_number();
        }
        if let Some(Some(js_bg_color)) = may_map.map(|m| m.get(&String::from("bgColor"))) {
            let color_arr = RawJsValue::deserialize_value(js_bg_color.inner().0, ctx).unwrap();
            let mut color: [f32; 4] = [1f32, 1f32, 1f32, 1f32];
            if let Some(arr) = color_arr.as_array() {
                color[0] = arr.get_unchecked(0).as_number().map(|n: f64| n as f32).unwrap_or(1.0f32);
                color[1] = arr.get_unchecked(1).as_number().map(|n: f64| n as f32).unwrap_or(1.0f32);
                color[2] = arr.get_unchecked(2).as_number().map(|n: f64| n as f32).unwrap_or(1.0f32);
                color[3] = arr.get_unchecked(3).as_number().map(|n: f64| n as f32).unwrap_or(1.0f32);
            }
            s2d.with_bg_color(color);
        }
    };
    let width = may_width.unwrap_or(640f64);
    let height = may_height.unwrap_or(480f64);

    s2d.with_window(move |wb: &mut WindowBuilder| {
        wb.window.dimensions = Some(LogicalSize {
            width: width,
            height: height,
        });
    });
    class_obj.set_opaque(Box::into_raw(s2d));
    class_obj.value()
}

pub unsafe extern "C" fn c_fetch_loader(ctx: *mut q::JSContext,_: q::JSValue,_count: c_int,argv: *mut q::JSValue) -> q::JSValue {
    let class_id = WORLD_CLASS.as_ref().unwrap().class_id();
    let world: &mut World = std::mem::transmute(q::JS_GetOpaque(*argv, class_id));
    if LOADER_REF_COUNT > 0 {
        return RawJsValue::val_null();
    }
    let loader = world.fetch_mut::<S2DLoader>();
    LOADER_REF_COUNT += 1;
    let loader_class: &JSClass = LOADER_CLASS.as_ref().unwrap();
    let mut loader_object: JSClassOject = loader_class.new_object(ctx);
    let box_loader = Box::new(loader);
    let loader_ptr = Box::into_raw(box_loader);
    loader_object.set_opaque(loader_ptr);
    loader_object.value()
}

pub unsafe extern "C" fn c_load_sync(ctx: *mut q::JSContext,_: q::JSValue,count: c_int,argv: *mut q::JSValue) -> q::JSValue {
    let args = std::slice::from_raw_parts(argv, count as usize);
    let loader_class: &JSClass = LOADER_CLASS.as_ref().unwrap();
    let loader: &mut FetchMut<'_, S2DLoader> = std::mem::transmute(q::JS_GetOpaque(args[0], loader_class.class_id()));

    let world: &mut World = std::mem::transmute(q::JS_GetOpaque(args[1],WORLD_CLASS.as_ref().unwrap().class_id()));

    let asset_type = RawJsValue::deserialize_value(args[2], ctx).unwrap().as_int().unwrap();
    let asset_path = RawJsValue::deserialize_value(args[3], ctx).unwrap().into_string().unwrap();
    let may_asset_id = match asset_type {
        0 => loader.load_sync::<Value, DefaultBackend>(&asset_path, world).map(|h| h.id()),
        1 => loader.load_sync::<types::Texture, DefaultBackend>(&asset_path, world).map(|h| h.id()),
        2 => loader.load_sync::<SpriteSheet, DefaultBackend>(&asset_path, world).map(|h| h.id()),
        3 => loader.load_sync::<FontAsset, DefaultBackend>(&asset_path, world).map(|h| h.id()),
        _ => Err(AssetLoadError::NotFoundLoader),
    };
    if may_asset_id.is_ok() {
        let aid = may_asset_id.unwrap() as i32;
        RawJsValue::val_i32(aid)
    } else {
        let err = may_asset_id.unwrap_err();
        let str_err = format!("{:?}", err);
        RawJsValue::val_string(&str_err, ctx)
    }
}

//mev <- mergeEvent [e0 $> "button",e1 $> "button-active"]
pub unsafe extern "C" fn c_merge_event(ctx: *mut q::JSContext,_: q::JSValue,count: c_int,argv: *mut q::JSValue) -> q::JSValue {
    let args = std::slice::from_raw_parts(argv, count as usize);
    let js_event_arr:Vec<AutoDropJSValue> = RawJsValue::deserialize_value_(args[0], ctx,true).unwrap().into_array_raw().unwrap();
    let mut rc_event:Arc<Event<QJSValue>> = Arc::new(Event::default());
    let event_class: &JSClass = EVENT_CLASS.as_ref().unwrap();
    let rc_event_ptr:*mut Event<QJSValue> = std::mem::transmute(Arc::get_mut(&mut rc_event).unwrap());
    let mut event_object: JSClassOject = event_class.new_object(ctx);
    event_object.set_opaque(rc_event_ptr);
    for js_ev in js_event_arr.iter() {
       let jse = js_ev.inner().0;
       let event: &mut Event<QJSValue> = event_ptr(jse);
       event.chain_rc_next(rc_event.clone());
    }
    event_object.value()
}

pub unsafe extern "C" fn c_get_event(ctx: *mut q::JSContext,_: q::JSValue,count: c_int,argv: *mut q::JSValue) -> q::JSValue {
    let args = std::slice::from_raw_parts(argv, count as usize);
    let world: &mut World = std::mem::transmute(q::JS_GetOpaque(args[0],WORLD_CLASS.as_ref().unwrap().class_id()));
    let may_id = RawJsValue::deserialize_value(args[1],ctx).ok();
    if may_id.is_none() {
        return RawJsValue::val_null()
    }
   
    let eid = may_id.unwrap().as_int().unwrap();
    let e = world.entities().entity(eid as u32);
    let mut frp_node_storage = world.write_storage::<JSFRPNode>();
    let mut event_storage = world.write_storage::<EventNode>();
    let event_type_id = RawJsValue::deserialize_value(args[2],ctx).ok().and_then(|q| q.as_int().map(|n| n as u32)).unwrap_or(0u32);
    let mut new_event = Arc::new(Event::default());
    
    let event_ptr:*mut Event<QJSValue>  = std::mem::transmute(Arc::get_mut(&mut new_event).unwrap());
    let event_class: &JSClass = EVENT_CLASS.as_ref().unwrap();
    let mut event_object: JSClassOject = event_class.new_object(ctx);
    event_object.set_opaque(event_ptr);
    let ret_val = event_object.value();
    if !frp_node_storage.contains(e) {
        let mut frp_node = JSFRPNode::default();
        frp_node.set_ctx(ctx);
        frp_node.add_event(event_type_id,new_event);
        frp_node.push_js_object(event_object);
        frp_node_storage.insert(e,frp_node).unwrap();
    } else {
        let frp_node = frp_node_storage.get_mut(e).unwrap();
        frp_node.push_js_object(event_object);
        frp_node.add_event(event_type_id,new_event);
    }
    if !event_storage.contains(e) {
        let ev = EventNode::default();
        event_storage.insert(e,ev).unwrap();
    };
    
    let is_capture = RawJsValue::deserialize_value(args[3],ctx).ok().and_then(|q| q.as_bool()).unwrap_or(false);
    let typ = GameEventType::from(event_type_id).unwrap_or(GameEventType::TouchStart);
    let ev_node:&mut EventNode = event_storage.get_mut(e).unwrap();
    let qctx = QJSContext(ctx);
    ev_node.register(is_capture,typ,move |e,world| {
        let frp_node_storage = world.write_storage::<JSFRPNode>();
        let frp_node = frp_node_storage.get(e).unwrap();
        if let Some(event_ref) = frp_node.events().get(&event_type_id) {
            event_ref.on_fire(RawJsValue::val_i32(eid).into(),qctx.0);
        }
    });
   
    ret_val
}

pub unsafe extern "C" fn c_new_entity(_ctx: *mut q::JSContext,_: q::JSValue,count: c_int,argv: *mut q::JSValue) -> q::JSValue {
    let args = std::slice::from_raw_parts(argv, count as usize);
    let world: &mut World = std::mem::transmute(q::JS_GetOpaque(args[0],WORLD_CLASS.as_ref().unwrap().class_id()));
    
    let e = world.create_entity().build();
    RawJsValue::val_i32(e.id() as i32)
}

pub unsafe extern "C" fn c_ref_count(ctx: *mut q::JSContext,_: q::JSValue,_count: c_int,argv: *mut q::JSValue) -> q::JSValue {
    let raw_js = RawJsValue(*argv);
    RawJsValue::val_i32(raw_js.ref_count())
}

pub unsafe extern "C" fn c_chain_event(ctx: *mut q::JSContext,_: q::JSValue,count: c_int,argv: *mut q::JSValue) -> q::JSValue {
    let args = std::slice::from_raw_parts(argv, count as usize);
    let event: &mut Event<QJSValue> = std::mem::transmute(q::JS_GetOpaque(args[0],EVENT_CLASS.as_ref().unwrap().class_id()));
    let f_val:QJSValue = args[1].into();
    let event_ptr:*mut Event<QJSValue> = event.chain_next(f_val);
    let event_class: &JSClass = EVENT_CLASS.as_ref().unwrap();
    let mut event_object: JSClassOject = event_class.new_object(ctx);
    event_object.set_opaque(event_ptr);
    event_object.value()
}

pub unsafe extern "C" fn c_destory_entity(ctx: *mut q::JSContext,_: q::JSValue,count: c_int,argv: *mut q::JSValue) -> q::JSValue {
    let args = std::slice::from_raw_parts(argv, count as usize);
    let world: &mut World = std::mem::transmute(q::JS_GetOpaque(args[0],WORLD_CLASS.as_ref().unwrap().class_id()));
    let e = get_entity(world,args[1], ctx).unwrap();
    world.entities().delete(e).unwrap();
   
    RawJsValue::val_null()
}

pub unsafe extern "C" fn c_new_behavior(ctx: *mut q::JSContext,_: q::JSValue,count: c_int,argv: *mut q::JSValue) -> q::JSValue {
    let args = std::slice::from_raw_parts(argv, count as usize);
    RawJsValue(args[0]).add_ref_count(1);
    let mut new_behavior = Box::new(Behavior::new(QJSValue(args[0]),ctx));
    let behavior_class: &JSClass = BEHAVIOR_CLASS.as_ref().unwrap();
    let mut js_behavior = behavior_class.new_object(ctx);
    new_behavior.set_object(QJSValue(js_behavior.value()));
    js_behavior.set_opaque(Box::into_raw(new_behavior));
    js_behavior.value()
}

pub unsafe extern "C" fn c_get_behavior_value(ctx: *mut q::JSContext,_: q::JSValue,count: c_int,argv: *mut q::JSValue) -> q::JSValue {
    let args = std::slice::from_raw_parts(argv, count as usize);
    let behavior: &mut Behavior<QJSValue> = std::mem::transmute(q::JS_GetOpaque(args[0],BEHAVIOR_CLASS.as_ref().unwrap().class_id()));
    let val = behavior.value();
    RawJsValue(val.0).add_ref_count(1);
    val.0
}

pub unsafe extern "C" fn c_set_behavior_callback(ctx: *mut q::JSContext,_: q::JSValue,count: c_int,argv: *mut q::JSValue) -> q::JSValue {
    let args = std::slice::from_raw_parts(argv, count as usize);
    let behavior: &mut Behavior<QJSValue> = std::mem::transmute(q::JS_GetOpaque(args[0],BEHAVIOR_CLASS.as_ref().unwrap().class_id()));
    behavior.set_call_func(args[1].into());
    RawJsValue::val_null()
}

pub unsafe extern "C" fn c_attach_behavior(_ctx: *mut q::JSContext,_: q::JSValue,count: c_int,argv: *mut q::JSValue) -> q::JSValue {
    let args = std::slice::from_raw_parts(argv, count as usize);
    let event: &mut Event<QJSValue> = std::mem::transmute(q::JS_GetOpaque(args[0],EVENT_CLASS.as_ref().unwrap().class_id()));
    let behavior: *mut Behavior<QJSValue> = std::mem::transmute(q::JS_GetOpaque(args[1],BEHAVIOR_CLASS.as_ref().unwrap().class_id()));
    let b_js = RawJsValue(args[1]);
    b_js.add_ref_count(1);
    event.attach_behavior(behavior);
    RawJsValue::val_bool(true)
}

pub unsafe extern "C" fn behavior_set_fold_func(ctx: *mut q::JSContext,_: q::JSValue,count: c_int,argv: *mut q::JSValue) -> q::JSValue {
    let args = std::slice::from_raw_parts(argv, count as usize);
    let behavior: &mut Behavior<QJSValue> = std::mem::transmute(q::JS_GetOpaque(args[0],BEHAVIOR_CLASS.as_ref().unwrap().class_id()));
    behavior.set_func(args[1].into());
    RawJsValue::val_null()
}

fn get_entity(world:&mut World,js_val:q::JSValue,ctx:*mut q::JSContext) -> Option<Entity> {
    RawJsValue::deserialize_value(js_val,ctx).ok()
                             .and_then(|n| n.as_int())
                             .map(|eid| world.entities().entity(eid as u32))
}

pub unsafe extern "C" fn c_get_view_port_size(ctx: *mut q::JSContext,_: q::JSValue,count: c_int,argv: *mut q::JSValue) -> q::JSValue {
    if count == 0 {
        return RawJsValue::val_null()
    }
    let world: &mut World = std::mem::transmute(q::JS_GetOpaque(*argv,WORLD_CLASS.as_ref().unwrap().class_id()));
    let (w,h) = {
        let view_size = world.fetch::<ViewPortSize>();
        (view_size.width() as f32,view_size.height() as f32)
    };
    JSValue::Array(vec![JSValue::Float(w as f64),JSValue::Float(h as f64)]).to_c_value(ctx)
}

pub unsafe extern "C" fn c_get_texture_size(ctx: *mut q::JSContext,_: q::JSValue,count: c_int,argv: *mut q::JSValue) -> q::JSValue {
    let args = std::slice::from_raw_parts(argv, count as usize);
    let world: &mut World = std::mem::transmute(q::JS_GetOpaque(args[0],WORLD_CLASS.as_ref().unwrap().class_id()));
    let tex_id = RawJsValue::deserialize_value(args[1], ctx).unwrap().as_int().unwrap();
    let storage = world.fetch::<AssetStorage<types::Texture>>();
    if let Some(tex_ref) = storage.get_by_id(tex_id as u32) {
        let (w,h) = tex_ref.texture_size();
        return JSValue::Array(vec![JSValue::Int(w as i32),JSValue::Int(h as i32)]).to_c_value(ctx);
    }
    RawJsValue::val_null()
}



//component
pub unsafe extern "C" fn c_add_cab_event_root(ctx: *mut q::JSContext,_: q::JSValue,count: c_int,argv: *mut q::JSValue) -> q::JSValue {
    let args = std::slice::from_raw_parts(argv, count as usize);
    let world: &mut World = std::mem::transmute(q::JS_GetOpaque(args[0],WORLD_CLASS.as_ref().unwrap().class_id()));
    let may_entity = get_entity(world, args[1], ctx);
    if let Some(entity) = may_entity {
        let mut storage = world.write_storage::<CABEventRoot>();
        if storage.contains(entity) {
            return RawJsValue::val_bool(false)
        };
        storage.insert(entity,CABEventRoot {}).unwrap();
    }
    RawJsValue::val_null()
}
/* #endregion */

pub unsafe extern "C" fn c_add_rect_2d(ctx: *mut q::JSContext,_: q::JSValue,count: c_int,argv: *mut q::JSValue) -> q::JSValue {
    let args = std::slice::from_raw_parts(argv, count as usize);
    let world: &mut World = std::mem::transmute(q::JS_GetOpaque(args[0],WORLD_CLASS.as_ref().unwrap().class_id()));
    let may_entity = get_entity(world, args[1], ctx);
    if let Some(entity) = may_entity {
        let mut storage = world.write_storage::<Rect2D>();
        if storage.contains(entity) {
            return RawJsValue::val_bool(false)
        };
        let width = RawJsValue(args[2]).to_value(ctx).ok().and_then(|v| v.as_number()).unwrap_or(0f64);
        let height = RawJsValue(args[3]).to_value(ctx).ok().and_then(|v| v.as_number()).unwrap_or(0f64);
        let a_x = RawJsValue(args[4]).to_value(ctx).ok().and_then(|v| v.as_number()).unwrap_or(0f64);
        let a_y = RawJsValue(args[5]).to_value(ctx).ok().and_then(|v| v.as_number()).unwrap_or(0f64);
        let new_rect = Rect2D {
            width:width as f32,
            height:height as f32,
            anchor:[a_x as f32,a_y as f32]
        };
        storage.insert(entity,new_rect).unwrap();
        return RawJsValue::val_bool(true);
    }
    RawJsValue::val_bool(false)
}


pub unsafe extern "C" fn c_add_transform(ctx: *mut q::JSContext,_: q::JSValue,count:c_int,argv: *mut q::JSValue) -> q::JSValue {
    let args = std::slice::from_raw_parts(argv, count as usize);
    let world: &mut World = std::mem::transmute(q::JS_GetOpaque(args[0],WORLD_CLASS.as_ref().unwrap().class_id()));
    let entity = get_entity(world, args[1], ctx).unwrap();
    if count <= 2 {
        let mut storage = world.write_storage::<Transform>();
        if storage.contains(entity) {
            return RawJsValue::val_bool(false)
        }
        storage.insert(entity,Transform::default()).unwrap();
        return RawJsValue::val_bool(true)
    }
    let mut pos:Vector3<f32> = Vector3::new(0f32,0f32,0f32);
    let mut scale:Vector3<f32> = Vector3::new(1f32,1f32,1f32);
    let mut r:Vector3<f32> = Vector3::new(0f32,0f32,0f32);
    if count > 2 {
        set_vector3_array(&mut pos,args[2],ctx);
    }
    if count > 3 {
        set_vector3_array(&mut scale,args[3],ctx);
    }
    if count > 4 {
        set_vector3_array(&mut r,args[4],ctx);
    }
    let mut new_trans = Transform::default();
    new_trans.set_position(pos);
    new_trans.set_scale(scale);
    new_trans.set_rotation_euler(r.x,r.y,r.z);
    let mut storage = world.write_storage::<Transform>();
    if storage.contains(entity) {
        return RawJsValue::val_bool(false)
    }
    storage.insert(entity,new_trans).unwrap();
    RawJsValue::val_bool(true)
}

pub unsafe extern "C" fn c_add_image_render(ctx: *mut q::JSContext,_: q::JSValue,count:c_int,argv: *mut q::JSValue) -> q::JSValue {
    let args = std::slice::from_raw_parts(argv, count as usize);
    let world: &mut World = std::mem::transmute(q::JS_GetOpaque(args[0],WORLD_CLASS.as_ref().unwrap().class_id()));
    let entity = get_entity(world, args[1], ctx).unwrap();
    let tex_id = RawJsValue::deserialize_value(args[2],ctx).unwrap().as_int().unwrap();
    let mut image_render = ImageRender::new(Handle::new(tex_id as u32));
    if count > 3 {
        let may_num_arr = RawJsValue::deserialize_value(args[3],ctx).ok().and_then(|f| f.array_get_number());
        if let Some(num_arr) = may_num_arr {
            image_render.set_color(*num_arr.get_unchecked(0) as f32,*num_arr.get_unchecked(1) as f32,
                                   *num_arr.get_unchecked(2) as f32,*num_arr.get_unchecked(3) as f32);
        }
    }

    if count > 4 {
        //TODO
        //let num_typ = RawJsValue::deserialize_value(args[4],ctx).ok().and_then(|f| f.as_int()).unwrap();
        //image_render.set_type();
    }
    
    let mut storage = world.write_storage::<ImageRender>();
    if storage.contains(entity) {
        return RawJsValue::val_bool(false)
    }
    let mut mesh_storage = world.write_storage::<Mesh2D>();
    storage.insert(entity,image_render).unwrap();
    mesh_storage.insert(entity,Mesh2D::default()).unwrap();
    RawJsValue::val_bool(true)
}


pub unsafe extern "C" fn c_add_sprite_render(ctx: *mut q::JSContext,_: q::JSValue,count:c_int,argv: *mut q::JSValue) -> q::JSValue {
    let args = std::slice::from_raw_parts(argv, count as usize);
    let world: &mut World = std::mem::transmute(q::JS_GetOpaque(args[0],WORLD_CLASS.as_ref().unwrap().class_id()));
    let entity = get_entity(world, args[1], ctx).unwrap();
    let sheet_id = RawJsValue::deserialize_value(args[2],ctx).unwrap().as_int().unwrap();
    let sprite_name = RawJsValue::deserialize_value(args[3],ctx).unwrap().into_string().unwrap();
    let mut sprite_render = SpriteRender::new(Handle::new(sheet_id as u32),&sprite_name);
    let js_type = RawJsValue(args[4]).to_value(ctx).unwrap();
    if let Some(num_arr) = js_type.array_get_number() {
        let typ = num_arr[0] as i32;
        match typ {
            0 => sprite_render.set_type(ImageType::Simple),
            1 => sprite_render.set_type(ImageType::Sliced(num_arr[1] as f32,num_arr[2] as f32,num_arr[3] as f32,num_arr[4] as f32)),
            2 => sprite_render.set_slice_type_by_cfg(num_arr[1] as usize,&world.fetch::<AssetStorage<SpriteSheet>>()),
            3 => {
                let ftyp = num_arr[1] as i32;
                match ftyp {
                    0 => sprite_render.set_type(ImageType::Filled(ImageFilledType::HorizontalLeft,num_arr[1] as f32)),
                    1 => sprite_render.set_type(ImageType::Filled(ImageFilledType::HorizontalRight,num_arr[2] as f32)),
                    2 => sprite_render.set_type(ImageType::Filled(ImageFilledType::VerticalTop,num_arr[3] as f32)),
                    3 => sprite_render.set_type(ImageType::Filled(ImageFilledType::VerticalBottom,num_arr[4] as f32)),
                    _ => ()
                }
            },
            4 => sprite_render.set_type(ImageType::Tiled),
            _ => ()
        }
    };
    let js_color = RawJsValue(args[5]).to_value(ctx).unwrap();
    if let Some(num_arr) = js_color.array_get_number() {
        sprite_render.set_color(num_arr[0] as f32,num_arr[1] as f32,num_arr[2] as f32,num_arr[3] as f32);
    }

    let mut storage = world.write_storage::<SpriteRender>();
    if storage.contains(entity) {
        return RawJsValue::val_bool(false)
    }
    storage.insert(entity,sprite_render).unwrap();
    let mut mesh_storage = world.write_storage::<Mesh2D>();
    mesh_storage.insert(entity,Mesh2D::default()).unwrap();
    RawJsValue::val_bool(true)
}

pub unsafe extern "C" fn c_add_text_render(ctx: *mut q::JSContext,_: q::JSValue,count:c_int,argv: *mut q::JSValue) -> q::JSValue {
    let args = std::slice::from_raw_parts(argv, count as usize);
    let world: &mut World = std::mem::transmute(q::JS_GetOpaque(args[0],WORLD_CLASS.as_ref().unwrap().class_id()));
    let entity = get_entity(world, args[1], ctx).unwrap();
    let font_id = RawJsValue::deserialize_value(args[2],ctx).unwrap().as_int().unwrap();
    let mut text_render = TextRender::new(Handle::new(font_id as u32));
    let mut storage = world.write_storage::<TextRender>();
    if storage.contains(entity) {
        return RawJsValue::val_bool(false)
    }
    let js_text =  RawJsValue(args[3]).to_value(ctx).unwrap().into_string();
    if let Some(text) = js_text {
        text_render.set_text(&text);
    }
    let js_color = RawJsValue(args[4]).to_value(ctx).unwrap();
    if let Some(num_arr) = js_color.array_get_number() {
        text_render.set_color(num_arr[0] as f32,num_arr[1] as f32,num_arr[2] as f32,num_arr[3] as f32);
    }
    let js_font_size = RawJsValue(args[5]).to_value(ctx).unwrap().as_int();
    if let Some(font_size) = js_font_size {
        text_render.set_font_size(font_size);
    }
    let js_line_mode = RawJsValue(args[6]).to_value(ctx).unwrap().as_int();
    if let Some(line_mode) = js_line_mode {
        if line_mode == 0 {
            text_render.set_line_mode(LineMode::Single);
        } else {
            text_render.set_line_mode(LineMode::Wrap);
        }
    }
    storage.insert(entity,text_render).unwrap();
    let mut mesh_storage = world.write_storage::<Mesh2D>();
    mesh_storage.insert(entity,Mesh2D::default()).unwrap();
    RawJsValue::val_null()
}

pub unsafe extern "C" fn c_add_transparent(ctx: *mut q::JSContext,_: q::JSValue,count:c_int,argv: *mut q::JSValue) -> q::JSValue {
    let args = std::slice::from_raw_parts(argv, count as usize);
    let world: &mut World = std::mem::transmute(q::JS_GetOpaque(args[0],WORLD_CLASS.as_ref().unwrap().class_id()));
    let entity = get_entity(world, args[1], ctx).unwrap();
    let mut storage = world.write_storage::<Transparent>();
    if storage.contains(entity) {
        return RawJsValue::val_bool(false)
    }
    storage.insert(entity,Transparent).unwrap();
    RawJsValue::val_bool(true)
}

pub unsafe extern "C" fn c_set_parent(ctx: *mut q::JSContext,_: q::JSValue,count:c_int,argv: *mut q::JSValue) -> q::JSValue {
    let args = std::slice::from_raw_parts(argv, count as usize);
    let world: &mut World = std::mem::transmute(q::JS_GetOpaque(args[0],WORLD_CLASS.as_ref().unwrap().class_id()));
    let entity  = get_entity(world, args[1], ctx).unwrap();
    let pentity = get_entity(world, args[2], ctx).unwrap();
    let mut storage = world.write_storage::<Parent>();
    if !storage.contains(entity) {
       let p = Parent {entity:pentity };
       storage.insert(entity,p).unwrap();
       return RawJsValue::val_null();
    }
    let cur_p = storage.get_mut(entity).unwrap();
    cur_p.entity = pentity;
    RawJsValue::val_null()
}

pub unsafe extern "C" fn c_get_sprite_rect_info(ctx: *mut q::JSContext,_: q::JSValue,count:c_int,argv: *mut q::JSValue) -> q::JSValue {
    let args = std::slice::from_raw_parts(argv, count as usize);
    let world: &mut World = std::mem::transmute(q::JS_GetOpaque(args[0],WORLD_CLASS.as_ref().unwrap().class_id()));
    let sheet_id = RawJsValue(args[1]).to_value(ctx).unwrap().as_int().unwrap();
    let sprite_name = RawJsValue(args[2]).to_value(ctx).unwrap().into_string().unwrap();
    let may_spr:Option<Sprite> = world.fetch_mut::<AssetStorage<SpriteSheet>>().get(&Handle::new(sheet_id as u32))
                       .and_then(|s| s.get_sprite(&sprite_name).map(|v| v.clone()) );
    if let Some(spr) = may_spr {
        let val = JSValue::Array(vec![JSValue::Float(spr.rect.x      as f64),
                                      JSValue::Float(spr.rect.y      as f64),
                                      JSValue::Float(spr.rect.width  as f64),
                                      JSValue::Float(spr.rect.height as f64)]);
        return val.to_c_value(ctx);
    }
    RawJsValue::val_null()
}

pub unsafe extern "C" fn set_transform_behavior(ctx: *mut q::JSContext,_: q::JSValue,count: c_int,argv: *mut q::JSValue) -> q::JSValue {
    let (qworld,entity,b_map,q_ctx) = get_w_e_m_q(argv,count,ctx);
    if let Some(b_js_pos)  =  b_map.get(&String::from("pos")) {
        let behavior: &mut Behavior<QJSValue> = std::mem::transmute(q::JS_GetOpaque(b_js_pos.inner().0,BEHAVIOR_CLASS.as_ref().unwrap().class_id()));
        let update_pos = move |val:QJSValue| {
            let mut_world:&mut World = std::mem::transmute(qworld.0);
            let arr = RawJsValue(val.0).to_value(q_ctx.0).unwrap().array_get_number().unwrap();
            let mut trans_storage = mut_world.write_storage::<Transform>();
            let t:&mut Transform = trans_storage.get_mut(entity).unwrap();
            t.set_position(Vector3::new(arr[0] as f32,arr[1] as f32,arr[2] as f32));
            update_mesh_2d(mut_world,entity);
        };
        update_pos(behavior.value());
        behavior.set_callback(move |val| {
            update_pos(val);
        })
    }
    let q2 = QJSContext(ctx);
    if let Some(b_js_scale)  =  b_map.get(&String::from("scale")) {
        let behavior: &mut Behavior<QJSValue> = std::mem::transmute(q::JS_GetOpaque(b_js_scale.inner().0,BEHAVIOR_CLASS.as_ref().unwrap().class_id()));
        let update_scale = move |val:QJSValue| {
            let mut_world:&mut World = std::mem::transmute(qworld.0);
            let arr = RawJsValue(val.0).to_value(q2.0).unwrap().array_get_number().unwrap();
            let mut trans_storage = mut_world.write_storage::<Transform>();
            let t:&mut Transform = trans_storage.get_mut(entity).unwrap();
            t.set_scale(Vector3::new(arr[0] as f32,arr[1] as f32,arr[2] as f32));
            update_mesh_2d(mut_world,entity);
        };
        update_scale(behavior.value());
        behavior.set_callback(move |val| {
            update_scale(val);
        })
    }
   
    RawJsValue::val_null()
}

pub unsafe extern "C" fn set_rect2d_behavior(ctx: *mut q::JSContext,_: q::JSValue,count: c_int,argv: *mut q::JSValue) -> q::JSValue {
    let (qworld,entity,b_map,q_ctx) = get_w_e_m_q(argv,count,ctx);
    if let Some(b_js_size)  =  b_map.get(&String::from("size")) {
        let behavior: &mut Behavior<QJSValue> = std::mem::transmute(q::JS_GetOpaque(b_js_size.inner().0,BEHAVIOR_CLASS.as_ref().unwrap().class_id()));
        let update_size = move |val:QJSValue| {
            let mut_world:&mut World = std::mem::transmute(qworld.0);
            let arr = RawJsValue(val.0).to_value(q_ctx.0).unwrap().array_get_number().unwrap();
            let mut rect_storage = mut_world.write_storage::<Rect2D>();
            let rect:&mut Rect2D = rect_storage.get_mut(entity).unwrap();
            rect.width  = arr[0] as f32;
            rect.height = arr[1] as f32;
            update_mesh_2d(mut_world,entity);
        };
        update_size(behavior.value());
        behavior.set_callback(move |val| {
            update_size(val);
        })
    }
    let q2 = QJSContext(ctx);
    if let Some(b_js_anchor)  =  b_map.get(&String::from("anchor")) {
        let behavior: &mut Behavior<QJSValue> = std::mem::transmute(q::JS_GetOpaque(b_js_anchor.inner().0,BEHAVIOR_CLASS.as_ref().unwrap().class_id()));
        let update_anchor = move |val:QJSValue| {
            let mut_world:&mut World = std::mem::transmute(qworld.0);
            let arr = RawJsValue(val.0).to_value(q2.0).unwrap().array_get_number().unwrap();
            let mut rect_storage = mut_world.write_storage::<Rect2D>();
            let rect:&mut Rect2D = rect_storage.get_mut(entity).unwrap();
            rect.anchor[0]  = arr[0] as f32;
            rect.anchor[1]  = arr[1] as f32;
            update_mesh_2d(mut_world,entity);
        };
        update_anchor(behavior.value());
        behavior.set_callback(move |val| {
            update_anchor(val);
        })
    }
    RawJsValue::val_null()
}

pub unsafe extern "C" fn set_image_render_behavior(ctx: *mut q::JSContext,_: q::JSValue,count: c_int,argv: *mut q::JSValue) -> q::JSValue {
    let (qworld,entity,b_map,q_ctx) = get_w_e_m_q(argv,count,ctx);
    if let Some(b_js_color)  =  b_map.get(&String::from("color")) {
        let behavior: &mut Behavior<QJSValue> = std::mem::transmute(q::JS_GetOpaque(b_js_color.inner().0,BEHAVIOR_CLASS.as_ref().unwrap().class_id()));
        let update_color = move |val:QJSValue| {
            let mut_world:&mut World = std::mem::transmute(qworld.0);
            let arr = RawJsValue(val.0).to_value(q_ctx.0).unwrap().array_get_number().unwrap();
            let mut image_storage = mut_world.write_storage::<ImageRender>();
            let image:&mut ImageRender = image_storage.get_mut(entity).unwrap();
            image.set_color(arr[0] as f32,arr[1] as f32,arr[2] as f32,arr[3] as f32);
            update_mesh_2d(mut_world,entity);
        };
        update_color(behavior.value());
        behavior.set_callback(move|val| {
            update_color(val);
        })
    }
    RawJsValue::val_null()
}

pub unsafe extern "C" fn set_sprite_render_behavior(ctx: *mut q::JSContext,_: q::JSValue,count: c_int,argv: *mut q::JSValue) -> q::JSValue {
    let (qworld,entity,b_map,q_ctx) = get_w_e_m_q(argv,count,ctx);
    if let Some(b_js_sprite_name)  =  b_map.get(&String::from("spriteName")) {
        let behavior: &mut Behavior<QJSValue> = std::mem::transmute(q::JS_GetOpaque(b_js_sprite_name.inner().0,BEHAVIOR_CLASS.as_ref().unwrap().class_id()));
        let update_sprite_name = move |val:QJSValue| {
            let mut_world:&mut World = std::mem::transmute(qworld.0);
            let sprite_name = RawJsValue(val.0).to_value(q_ctx.0).unwrap().into_string().unwrap();
            let mut sprite_storage = mut_world.write_storage::<SpriteRender>();
            let sprite:&mut SpriteRender = sprite_storage.get_mut(entity).unwrap();
            sprite.set_sprite_name(sprite_name);
            update_mesh_2d(mut_world,entity);
        };
        update_sprite_name(behavior.value());
        behavior.set_callback(move|val| {
            update_sprite_name(val);
        })
    }
    if let Some(b_js_color)  =  b_map.get(&String::from("color")) {
        let q2 = QJSContext(ctx);
        let behavior: &mut Behavior<QJSValue> = std::mem::transmute(q::JS_GetOpaque(b_js_color.inner().0,BEHAVIOR_CLASS.as_ref().unwrap().class_id()));
        let update_color = move |val:QJSValue| {
            let mut_world:&mut World = std::mem::transmute(qworld.0);
            let arr = RawJsValue(val.0).to_value(q2.0).unwrap().array_get_number().unwrap();
            let mut sprite_storage = mut_world.write_storage::<SpriteRender>();
            let sprite:&mut SpriteRender = sprite_storage.get_mut(entity).unwrap();
            sprite.set_color(arr[0] as f32,arr[1] as f32,arr[2] as f32,arr[3] as f32);
            update_mesh_2d(mut_world,entity);
        };
        update_color(behavior.value());
        behavior.set_callback(move|val| {
            update_color(val);
        })
    }
    RawJsValue::val_null()
}

pub unsafe extern "C" fn set_text_render_behavior(ctx: *mut q::JSContext,_: q::JSValue,count: c_int,argv: *mut q::JSValue) -> q::JSValue {
    let (qworld,entity,b_map,q_ctx) = get_w_e_m_q(argv,count,ctx);
    if let Some(b_js_text)  =  b_map.get(&String::from("text")) {
        let behavior: &mut Behavior<QJSValue> = std::mem::transmute(q::JS_GetOpaque(b_js_text.inner().0,BEHAVIOR_CLASS.as_ref().unwrap().class_id()));
        let update_text = move |val:QJSValue| {
            let mut_world:&mut World = std::mem::transmute(qworld.0);
            let mut text_storage = mut_world.write_storage::<TextRender>();
            let text:&mut TextRender = text_storage.get_mut(entity).unwrap();
            let b_str = RawJsValue(val.0).to_value(q_ctx.0).unwrap().into_string().unwrap();
            text.set_text(&b_str);
        };
        update_text(behavior.value());
        behavior.set_callback(move |val| {
            update_text(val);
        })
    }
    if let Some(b_js_color)  =  b_map.get(&String::from("color")) {
        let q2 = QJSContext(ctx);
        let behavior: &mut Behavior<QJSValue> = std::mem::transmute(q::JS_GetOpaque(b_js_color.inner().0,BEHAVIOR_CLASS.as_ref().unwrap().class_id()));
        let update_color = move |val:QJSValue| {
            let mut_world:&mut World = std::mem::transmute(qworld.0);
            let arr = RawJsValue(val.0).to_value(q2.0).unwrap().array_get_number().unwrap();
            let mut text_storage = mut_world.write_storage::<TextRender>();
            let text:&mut TextRender = text_storage.get_mut(entity).unwrap();
            text.set_color(arr[0] as f32,arr[1] as f32,arr[2] as f32,arr[3] as f32);
        };
        update_color(behavior.value());
        behavior.set_callback(move |val| {
            update_color(val);
        })
    }
    if let Some(b_js_linemode)  =  b_map.get(&String::from("lineMode")) {
        let q3 = QJSContext(ctx);
        let behavior: &mut Behavior<QJSValue> = std::mem::transmute(q::JS_GetOpaque(b_js_linemode.inner().0,BEHAVIOR_CLASS.as_ref().unwrap().class_id()));
        let update_line_mode = move |val:QJSValue| {
            let mut_world:&mut World = std::mem::transmute(qworld.0);
            let line_mode_num = RawJsValue(val.0).to_value(q3.0).unwrap().as_int().unwrap();
            let mut text_storage = mut_world.write_storage::<TextRender>();
            let text:&mut TextRender = text_storage.get_mut(entity).unwrap();
            let line_mode = match line_mode_num {
                0 => LineMode::Single,
                _ => LineMode::Wrap
            };
            text.set_line_mode(line_mode);
        };
        update_line_mode(behavior.value());
        behavior.set_callback(move |val| {
            update_line_mode(val);
        });
    }
    if let Some(b_js_anchor)  =  b_map.get(&String::from("anchor")) {
        let q = QJSContext(ctx);
        let behavior: &mut Behavior<QJSValue> = std::mem::transmute(q::JS_GetOpaque(b_js_anchor.inner().0,BEHAVIOR_CLASS.as_ref().unwrap().class_id()));
        let update_anchor = move |val:QJSValue| {
            let mut_world:&mut World = std::mem::transmute(qworld.0);
            let anchor:AnchorAlign = AnchorAlign::from(RawJsValue(val.0).to_value(q.0).unwrap().as_int().unwrap() as u32);
            let mut text_storage = mut_world.write_storage::<TextRender>();
            let text:&mut TextRender = text_storage.get_mut(entity).unwrap();
            text.set_anchor(anchor);
        };
        update_anchor(behavior.value());
        behavior.set_callback(move |val| {
            update_anchor(val);
        });
    }
    if let Some(b_js_font_size)  =  b_map.get(&String::from("fontSize")) {
        let q = QJSContext(ctx);
        let behavior: &mut Behavior<QJSValue> = std::mem::transmute(q::JS_GetOpaque(b_js_font_size.inner().0,BEHAVIOR_CLASS.as_ref().unwrap().class_id()));
        let update_font_size = move |val:QJSValue| {
            let mut_world:&mut World = std::mem::transmute(qworld.0);
            let font_size = RawJsValue(val.0).to_value(q.0).unwrap().as_int().unwrap();
            let mut text_storage = mut_world.write_storage::<TextRender>();
            let text:&mut TextRender = text_storage.get_mut(entity).unwrap();
            text.set_font_size(font_size);
        };
        update_font_size(behavior.value());
        behavior.set_callback(move |val| {
            update_font_size(val);
        });
    }
    RawJsValue::val_null()
}

/*#region tools*/
unsafe fn get_w_e_m_q<'a>(argv: *mut q::JSValue,count:i32,ctx:*mut q::JSContext) -> (QJSWorld,Entity,HashMap<String,AutoDropJSValue>,QJSContext) {
    let args = std::slice::from_raw_parts(argv, count as usize);
    let world: *mut World = std::mem::transmute(q::JS_GetOpaque(args[0],WORLD_CLASS.as_ref().unwrap().class_id()));
    let entity = get_entity( std::mem::transmute(world),args[1],ctx).unwrap();
    let b_map = RawJsValue::deserialize_value(args[2],ctx).unwrap().into_object().unwrap();
    let q_ctx = QJSContext(ctx);
    (QJSWorld(world),entity,b_map,q_ctx)
}

unsafe fn set_vector3_array(arr:&mut Vector3<f32>,js_val:q::JSValue,ctx:*mut q::JSContext) {
    let may_num_arr = RawJsValue::deserialize_value(js_val,ctx).ok().and_then(|f| f.array_get_number());
    if let Some(num_arr) = may_num_arr {
        arr.x = *num_arr.get_unchecked(0) as f32;
        arr.y = *num_arr.get_unchecked(1) as f32;
        arr.z = *num_arr.get_unchecked(2) as f32;
    }   
}

fn update_mesh_2d(world:&World,entity:Entity) {
    let mut mesh_storage = world.write_storage::<Mesh2D>();
    let mesh:&mut Mesh2D = mesh_storage.get_mut(entity).unwrap();
    mesh.is_dirty = true;
}

unsafe fn event_ptr(val:q::JSValue) -> &'static mut Event<QJSValue> {
    std::mem::transmute(q::JS_GetOpaque(val,EVENT_CLASS.as_ref().unwrap().class_id()))
}


/*#endregion newImage*/
