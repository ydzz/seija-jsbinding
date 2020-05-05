import {g2d,app} from "seija";
import * as core from "./core.js";
export function testNumberBehavior(eClick) {
    var bNumer = g2d.newBehavior(0);
    g2d.attachBehavior(eClick,bNumer);
    bNumer.myFunc = function(val,eVal) {
        return val + 1;
    };
    bNumer.callBack = function(val) {
        console.error(val);
    };
    g2d.setBehaviorFoldFunc(bNumer,bNumer.myFunc);
    g2d.setBehaviorCallback(bNumer,bNumer.callBack);
}


export function testStringBehavior(eClick) {
    var bString = core.foldBehavior("Default",eClick,function(val,eVal) {   
        return val + ":+:";
    });

    core.effectBehavior(bString,function(val) {
        console.error(val);
    });
}

export function testObjectBehavior(eClick) {
    var bObject = core.foldBehavior({},eClick,function(val,eVal) {
        var rNumber = Math.floor((Math.random()*100)+1);
        var newObject = { };
        newObject[rNumber] = eVal;
        return newObject;
    });
    core.effectBehavior(bObject,function(val) {
        console.error(val);
    });
}

export function testArrayBehavior(eClick) {
    var bArr = core.foldBehavior([],eClick,function(val,eVal) {
        var rNumber = Math.floor((Math.random()*100)+1);
        val.push(rNumber);
        return val;
    });
    core.effectBehavior(bArr,function(val) {
        console.error(val);
    });
}

export function testFunctionBehavior(eClick) {
    var globalFucker = "GGGGGGGGGGG";

    var bObject = core.foldBehavior(testFunc0(globalFucker),eClick,function(val,eVal) {
        
        return val;
    });

    core.effectBehavior(bObject,function(f) {
        f();
    });
}


function testFunc0(global) {
    return function() {
        console.error(global);
    }
}

function testFunc1() {
    return function() {
        console.error("testFunc1");
    }
}