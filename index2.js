import {g2d,app} from "seija";

var s2d = g2d.newSimple2d({
  window: {width:1024,height:768},
});

var myapp = app.newApp(s2d,{OnStart,ResPath: "/home/yangdao/Project/Rust/seija/examples/first/res/"});

 

function OnStart(world) {
  var loader = g2d.fetchLoader(world);
  var sheet = g2d.loadSync(loader,world,2,"111/material.json");
  var root = addEventRoot(world);
  
  var sprite = mkSprite(world,sheet,"button",root);
  var ev0 = g2d.getEvent(world,sprite,4,false);
  var ev1 = g2d.getEvent(world,sprite,5,false);
  var ev00 = chainEvent(ev0,(_) => "button-hover");
  var ev11 = chainEvent(ev1,(_) => "button");
  var mev = merageEvent([ev00,ev11]);
  

  var b = g2d.newBehavior("button");
  g2d.attachBehavior(mev,b);
 
  setBehaviorFoldFunc(b,function(old,ev) {
    //console.error(old);
    console.error(ev);
    return ev;
  });
  g2d.setSpriteRenderBehavior(world,sprite,{spriteName:b });
 
  var ccc = g2d.newBehavior("Fucker");
 
}

function mkSprite(world,sheet,sprName,p) {
  var e = g2d.newEntity(world);
  g2d.addRect2d(world,e,100,35,0.5,0.5);
  g2d.addTransform(world,e);
  g2d.addTransparent(world,e);
  g2d.addSpriteRender(world,e,sheet,sprName,[2,0],[1,1,1,1]);
  g2d.setParent(world,e,p);
  return e;
}

function addEventRoot(world) {
  var e = g2d.newEntity(world);
  g2d.addCABEventRoot(world,e);
  g2d.addRect2d(world,e,1024,768,0.5,0.5);
  g2d.addTransform(world,e);
  return e;
}

function setBehaviorFoldFunc(b,f) {
  b.myfunc = f;
  g2d.behaviorSetFoldFunc(b,b.myfunc);
}

function merageEvent(eventArray) {
  var newEvent = g2d.mergeEvent(eventArray);
  for(var i = 0; i < eventArray.length;i++) {
    if (eventArray[i].childrens == undefined) {
      eventArray[i].childrens = [];
    } 
    eventArray[i].childrens.push(newEvent);
  }
  return newEvent;
}

function chainEvent(event,f) {
  var newEvent = g2d.chainEvent(event,f);
  newEvent.f = f;
  if(event.childrens == undefined) {
      event.childrens = [];
  }
  event.childrens.push(newEvent);
  return newEvent;
}

app.runApp(myapp);
