import {g2d,app} from "seija";

export function addEventRoot(world) {
    var e = g2d.newEntity(world);
    g2d.addCABEventRoot(world,e);
    g2d.addRect2d(world,e,1024,768,0.5,0.5);
    g2d.addTransform(world,e);
    return e;
}

export function mkSprite(world,sheet,sprName,prop,p) {
    var e = g2d.newEntity(world);
    if(prop.size) {
        g2d.addRect2d(world,e,prop.size[0],prop.size[1],0.5,0.5);
    } else {
        g2d.addRect2d(world,e,32,32,0.5,0.5);
    }
    g2d.addTransform(world,e,prop.pos || [0,0,0]);
    g2d.addTransparent(world,e);
    g2d.addSpriteRender(world,e,sheet,sprName,[2,0],[1,1,1,1]);
    g2d.setParent(world,e,p);
    return e;
}


export function chainEvent(event,f) {
    var newEvent = g2d.chainEvent(event,f);
    newEvent.f = f;
    if(event.childrens == undefined) {
        event.childrens = [];
    }
    event.childrens.push(newEvent);
    return newEvent;
}


export function foldBehavior(val,eClick,foldFunc) {
    var bRet = g2d.newBehavior(val);
    g2d.attachBehavior(eClick,bRet);
    bRet.myFunc = foldFunc;
    g2d.setBehaviorFoldFunc(bRet,bRet.myFunc);
    return bRet;
}

export function effectBehavior(b,f) {
    b.callBack = f;
    g2d.setBehaviorCallback(b,b.callBack);
}

export function effectEvent(e,f) {
    chainEvent(e,f);
}

export function setNextEvent(event,nextEvent) {
    g2d.setNextEvent(event,nextEvent);
    if(event.childrens == undefined) {
        event.childrens = [];
    }
    event.childrens.push(nextEvent);
}

export function tagBehavior(b,ev) {
    var te = g2d.tagBehavior(b,ev);
    if(ev.childrens == undefined) {
      ev.childrens = [];
    }
    ev.childrens.push(te);
    return te;
}

export function logEvent(e) {
    effectEvent(e,(val) => console.error(val));
}

export function logBehavior(b) {
    effectBehavior(b,(val) => console.error(val));
}