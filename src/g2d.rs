use crate::core::WORLD_CLASS;
use qjs_rs::{
    q, JSClass, JSClassOject, JSContext, JSPropertyItem, JSValue, RawJsValue,
};
use seija::assets::{errors::AssetLoadError, Handle,AssetStorage};
use seija::common::{Rect2D, Transform,transform::{Parent}};
use seija::g2d::Image;
use seija::frp::{FRPNode,Event,Behavior};
use seija::json::Value;
use seija::math::Vector3;
use seija::module_bundle::{DefaultBackend, S2DLoader, Simple2d};
use seija::render::{
    components::{ImageRender, SpriteSheet,ImageType},
    types, FontAsset,
};
use seija::window::{ViewPortSize};
use seija::event::{cb_event::{CABEventRoot},EventNode,GameEventType};
use seija::specs::{shred::FetchMut,Entity,World,WorldExt,world::{Builder}};
use seija::win::{dpi::LogicalSize, WindowBuilder};
//use seija::event::{EventNode};
use std::os::raw::c_int;
use std::sync::{Arc};
use crate::core::{JSFRPNode,QJSValue,QJSContext};

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

    let app_obj = q::JS_NewObject(ctx.c_ctx());
    let app_attrs = vec![
        JSPropertyItem::func(c_str!("newSimple2d"), Some(c_new_simple2d), 1),
        JSPropertyItem::func(c_str!("fetchLoader"), Some(c_fetch_loader), 1),
        JSPropertyItem::func(c_str!("loadSync"), Some(c_load_sync), 1),
        JSPropertyItem::func(c_str!("newImage"), Some(c_new_image), 1),
        JSPropertyItem::func(c_str!("getEvent"),Some(c_get_event) ,1),
        JSPropertyItem::func(c_str!("chainEvent"),Some(c_chain_event) ,1),
        JSPropertyItem::func(c_str!("newEntity"),Some(c_new_entity) ,1),
        JSPropertyItem::func(c_str!("refCount"),Some(c_ref_count) ,1),
        JSPropertyItem::func(c_str!("destoryEntity"),Some(c_destory_entity) ,1),
        JSPropertyItem::func(c_str!("newBehavior"),Some(c_new_behavior) ,1),
        JSPropertyItem::func(c_str!("attachBehavior"),Some(c_attach_behavior) ,1),
        JSPropertyItem::func(c_str!("behaviorSetFoldFunc"),Some(behavior_set_fold_func), 1),
        JSPropertyItem::func(c_str!("setTransformPositonB"),Some(set_transform_positon_b), 1),
        JSPropertyItem::func(c_str!("getViewPortSize"),Some(c_get_view_port_size), 1),

        JSPropertyItem::func(c_str!("getTextureSize"),Some(c_get_texture_size), 1),
        JSPropertyItem::func(c_str!("setParent"),Some(c_set_parent), 1),
        //component
        JSPropertyItem::func(c_str!("addCABEventRoot"),Some(c_add_cab_event_root) ,1),
        JSPropertyItem::func(c_str!("addRect2d"), Some(c_add_rect_2d), 1),
        JSPropertyItem::func(c_str!("addTransform"),Some(c_add_transform), 1),
        JSPropertyItem::func(c_str!("addImageRender"),Some(c_add_image_render), 1)
    ];
    ctx.set_property_function_list(app_obj, &app_attrs);
    q::JS_SetModuleExport(ctx.c_ctx(), m, c_str!("g2d").as_ptr(), app_obj);
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

unsafe extern "C" fn behavior_finalizer(_rt: *mut q::JSRuntime, _val: q::JSValue) {
    dbg!("behavior finalizer");
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

/*#region newImage*/
//TODO delete it
pub unsafe extern "C" fn c_new_image(ctx: *mut q::JSContext,_: q::JSValue,count: c_int,argv: *mut q::JSValue) -> q::JSValue {
    let args = std::slice::from_raw_parts(argv, count as usize);
    let world: &mut World = std::mem::transmute(q::JS_GetOpaque(
        args[0],
        WORLD_CLASS.as_ref().unwrap().class_id(),
    ));
    let tex_id = RawJsValue::deserialize_value(args[1], ctx).unwrap().as_int().unwrap();
    let mut image_render = ImageRender::new(Handle::new(tex_id as u32));
    let mut t = Transform::default();
    let mut rect: Rect2D = Rect2D {
        width: 100f32,
        height: 100f32,
        anchor: [0.5f32, 0.5f32],
    };
    let mut transparent = true;
    let mut is_has_w = false;
    let mut is_has_h = false;
    let js_parent = RawJsValue::deserialize_value(args[2],ctx).ok().unwrap();
    let may_e = match js_parent {
        JSValue::Int(eid) => {
            Some(world.entities().entity(eid as u32))
        },
        _ => None,
    };
    if count > 3 {
        let js_object = RawJsValue::deserialize_value(args[3], ctx)
            .ok()
            .and_then(|jo| jo.into_object());
        let object_map = &js_object.as_ref();

        
        let may_rect_js_value = object_map.and_then(|o| o.get(&String::from("rect")))
                                          .and_then(|js_val| RawJsValue::deserialize_value(js_val.inner().0, ctx).ok());
        if let Some(rect_js_value) = may_rect_js_value.as_ref() {
            if let Some(width) = rect_js_value.object_get_number("width", ctx) {
                rect.width = width as f32;
                is_has_w = true;
            }
            if let Some(height) = rect_js_value.object_get_number("height", ctx) {
                rect.height = height as f32;
                is_has_h = true;
            }
        }
        let may_t_map = object_map.and_then(|o| o.get(&String::from("trans")))
                                  .and_then(|js_val| RawJsValue::deserialize_value(js_val.inner().0, ctx).ok())
                                  .and_then(|v| v.into_object());
        if let Some(t_map) = may_t_map.as_ref() {
            if let Some(pos) = t_map.get(&String::from("pos")) {
                let pos_arr = RawJsValue::deserialize_value(pos.inner().0, ctx).ok().and_then(|arr_val| arr_val.array_get_number());
                if let Some(number_arr) = pos_arr {
                    let x = number_arr.get(0).map(|n| *n as f32).unwrap_or(0f32);
                    let y = number_arr.get(1).map(|n| *n as f32).unwrap_or(0f32);
                    let z = number_arr.get(2).map(|n| *n as f32).unwrap_or(0f32);
                    t.set_position(Vector3::new(x, y, z));
                }
            }
            if let Some(scale) = t_map.get(&String::from("scale")) {
                let scale_arr = RawJsValue::deserialize_value(scale.inner().0, ctx).ok().and_then(|arr_val| arr_val.array_get_number());
                if let Some(number_arr) = scale_arr {
                    let sx = number_arr.get(0).map(|n| *n as f32).unwrap_or(1f32);
                    let sy = number_arr.get(1).map(|n| *n as f32).unwrap_or(1f32);
                    t.set_scale(Vector3::new(sx, sy, 1f32));
                }
            }
        }

        let may_color_array = object_map.and_then(|o| o.get(&String::from("color")))
                                        .and_then(|js_val| RawJsValue::deserialize_value(js_val.inner().0, ctx).ok())
                                        .and_then(|v| v.array_get_number());
        if let Some(color_array) = may_color_array {
            let r = color_array.get(0).map(|n| *n as f32).unwrap_or(0f32);
            let g = color_array.get(1).map(|n| *n as f32).unwrap_or(0f32);
            let b = color_array.get(2).map(|n| *n as f32).unwrap_or(0f32);
            let a = color_array.get(2).map(|n| *n as f32).unwrap_or(0f32);
            image_render.set_color(r,g,b,a);
        }

        let may_transparent = object_map.and_then(|o| o.get(&String::from("transparent")))
                           .and_then(|js_val| RawJsValue::deserialize_value(js_val.inner().0, ctx).ok())
                           .and_then(|val| val.as_bool());
        if let Some(is_transparent) = may_transparent {
            transparent = is_transparent;
        }
    }
    if !is_has_w || !is_has_h {
        let storage = world.fetch::<AssetStorage<types::Texture>>();
        if let Some(tex_ref) = storage.get_by_id(tex_id as u32) {
            let (w,h) = tex_ref.texture_size();
            if !is_has_w {
                rect.width = w as f32;
            }
            if !is_has_h {
                rect.height = h as f32;
            }
        }
    }
    let image = Image::create(world, rect, t, image_render,transparent,may_e);
    RawJsValue::val_i32(image.id() as i32)
}
/*#endregion newImage*/

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
    };
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
    let (w,h) = {
        let view_size = world.fetch::<ViewPortSize>();
        (view_size.width() as f32,view_size.height() as f32)
    };
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

pub unsafe extern "C" fn c_attach_behavior(_ctx: *mut q::JSContext,_: q::JSValue,count: c_int,argv: *mut q::JSValue) -> q::JSValue {
    let args = std::slice::from_raw_parts(argv, count as usize);
    let event: &mut Event<QJSValue> = std::mem::transmute(q::JS_GetOpaque(args[0],EVENT_CLASS.as_ref().unwrap().class_id()));
    let behavior: &mut Behavior<QJSValue> = std::mem::transmute(q::JS_GetOpaque(args[1],BEHAVIOR_CLASS.as_ref().unwrap().class_id()));
    
    let b_js = RawJsValue(args[1]);
    b_js.add_ref_count(1);
    event.attach_behavior(Box::from_raw(behavior));
    RawJsValue::val_bool(true)
}

pub unsafe extern "C" fn behavior_set_fold_func(ctx: *mut q::JSContext,_: q::JSValue,count: c_int,argv: *mut q::JSValue) -> q::JSValue {
    let args = std::slice::from_raw_parts(argv, count as usize);
    let behavior: &mut Behavior<QJSValue> = std::mem::transmute(q::JS_GetOpaque(args[0],BEHAVIOR_CLASS.as_ref().unwrap().class_id()));
    behavior.set_func(args[1].into());
    RawJsValue::val_null()
}

pub unsafe extern "C" fn set_transform_positon_b(ctx: *mut q::JSContext,_: q::JSValue,count: c_int,argv: *mut q::JSValue) -> q::JSValue {
    let args = std::slice::from_raw_parts(argv, count as usize);
    let world: &mut World = std::mem::transmute(q::JS_GetOpaque(args[0],WORLD_CLASS.as_ref().unwrap().class_id()));
    let behavior: &mut Behavior<QJSValue> = std::mem::transmute(q::JS_GetOpaque(args[1],BEHAVIOR_CLASS.as_ref().unwrap().class_id()));
    let entity = get_entity(world,args[2],ctx).unwrap();
    let q_ctx = QJSContext(ctx);
    behavior.set_callback(move |val|{
     
      let js_val:JSValue = RawJsValue::deserialize_value(val.0,q_ctx.0).unwrap();
      if let Some(pos_arr) = js_val.as_array() {
        let x = pos_arr.get_unchecked(0).as_number().unwrap_or(0f64);
        let y = pos_arr.get_unchecked(1).as_number().unwrap_or(0f64);
        let z = pos_arr.get_unchecked(2).as_number().unwrap_or(0f64);
        let mut trans_storage = world.write_storage::<Transform>();
        let t = trans_storage.get_mut(entity).unwrap();
        t.set_position(Vector3::new(x as f32,y as  f32,z as f32));
      }
    });
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
    storage.insert(entity,image_render).unwrap();
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


/*#region tools*/
unsafe fn set_vector3_array(arr:&mut Vector3<f32>,js_val:q::JSValue,ctx:*mut q::JSContext) {
    let may_num_arr = RawJsValue::deserialize_value(js_val,ctx).ok().and_then(|f| f.array_get_number());
    if let Some(num_arr) = may_num_arr {
        arr.x = *num_arr.get_unchecked(0) as f32;
        arr.y = *num_arr.get_unchecked(1) as f32;
        arr.z = *num_arr.get_unchecked(2) as f32;
    }   
}

/*#endregion newImage*/
