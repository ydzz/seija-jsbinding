use seija::core::IGame;
use seija::specs::{World,WorldExt,Builder,Component,DenseVecStorage};
use seija::common::{Transform};
use seija::render::{ActiveCamera,Camera};
use seija::module_bundle::{S2DLoader};
use std::sync::{Arc};
use std::ops::{Deref,DerefMut};

use qjs_rs::{JSValue,q,RawJsValue,AutoDropJSValue,JSClass,JSClassOject};
use seija::frp::{IFRPObject,FRPNode};

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
        world.register::<JSFRPNode>();
        self.world_object.set_opaque::<World>(world as *mut World);
        let camera_transform = Transform::default();
        let entity = world.create_entity().with(camera_transform).with(Camera::standard_2d(1024f32, 768f32)).build();
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
pub struct JSFRPNode {
    node:FRPNode<QJSValue>,
    js_objects:Vec<JSClassOject>,
    ctx:Option<QJSContext>
}

impl JSFRPNode {
    pub fn push_js_object(&mut self,js_object:JSClassOject) {
        RawJsValue(js_object.value()).add_ref_count(1);
        self.js_objects.push(js_object);
    }

    pub fn set_ctx(&mut self,ctx:*mut q::JSContext) {
        self.ctx = Some(QJSContext(ctx));
    }
}

impl Drop for JSFRPNode {
    fn drop(&mut self) {
       //dbg!("Drop JSFRPNode");
       for object in self.js_objects.iter() {
           AutoDropJSValue::drop_js_value(object.value(), self.ctx.as_ref().unwrap().0);
       }
    }
}

impl Deref for JSFRPNode {
    type Target = FRPNode<QJSValue>;
    fn deref(&self) -> &Self::Target {
        &self.node
    }
}

impl DerefMut for JSFRPNode {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.node
    }
}

impl Component for JSFRPNode {
    type Storage = DenseVecStorage<JSFRPNode>;
}

unsafe impl Send for JSFRPNode {}
unsafe impl Sync for JSFRPNode {}


pub struct QJSContext(pub *mut q::JSContext);
unsafe impl Send for QJSContext {}
unsafe impl Sync for QJSContext {}

#[derive(Copy,Clone)]
pub struct QJSWorld(pub *mut World);
unsafe impl Send for QJSWorld {}
unsafe impl Sync for QJSWorld {}


#[derive(Copy,Clone)]
pub struct QJSValue(pub q::JSValue);

impl QJSValue {
    pub fn new(val:q::JSValue) -> Self {
        QJSValue(val)
    }
}

impl Into<q::JSValue> for QJSValue {
    fn into(self) -> q::JSValue {
        self.0
    }
}

impl From<q::JSValue> for QJSValue {
    fn from(val:q::JSValue) -> Self {
        QJSValue(val)
    }
}

impl IFRPObject for QJSValue {
    type Context = *mut q::JSContext;
    fn call(&self,val:QJSValue,ctx: *mut q::JSContext) -> QJSValue {
        unsafe {
            let mut val2 = val.0;
            q::JS_Call(ctx,self.0,RawJsValue::val_null(),1,&mut val2).into()
        } 
    }

    fn fold_call(&self,old_val: Self, val: Self, ctx: Self::Context) -> QJSValue {
        unsafe {
          
            let mut val_arr = [old_val.0,val.0];
           
            q::JS_Call(ctx,self.0,RawJsValue::val_null(),2,val_arr.as_mut_ptr()).into()
        }
    }

    fn drop_object(&self,ctx:Self::Context) {
        AutoDropJSValue::drop_js_value(self.0, ctx)
    }

    fn debug(&self,_:Self::Context) {
        dbg!(self.0.tag);
        dbg!(RawJsValue(self.0).ref_count());
    }

    fn add_ref_count(&self,count:i32) {
        RawJsValue(self.0).add_ref_count(count);
    }
}