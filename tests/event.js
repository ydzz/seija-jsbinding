import {g2d,app} from "seija";
import * as core from "./core.js";

export function testNormalEvent(eClick) {
   var eString = core.chainEvent(eClick,(e) => "String");
   var eNumber = core.chainEvent(eString,(e) => 6666);
   var eNumber2 = core.chainEvent(eString,(e) => 7777);

   var eNumber0 = core.chainEvent(eNumber,(e) => e + 1);

   core.effectEvent(eNumber0,(e) => {
       console.error(e);
   });

   var bNumber = core.foldBehavior(2,eNumber0,function(val,eVal) {
       return val * 2;
   });
   core.effectBehavior(bNumber,(val) => console.error(val));
}

export function testSetNextEvent(eClick) {
    var newEvent = g2d.newEvent();
    var bNumber = core.foldBehavior(0,newEvent,(val,eVal) => val + eVal);
    core.setNextEvent(eClick,newEvent);
    var newEvent2 = core.chainEvent(newEvent,(e) => e + 1);
    var bObject = core.foldBehavior({},newEvent2,(val,eVal) => {
        var newObject = {};
        var rNumber = Math.floor((Math.random()*100)+1);
        newObject[rNumber] = val;
        return newObject;
    });

    var tagE = core.tagBehavior(bObject,eClick);
    var bTag = core.foldBehavior([],tagE,function(val,eVal) {
        var newVal = ["A"].concat(val);
        return newVal;
    });
    
    //core.effectBehavior(bNumber,(val) => console.error(val));
    //core.effectBehavior(bObject,(val) => console.error(val));
    core.logEvent(tagE);
    core.logBehavior(bTag);
}