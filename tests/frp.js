import {g2d} from "seija";

export function defaultEvent() {
    var retObject = {
        nextEvents:[],
        behavoirs:[],
        func:null,
        onFire: function(val) {
            var newVal = val;
            if(this.func != null) {
                newVal = this.func(val);
            }
            for(var i = 0;i < this.nextEvents.length;i++) {
                var curEvent = this.nextEvents[i];
                curEvent.onFire(newVal);
            }

            for(var i = 0;i < this.behavoirs.length;i++) {
                var curBehavior = this.behavoirs[i];
                curBehavior.onValue(newVal);
            }


        }
    };
    retObject.onFire.bind(retObject);
    return retObject;
}

export function fetchEvent(world,entity,eventType,isCapture) {
    var newEvent = defaultEvent();
    g2d.attachNodeEvent(world,entity,eventType,isCapture,newEvent);
    return newEvent;
}

export function fetchTimeEvent(world,entity,updateType,updateNumber) {
    var newEvent = defaultEvent();
    g2d.attachTimeEvent(world,entity,updateType,updateNumber,newEvent);
    return newEvent;
}

export function fetchGlobalEvent(world,entity,evType) {
    var newEvent = defaultEvent();
    g2d.attachGlobalEvent(world,entity,evType,newEvent);
    return newEvent;
}

export function chainEvent(ev,fn) {
    var newEvent = defaultEvent();
    newEvent.func = fn;
    ev.nextEvents.push(newEvent);
    return newEvent;
}


export function newBehavior(val) {
    var retObject = {
        value:val,
        foldFunc:null,
        callBack:null,
        attachInfo:null,
        attchCallback:null,
        onValue: function(eVal) {
            if(this.foldFunc != null) {
                this.value = this.foldFunc(this.value,eVal);
            } else {
                this.value = eVal;
            }
            if(this.callBack != null) {
                this.callBack(this.value);
            }
            if(this.attchCallback != null) {
                this.attchCallback(this.value);
            }
        }
    };
    retObject.onValue.bind(retObject);
    return retObject;
}

export function holdBehavior(val,event) {
    var behavoir = newBehavior(val);
    event.behavoirs.push(behavoir);
    return behavoir;
}

export function foldBehavior(val,event,func) {
    var behavoir = newBehavior(val);
    event.behavoirs.push(behavoir);
    behavoir.foldFunc = func;
    return behavoir;
}

export function callbackBehavior(behavoir,f) {
    behavoir.callBack = f;
}

export function setTransformBehavior(world,entity,attrType,behavoir) {
    behavoir.attchCallback = function(val) {
        g2d.setTransform(world,entity,attrType,val);
    }.bind(behavoir);
}