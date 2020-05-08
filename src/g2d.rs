use crate::core::WORLD_CLASS;
use qjs_rs::{
    q, JSClass, JSClassOject, JSContext, JSPropertyItem, JSValue, RawJsValue,AutoDropJSValue,
};
use seija::rendy::texture::{image::{ImageTextureConfig}};
use seija::assets::{AssetLoadError, Handle,AssetStorage,TextuteLoaderInfo,SpriteSheetLoaderInfo,FontAssetLoaderInfo};
use seija::common::{Rect2D, Transform,transform::{Parent,component::{ParentHierarchy}}};
use seija::frp::{Event,Behavior};
use seija::math::Vector3;
use seija::module_bundle::{DefaultBackend, S2DLoader, Simple2d};
use seija::render::{
    Transparent,
    components::{ImageRender,SpriteRender,ImageGenericInfo,LineMode,TextRender,SpriteSheet,ImageType,Mesh2D,ImageFilledType,Sprite},
    types,
};
use seija::rendy::hal::image::{SamplerDesc,Filter,WrapMode};
use seija::window::{ViewPortSize};
use seija::event::{cb_event::{CABEventRoot},EventNode,GameEventType};
use seija::specs::{shred::FetchMut,Entity,World,WorldExt,world::{Builder}};
use seija::win::{dpi::LogicalSize, WindowBuilder};

use std::ffi::CString;
use std::os::raw::c_int;
use crate::core::{QJSContext,JSEventComponent};

pub static mut SIMPLE2D_CLASS: Option<JSClass> = None;
pub static mut LOADER_CLASS: Option<JSClass> = None;

static mut LOADER_REF_COUNT: u32 = 0;



pub unsafe fn g2d_init(ctx: &mut JSContext, m: *mut q::JSModuleDef) {
    SIMPLE2D_CLASS = Some(JSClass::new_full("Simple2d", ctx.c_rt(),Some(simple2d_finalizer),None,None));
    LOADER_CLASS = Some(JSClass::new_full("Loader",ctx.c_rt(),Some(loader_finalizer),None,None));
    
    let g2d_obj = q::JS_NewObject(ctx.c_ctx());
    let g2d_attrs = vec![
        JSPropertyItem::func(c_str!("newSimple2d"), Some(c_new_simple2d), 1),
        JSPropertyItem::func(c_str!("fetchLoader"), Some(c_fetch_loader), 1),
        JSPropertyItem::func(c_str!("loadSync"), Some(c_load_sync), 1),
        JSPropertyItem::func(c_str!("attachNodeEvent"),Some(c_attach_node_event) ,1),
        JSPropertyItem::func(c_str!("refCount"),Some(c_ref_count) ,1),
        JSPropertyItem::func(c_str!("getViewPortSize"),Some(c_get_view_port_size), 1),
        JSPropertyItem::func(c_str!("getTextureSize"),Some(c_get_texture_size), 1),
        JSPropertyItem::func(c_str!("getSpriteRectInfo"),Some(c_get_sprite_rect_info), 1),
        //entity
        JSPropertyItem::func(c_str!("newEntity"),Some(c_new_entity) ,1),
        JSPropertyItem::func(c_str!("setParent"),Some(c_set_parent), 1),
        JSPropertyItem::func(c_str!("removeAllChildren"),Some(c_remove_all_children), 1),
        JSPropertyItem::func(c_str!("destoryEntity"),Some(c_destory_entity) ,1),
        JSPropertyItem::func(c_str!("getChildrens"),Some(c_get_childrens) ,1),
        //component
        JSPropertyItem::func(c_str!("addCABEventRoot"),Some(c_add_cab_event_root) ,1),
        JSPropertyItem::func(c_str!("addRect2d"), Some(c_add_rect_2d), 1),
        JSPropertyItem::func(c_str!("addTransform"),Some(c_add_transform), 1),
        JSPropertyItem::func(c_str!("addImageRender"),Some(c_add_image_render), 1),
        JSPropertyItem::func(c_str!("addSpriteRender"),Some(c_add_sprite_render), 1),
        JSPropertyItem::func(c_str!("addTextRender"),Some(c_add_text_render), 1),
        JSPropertyItem::func(c_str!("addTransparent"),Some(c_add_transparent), 1),
        //component attr
        JSPropertyItem::func(c_str!("setTransform"),Some(set_transform),1),
        JSPropertyItem::func(c_str!("setRect2d"),Some(set_rect2d),1),
        JSPropertyItem::func(c_str!("setImageRender"),Some(set_image_render),1),
        JSPropertyItem::func(c_str!("setSpriteRender"),Some(set_sprite_render),1),
        JSPropertyItem::func(c_str!("setTextRender"),Some(set_text_render),1),
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
        //0 => loader.load_sync::<Value, DefaultBackend>(&asset_path, world).map(|h| h.id()),
        1 => {
            let cfg = parse_image_config(args[4], ctx);
            loader.load_sync::<_, DefaultBackend>(TextuteLoaderInfo::new(asset_path.as_str(),cfg), world).map(|h| h.id())
        },
        2 => {
            let cfg = parse_image_config(args[4], ctx);
            loader.load_sync::<_, DefaultBackend>(SpriteSheetLoaderInfo::new(&asset_path,cfg), world).map(|h| h.id())
        },
        3 => loader.load_sync::<_, DefaultBackend>(FontAssetLoaderInfo::new(&asset_path), world).map(|h| h.id()),
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

fn parse_image_config(val:q::JSValue,ctx:*mut q::JSContext) -> ImageTextureConfig {
    if let Some(obj) = RawJsValue::deserialize_value(val, ctx).ok().and_then(|v| v.into_object()) {
        let mut config:ImageTextureConfig = ImageTextureConfig::default();
        let js_s_info = obj.get(&String::from("sampler_info")).and_then(|v|v.inner().to_value(ctx).ok()).and_then(|v|v.as_array_int());
        if let Some(s_type) = js_s_info {
            match *s_type.as_slice() {
                [0,0] => config.sampler_info = SamplerDesc::new(Filter::Nearest, WrapMode::Tile),
                [0,1] => config.sampler_info = SamplerDesc::new(Filter::Nearest, WrapMode::Mirror),
                [0,2] => config.sampler_info = SamplerDesc::new(Filter::Nearest, WrapMode::Clamp),
                [0,3] => config.sampler_info = SamplerDesc::new(Filter::Nearest, WrapMode::Border),
                [1,0] => config.sampler_info = SamplerDesc::new(Filter::Linear, WrapMode::Tile),
                [1,1] => config.sampler_info = SamplerDesc::new(Filter::Linear, WrapMode::Mirror),
                [1,2] => config.sampler_info = SamplerDesc::new(Filter::Linear, WrapMode::Clamp),
                [1,3] => config.sampler_info = SamplerDesc::new(Filter::Linear, WrapMode::Border),
                _ => config.sampler_info = SamplerDesc::new(Filter::Linear, WrapMode::Clamp),
            };
        }
        if let Some(is_mips) = obj.get(&String::from("generate_mips")).and_then(|v|v.inner().to_value(ctx).ok()).and_then(|v|v.as_bool()) {
            config.generate_mips = is_mips;
        }
        if let Some(is_mips) = obj.get(&String::from("premultiply_alpha")).and_then(|v|v.inner().to_value(ctx).ok()).and_then(|v|v.as_bool()) {
            config.premultiply_alpha = is_mips;
        }
        return config;
    }
    ImageTextureConfig::default()
}



pub unsafe extern "C" fn c_attach_node_event(ctx: *mut q::JSContext,_: q::JSValue,count: c_int,argv: *mut q::JSValue) -> q::JSValue {
    let args = std::slice::from_raw_parts(argv, count as usize);
    let world: &mut World = std::mem::transmute(q::JS_GetOpaque(args[0],WORLD_CLASS.as_ref().unwrap().class_id()));
    let eid = RawJsValue(args[1]).to_value(ctx).unwrap().as_int().unwrap();
    let entity = world.entities().entity(eid as u32);
    let event_type_id = RawJsValue(args[2]).to_value(ctx).unwrap().as_int().unwrap() as u32;
    
    let mut js_component_storage = world.write_storage::<JSEventComponent>();
    let mut event_storage = world.write_storage::<EventNode>();
    if !js_component_storage.contains(entity) {
        let mut js_component = JSEventComponent::default();
        js_component.set_ctx(ctx);
        let val = q::JS_DupValue(args[4]);
        js_component.insert_node(event_type_id,val);
        js_component_storage.insert(entity,js_component).unwrap();
    } else {
        let js_component = js_component_storage.get_mut(entity).unwrap();
        let val = q::JS_DupValue(args[4]);
        js_component.insert_node(event_type_id,val);
    }
    if !event_storage.contains(entity) {
        let ev = EventNode::default();
        event_storage.insert(entity,ev).unwrap();
    };
    let is_capture = RawJsValue(args[3]).to_value(ctx).unwrap().as_bool().unwrap();
    let ev_typ = GameEventType::from(event_type_id).unwrap_or(GameEventType::TouchStart);
    let ev_node:&mut EventNode = event_storage.get_mut(entity).unwrap();
    let qctx = QJSContext(ctx);
    ev_node.register(is_capture,ev_typ,move |e,world| {
        let may_js_component = world.read_storage::<JSEventComponent>().get(e).unwrap().ev_nodes.get(&event_type_id).map(|v|*v);
        if let Some(js_component) = may_js_component {
          let fire_func_name = CString::new("onFire").unwrap();
          let fire_func = q::JS_GetPropertyStr(qctx.0, js_component, fire_func_name.as_ptr());
          let mut js_e_val = JSValue::Int(e.id() as i32).to_c_value(qctx.0);
          q::JS_Call(qctx.0,fire_func,js_component,1,&mut js_e_val);
          AutoDropJSValue::drop_js_value(fire_func,qctx.0);
        }
    });
    RawJsValue::val_null()
}



pub unsafe extern "C" fn c_new_entity(_ctx: *mut q::JSContext,_: q::JSValue,count: c_int,argv: *mut q::JSValue) -> q::JSValue {
    let args = std::slice::from_raw_parts(argv, count as usize);
    let world: &mut World = std::mem::transmute(q::JS_GetOpaque(args[0],WORLD_CLASS.as_ref().unwrap().class_id()));
    
    let e = world.create_entity().build();
    RawJsValue::val_i32(e.id() as i32)
}

pub unsafe extern "C" fn c_ref_count(_ctx: *mut q::JSContext,_: q::JSValue,_count: c_int,argv: *mut q::JSValue) -> q::JSValue {
    let raw_js = RawJsValue(*argv);
    RawJsValue::val_i32(raw_js.ref_count())
}



pub unsafe extern "C" fn c_destory_entity(ctx: *mut q::JSContext,_: q::JSValue,count: c_int,argv: *mut q::JSValue) -> q::JSValue {
    let args = std::slice::from_raw_parts(argv, count as usize);
    let world: &mut World = std::mem::transmute(q::JS_GetOpaque(args[0],WORLD_CLASS.as_ref().unwrap().class_id()));
    let e = get_entity(world,args[1], ctx).unwrap();
    world.entities().delete(e).unwrap();
   
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
        let may_num_arr = RawJsValue::deserialize_value(args[3],ctx).ok().and_then(|f| f.as_array_number());
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
    if let Some(num_arr) = js_type.as_array_number() {
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
    if let Some(num_arr) = js_color.as_array_number() {
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
    if let Some(num_arr) = js_color.as_array_number() {
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

pub unsafe extern "C" fn c_remove_all_children(ctx: *mut q::JSContext,_: q::JSValue,count:c_int,argv: *mut q::JSValue) -> q::JSValue {
    let args = std::slice::from_raw_parts(argv, count as usize);
    let world: &mut World = std::mem::transmute(q::JS_GetOpaque(args[0],WORLD_CLASS.as_ref().unwrap().class_id()));
    let entity  = get_entity(world, args[1], ctx).unwrap();
    let hierarchy = world.fetch_mut::<ParentHierarchy>();
    let entities = world.entities();
    for ce in hierarchy.all_children_iter(entity) {
        entities.delete(ce).unwrap();
    }
    RawJsValue::val_null()
}

pub unsafe extern "C" fn c_get_childrens(ctx: *mut q::JSContext,_: q::JSValue,count:c_int,argv: *mut q::JSValue) -> q::JSValue {
    let args = std::slice::from_raw_parts(argv, count as usize);
    let world: &mut World = std::mem::transmute(q::JS_GetOpaque(args[0],WORLD_CLASS.as_ref().unwrap().class_id()));
    let entity  = get_entity(world, args[1], ctx).unwrap();
    let hierarchy = world.fetch_mut::<ParentHierarchy>();
    let mut arr:Vec<JSValue> = vec![];
    for ce in hierarchy.all_children_iter(entity) {
        arr.push(JSValue::Int(ce.id() as i32));
    }
    JSValue::Array(arr).to_c_value(ctx)
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

pub unsafe extern "C" fn set_transform(ctx: *mut q::JSContext,_: q::JSValue,count: c_int,argv: *mut q::JSValue) -> q::JSValue {
    let (world,entity,attr_type,value) = get_w_e_t_v(argv,count,ctx);
    let mut trans_storage = (&*world).write_storage::<Transform>();
    let trans:&mut Transform = trans_storage.get_mut(entity).unwrap();
    match attr_type {
        0 => {
            let arr = RawJsValue(value).to_value(ctx).unwrap().as_array_number().unwrap();
            trans.set_position(Vector3::new(arr[0] as f32,arr[1] as f32,arr[2] as f32));
        },
        1 => {
            let arr = RawJsValue(value).to_value(ctx).unwrap().as_array_number().unwrap();
            trans.set_scale(Vector3::new(arr[0] as f32,arr[1] as f32,arr[2] as f32));
        },
        2 => {
            let arr = RawJsValue(value).to_value(ctx).unwrap().as_array_number().unwrap();
            trans.set_rotation_euler(arr[0] as f32,arr[1] as f32,arr[2] as f32);
        },
        _ => ()
    };
    update_mesh_2d(&*world, entity);
    RawJsValue::val_null()
}


pub unsafe extern "C" fn set_rect2d(ctx: *mut q::JSContext,_: q::JSValue,count: c_int,argv: *mut q::JSValue) -> q::JSValue {
    let (world,entity,attr_type,value) = get_w_e_t_v(argv,count,ctx);
    let mut rect_storage = (&*world).write_storage::<Rect2D>();
    let rect:&mut Rect2D = rect_storage.get_mut(entity).unwrap();
    match attr_type {
        0 => {
            let arr = RawJsValue(value).to_value(ctx).unwrap().as_array_number().unwrap();
            rect.width  = arr[0] as f32;
            rect.height = arr[1] as f32;
        },
        1 => {
            let arr = RawJsValue(value).to_value(ctx).unwrap().as_array_number().unwrap();
            rect.anchor[0] = arr[0] as f32;
            rect.anchor[1] = arr[1] as f32;
        },
        _ => ()
    };
    update_mesh_2d(&*world, entity);
    RawJsValue::val_null()
}



pub unsafe extern "C" fn set_image_render(ctx: *mut q::JSContext,_: q::JSValue,count: c_int,argv: *mut q::JSValue) -> q::JSValue {
    let (world,entity,attr_type,value) = get_w_e_t_v(argv,count,ctx);
    let mut render_storage = (&*world).write_storage::<ImageRender>();
    let render:&mut ImageRender = render_storage.get_mut(entity).unwrap();
    match attr_type {
        0 => {
            let arr = RawJsValue(value).to_value(ctx).unwrap().as_array_number().unwrap();
            render.set_color(arr[0] as f32, arr[1] as f32, arr[2] as f32, arr[3] as f32); 
        },
        1 => {
            let tex_id = RawJsValue(value).to_value(ctx).unwrap().as_int().unwrap();
            render.set_texture(Handle::new(tex_id as u32));
        },
        2 => {
            let arr = RawJsValue(value).to_value(ctx).unwrap().as_array_number().unwrap();
            set_image_type(render.info_mut(),arr,&*world);
        },
        _ => ()
    };
    update_mesh_2d(&*world, entity);
    RawJsValue::val_null()
}



pub unsafe extern "C" fn set_sprite_render(ctx: *mut q::JSContext,_: q::JSValue,count: c_int,argv: *mut q::JSValue) -> q::JSValue {
    let (world,entity,attr_type,value) = get_w_e_t_v(argv,count,ctx);
    let mut render_storage = (&*world).write_storage::<SpriteRender>();
    let render:&mut SpriteRender = render_storage.get_mut(entity).unwrap();
    match attr_type {
        0 => {
            let arr = RawJsValue(value).to_value(ctx).unwrap().as_array_number().unwrap();
            render.set_color(arr[0] as f32, arr[1] as f32, arr[2] as f32, arr[3] as f32); 
        },
        1 => {
            let spr_name = RawJsValue(value).to_value(ctx).unwrap().into_string().unwrap();
            render.set_sprite_name(spr_name);
        },
        2 => {
            let arr = RawJsValue(value).to_value(ctx).unwrap().as_array_number().unwrap();
            let typ = arr[0] as i32;
            if typ == 2 {
                set_image_type(render.info_mut(),arr,&*world);
            } else {
                render.set_slice_type_by_cfg(arr[1] as usize,&(&*world).fetch::<AssetStorage<SpriteSheet>>());
            }
            
        },
        3 => {
            let fill_value = RawJsValue(value).to_value(ctx).unwrap().as_number().unwrap();
            render.set_fill_value(fill_value as f32);
        },
        4 => {
            let sheet = RawJsValue(value).to_value(ctx).unwrap().as_int().unwrap();
            render.sprite_sheet = Handle::new(sheet as u32);
        },
        _ => ()
    };
    update_mesh_2d(&*world, entity);
    RawJsValue::val_null()
}



pub unsafe extern "C" fn set_text_render(ctx: *mut q::JSContext,_: q::JSValue,count: c_int,argv: *mut q::JSValue) -> q::JSValue {
    let (world,entity,attr_type,value) = get_w_e_t_v(argv,count,ctx);
    let mut text_storage = (&*world).write_storage::<TextRender>();
    let render:&mut TextRender = text_storage.get_mut(entity).unwrap();
    match attr_type {
        0 => {
            let text = RawJsValue(value).to_value(ctx).unwrap().into_string().unwrap();
            render.set_text(text.as_str());
        },
        1 => {
            let font_size = RawJsValue(value).to_value(ctx).unwrap().as_int().unwrap();
            render.set_font_size(font_size);
        },
        2 => {
            let arr = RawJsValue(value).to_value(ctx).unwrap().as_array_number().unwrap();
            render.set_color(arr[0] as f32, arr[1] as f32, arr[2] as f32, arr[3] as f32);
        },
        3 => {
            let anchor = RawJsValue(value).to_value(ctx).unwrap().as_int().unwrap() as u32;
            render.set_anchor(anchor.into());
        },
        4 => {
            let line_mode = RawJsValue(value).to_value(ctx).unwrap().as_int().unwrap() as u32;
            if line_mode == 0 {
                render.set_line_mode(LineMode::Single);
            } else {
                render.set_line_mode(LineMode::Wrap);
            }
        },
        5 => {
            let font_id = RawJsValue(value).to_value(ctx).unwrap().as_int().unwrap() as u32;
            render.font = Handle::new(font_id);
        }
        _ => ()
    };
    RawJsValue::val_null()
}


/*#region tools*/
unsafe fn get_w_e_t_v<'a>(argv: *mut q::JSValue,count:i32,ctx:*mut q::JSContext) -> (*mut World,Entity,i32,q::JSValue) {
    let args = std::slice::from_raw_parts(argv, count as usize);
    let world: *mut World = std::mem::transmute(q::JS_GetOpaque(args[0],WORLD_CLASS.as_ref().unwrap().class_id()));
    let entity = get_entity(&mut *world,args[1],ctx).unwrap();
    let attr_type = RawJsValue(args[2]).to_value(ctx).unwrap().as_int().unwrap();
    
    (world,entity,attr_type,args[3])
}

unsafe fn set_vector3_array(arr:&mut Vector3<f32>,js_val:q::JSValue,ctx:*mut q::JSContext) {
    let may_num_arr = RawJsValue::deserialize_value(js_val,ctx).ok().and_then(|f| f.as_array_number());
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


fn set_image_type(info:&mut ImageGenericInfo,num_arr:Vec<f64>,world:&World) {
        let typ = num_arr[0] as i32;
        match typ {
            0 => info.typ = ImageType::Simple,
            1 => info.typ = ImageType::Sliced(num_arr[1] as f32,num_arr[2] as f32,num_arr[3] as f32,num_arr[4] as f32),
            3 => {
                let ftyp = num_arr[1] as i32;
                match ftyp {
                    0 => info.typ = ImageType::Filled(ImageFilledType::HorizontalLeft,num_arr[1] as f32),
                    1 => info.typ = ImageType::Filled(ImageFilledType::HorizontalRight,num_arr[2] as f32),
                    2 => info.typ = ImageType::Filled(ImageFilledType::VerticalTop,num_arr[3] as f32),
                    3 => info.typ = ImageType::Filled(ImageFilledType::VerticalBottom,num_arr[4] as f32),
                    _ => ()
                }
            },
            4 => info.typ = ImageType::Tiled,
            _ => ()
        }
}

/*#endregion newImage*/
