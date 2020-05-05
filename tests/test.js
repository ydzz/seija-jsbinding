import {g2d,app} from "seija";
import * as behavior from "./behavior.js";
import * as event from "./event.js";
import * as core from "./core.js";

var s2d = g2d.newSimple2d({  window: {width:1024,height:768} });
  
var myapp = app.newApp(s2d,{OnStart,ResPath: "../seija/examples/first/res/"});


function OnStart(world) {
    var loader = g2d.fetchLoader(world);
    var sheet = g2d.loadSync(loader,world,2,"111/material.json");
    var root = core.addEventRoot(world);
    var elSprite = core.mkSprite(world,sheet,"button",{pos:[0,0,0],size:[100,100] },root);
    var eClick = g2d.getEvent(world,elSprite,2,false);

    //testBehavior(eClick);
     //removeAllChildren
   core.chainEvent(eClick,function(e) {
     var idList = g2d.getChildrens(world,root);
     console.error(idList);
     g2d.removeAllChildren(world,root);
     return e;
   });
}

function testBehavior(eClick) {
    behavior.testNumberBehavior(eClick);
    behavior.testStringBehavior(eClick);
    behavior.testObjectBehavior(eClick);
    behavior.testFunctionBehavior(eClick);
    behavior.testArrayBehavior(eClick);
}

function testEvent(eClick) {
    event.testNormalEvent(eClick);
    event.testSetNextEvent(eClick);
}


app.runApp(myapp);