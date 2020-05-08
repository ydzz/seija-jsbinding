import {g2d,app} from "seija";
import * as core from "./core.js";
import * as frp from "./frp.js";

var s2d = g2d.newSimple2d({  window: {width:1024,height:768} });
  
var myapp = app.newApp(s2d,{OnStart,ResPath: "../seija/examples/first/res/"});


function OnStart(world) {
    var loader = g2d.fetchLoader(world);
    var sheet = g2d.loadSync(loader,world,2,"111/material.json",null);
    var root = core.addEventRoot(world);
    var elSprite = core.mkSprite(world,sheet,"button",{pos:[0,0,0],size:[100,100] },root);
    
    var eClick = frp.fetchEvent(world,elSprite,2,false);
  
    
    var behavior = frp.foldBehavior([0,0,0],eClick,function(v,ev){
        v[0] += 1;
        return v;
    });
    
    frp.setTransformBehavior(world,elSprite,0,behavior);
}


app.runApp(myapp);