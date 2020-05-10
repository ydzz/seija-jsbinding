import {g2d,app} from "seija";
import * as core from "./core.js";
import * as frp from "./frp.js";

var s2d = g2d.newSimple2d({  window: {width:1024,height:768} });
  
var myapp = app.newApp(s2d,{OnStart,ResPath: "../seija/examples/first/res/"});


function OnStart(world) {
    var loader = g2d.fetchLoader(world);
    var sheet = g2d.loadSync(loader,world,2,"111/material.json",null);
    var root = core.addEventRoot(world);
    var elSprite = core.mkSprite(world,sheet,"button",{pos:[0,0,0],size:[50,50] },root);
    
    var eKey = frp.fetchGlobalEvent(world,elSprite,6);
    frp.chainEvent(eKey,function(val) {
       
        return val;
    });
}


app.runApp(myapp);