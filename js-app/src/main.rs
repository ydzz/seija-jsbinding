use qjs_rs::{JSRuntime,JSContext,init_internal};
use seija_jsbinding::{binding_all,boot_start};

fn main() {
    let mut runtime = JSRuntime::new().unwrap();
    let mut ctx = JSContext::new(&runtime).unwrap();
    init_internal(&mut ctx,&mut runtime);
    binding_all(&mut ctx);
    boot_start(&mut ctx,"./main.js").unwrap();
}