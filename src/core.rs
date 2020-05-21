use seija::core::IGame;
use seija::specs::{World,WorldExt,Builder,Component,DenseVecStorage};
use seija::common::{Transform,UpdateCallBack};
use seija::window::{ViewPortSize};
use seija::render::{ActiveCamera,Camera};
use seija::module_bundle::{S2DLoader};
use seija::event::{GameEventCallBack,GameEvent};
use std::collections::HashMap;
use std::ffi::CString;
use qjs_rs::{JSValue,q,RawJsValue,AutoDropJSValue,JSClass,JSClassOject};

pub static mut WORLD_CLASS:Option<JSClass> = None;

pub struct JSGame {
    ctx:*mut q::JSContext,
    on_start:Option<AutoDropJSValue>,
    on_update:Option<AutoDropJSValue>,
    on_quit:Option<AutoDropJSValue>,
    world_object:JSClassOject,
    res_path:String,
}

impl JSGame {
    pub fn new(js_val:JSValue,ctx:*mut q::JSContext) -> Self {
        let mut on_start:Option<AutoDropJSValue> = None;
        let mut on_update:Option<AutoDropJSValue> = None;
        let mut on_quit:Option<AutoDropJSValue> = None;
        let mut res_path = String::from("");
        let world_class:&JSClass = unsafe {  WORLD_CLASS.as_ref().unwrap() };
        let js_object = world_class.new_object(ctx);
        match js_val {
            JSValue::Object(mut obj) => {
                on_start = obj.remove(&String::from("OnStart"));
                on_update = obj.remove(&String::from("OnUpdate"));
                on_quit = obj.remove(&String::from("OnQuit"));
                res_path = obj.remove(&String::from("ResPath"))
                              .and_then(|v:AutoDropJSValue| v.inner().to_value(ctx).ok())
                              .map(|val| val.to_string()).unwrap_or(String::from("./Res"));
            },
            _ => {}
        }
        JSGame {
            ctx,
            on_start,
            on_update,
            on_quit,
            world_object: js_object,
            res_path
        }
    }
}

impl IGame for JSGame {
    fn start(&mut self,world:&mut World) {
        world.register::<JSEventComponent>();
        self.world_object.set_opaque::<World>(world as *mut World);
        let camera_transform = Transform::default();
        let (w,h) = {
            let view_port = world.fetch::<ViewPortSize>();
            (view_port.width() as f32,view_port.height() as f32)
        };
        let entity = world.create_entity().with(camera_transform).with(Camera::standard_2d(w, h)).build();
        world.insert(ActiveCamera {entity : Some(entity) });
        world.fetch::<S2DLoader>().env().set_fs_root(self.res_path.as_str());
        if let Some(ref fn_val) = self.on_start {
            unsafe {
               let ret_val = q::JS_Call(self.ctx,fn_val.inner().0,RawJsValue::val_null(),1,&mut self.world_object.value());
               AutoDropJSValue::drop_js_value(ret_val,self.ctx);
            }
        }
    }

    fn update(&mut self,_world:&mut World) {
        if let Some(ref fn_val) = self.on_update {
            unsafe {
                let ret_val = q::JS_Call(self.ctx,fn_val.inner().0,RawJsValue::val_null(),1,&mut self.world_object.value());
                AutoDropJSValue::drop_js_value(ret_val,self.ctx);
            }
        }
    }

    fn quit(&mut self,_world:&mut World) {
        if let Some(ref fn_val) = self.on_quit {
            unsafe {
                let ret_val = q::JS_Call(self.ctx,fn_val.inner().0,RawJsValue::val_null(),1,&mut self.world_object.value());
                AutoDropJSValue::drop_js_value(ret_val,self.ctx);
            }
        }
        AutoDropJSValue::drop_js_value(self.world_object.value(), self.ctx);
    }
}
#[derive(Default)]
pub struct JSEventComponent {
    pub ev_nodes:HashMap<u32,q::JSValue>,
    ctx:Option<QJSContext>
}

impl Drop for JSEventComponent {
    fn drop(&mut self) {
        for (_,js_val) in self.ev_nodes.iter() {
            AutoDropJSValue::drop_js_value(*js_val, self.ctx.as_ref().unwrap().0);
        }        
    }
}

impl JSEventComponent {
    pub fn insert_node(&mut self,typ_id:u32,value:q::JSValue) {
        self.ev_nodes.insert(typ_id,value);
    }

    pub fn set_ctx(&mut self,ctx:*mut q::JSContext) {
        self.ctx = Some(QJSContext(ctx));
    }
    
    pub fn ctx(&self) -> Option<&QJSContext> {
        self.ctx.as_ref()
    }
}

unsafe impl Send for JSEventComponent {}
unsafe impl Sync for JSEventComponent {}

impl Component for JSEventComponent {
    type Storage = DenseVecStorage<JSEventComponent>;
}

pub struct QJSContext(pub *mut q::JSContext);
unsafe impl Send for QJSContext {}
unsafe impl Sync for QJSContext {}


pub struct JSEventCallback {
    pub val:q::JSValue,
    pub ctx:Option<QJSContext>
}

impl UpdateCallBack for JSEventCallback {
    fn run(&self) {
        unsafe {
            let ctx = self.ctx.as_ref().unwrap();
            let fire_func_name = CString::new("onFire").unwrap();
            let fire_func = q::JS_GetPropertyStr(ctx.0, self.val, fire_func_name.as_ptr());
            q::JS_Call(ctx.0,fire_func,self.val,0,std::ptr::null_mut());
            AutoDropJSValue::drop_js_value(fire_func,ctx.0);
        }
    }
}

impl Drop for JSEventCallback {
    fn drop(&mut self) {
        AutoDropJSValue::drop_js_value(self.val, self.ctx.as_ref().unwrap().0);
    }
}

unsafe impl Send for JSEventCallback {}
unsafe impl Sync for JSEventCallback {}

impl GameEventCallBack for JSEventCallback {
    fn run(&self,ev:&GameEvent) {
        unsafe {
            let js_val = {
                match ev {
                    GameEvent::KeyBoard(code,is_press) => {
                        JSValue::Array(vec![JSValue::Int(*code as i32),JSValue::Bool(*is_press)])
                    },
                    _ => JSValue::Null
                } 
            };
            let ctx = self.ctx.as_ref().unwrap();
            let fire_func_name = CString::new("onFire").unwrap();
            let fire_func = q::JS_GetPropertyStr(ctx.0, self.val, fire_func_name.as_ptr());
            let val_ptr = &mut js_val.to_c_value(ctx.0);
            q::JS_Call(ctx.0,fire_func,self.val,1,val_ptr);
            AutoDropJSValue::drop_js_value(fire_func,ctx.0);
            AutoDropJSValue::drop_js_value(*val_ptr,ctx.0);
        }
    }
}