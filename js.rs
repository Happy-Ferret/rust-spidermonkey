/* Rust bindings to the SpiderMonkey JavaScript engine. */

use std;
import comm::chan;
import ctypes::{ size_t, void, c_uint };
import ptr::null;

export new_runtime, new_context, set_options, set_version, new_class;
export new_compartment_and_global_object, init_standard_classes, options;
export null_principals, compile_script, execute_script, value_to_source;
export get_string_bytes, get_string, ext;

/* Structures. */
type JSClass = {
    name: *u8,
    flags: u32,

    /* Mandatory non-null function pointer members. */
    addProperty: JSPropertyOp,
    delProperty: JSPropertyOp,
    getProperty: JSPropertyOp,
    setProperty: JSStrictPropertyOp,
    enumerate: JSEnumerateOp,
    resolve: JSResolveOp,
    convert: JSConvertOp,
    finalize: JSFinalizeOp,

    /* Optionally non-null members. */
    reserved0: JSClassInternal,
    checkAccess: JSCheckAccessOp,
    call: JSNative,
    construct: JSNative,
    xdrObject: JSXDRObjectOp,
    hasInstance: JSHasInstanceOp,
    trace: JSTraceOp,

    reserved1: JSClassInternal,
    reserved: (*void, *void, *void, *void, *void, *void, *void, *void,  /* 8 */
               *void, *void, *void, *void, *void, *void, *void, *void,  /* 16 */
               *void, *void, *void, *void, *void, *void, *void, *void,  /* 24 */
               *void, *void, *void, *void, *void, *void, *void, *void,  /* 32 */
               *void, *void, *void, *void, *void, *void, *void, *void)  /* 40 */
};

type error_report = {
	message: str,
	filename: str,
	lineno: uint,
	flags: uint
};

type log_message = {
	message: str,
	level: uint,
};

/* Opaque types. */
type jsval = u64;
tag jsid { jsid_priv(uint); }
tag object { object_priv(*JSObject); }
tag principals { principals_priv(*JSPrincipals); }
tag script { script_priv(*JSScript); }
tag string { string_priv(*JSString); }

tag JSClassInternal { JSClassInternal(@JSClassInternal); }
tag JSCompartment   { JSCompartment(@JSCompartment);     }
tag JSContext       { JSContext(@JSContext);             }
tag JSObject        { JSObject(@JSObject);               }
tag JSPrincipals    { JSPrincipals(@JSPrincipals);       }
tag JSRuntime       { JSRuntime(@JSRuntime);             }
tag JSScript        { JSScript(@JSScript);               }
tag JSString        { JSString(@JSString);               }
tag JSCrossCompartmentCall {
    JSCrossCompartmentCall(@JSCrossCompartmentCall);
}

/* Types that shouldn't be opaque, but currently are due to limitations in
 * Rust. */
type JSPropertyOp = u64;
type JSStrictPropertyOp = u64;
type JSEnumerateOp = u64;
type JSResolveOp = u64;
type JSConvertOp = u64;
type JSFinalizeOp = u64;

tag JSCheckAccessOp     { JSCheckAccessOp(@JSCheckAccessOp);       }
tag JSEqualityOp        { JSEqualityOp(@JSEqualityOp);             }
tag JSHasInstanceOp     { JSHasInstanceOp(@JSHasInstanceOp);       }
tag JSNative            { JSNative(@JSNative);                     }
tag JSNewEnumerateOp    { JSNewEnumerateOp(@JSNewEnumerateOp);     }
tag JSNewResolveOp      { JSNewResolveOp(@JSNewResolveOp);         }
tag JSStringFinalizeOp  { JSStringFinalizeOp(@JSStringFinalizeOp); }
tag JSTraceOp           { JSTraceOp(@JSTraceOp);                   }
tag JSTraceNamePrinter  { JSTraceNamePrinter(@JSTraceNamePrinter); }
tag JSTypeOfOp          { JSTypeOfOp(@JSTypeOfOp);                 }
tag JSXDRObjectOp       { JSXDRObjectOp(@JSXDRObjectOp);           }

/* Non-opaque types. */
type JSProtoKey = uint;
type JSVersion = uint;
type jsrefcount = uint;

mod options {
    const strict : u32                  = 0x00001u32;   // JS_BIT(0)
    const werror : u32                  = 0x00002u32;   // JS_BIT(1)
    const varobjfix : u32               = 0x00004u32;   // JS_BIT(2)
    const private_is_nsISupports : u32  = 0x00008u32;   // JS_BIT(3)
    const compile_n_go : u32            = 0x00010u32;   // JS_BIT(4)
    const atline : u32                  = 0x00020u32;   // JS_BIT(5)
    const xml : u32                     = 0x00040u32;   // JS_BIT(6)
    const dont_report_uncaught : u32    = 0x00100u32;   // JS_BIT(8)
    const relimit : u32                 = 0x00200u32;   // JS_BIT(9)
    const no_script_rval : u32          = 0x01000u32;   // JS_BIT(12)
    const unrooted_global : u32         = 0x02000u32;   // JS_BIT(13)
    const methodjit : u32               = 0x04000u32;   // JS_BIT(14)
    const methodjit_always : u32        = 0x10000u32;   // JS_BIT(16)
    const pccount : u32                 = 0x20000u32;   // JS_BIT(17)
    const type_inference : u32          = 0x40000u32;   // JS_BIT(18)
    const soften : u32                  = 0x80000u32;   // JS_BIT(19)
}

#[link_name="mozjs"]
native mod js {
    fn JS_Init(maxbytes : u32) -> *JSRuntime;
    fn JS_Finish(rt : *JSRuntime);
    fn JS_ShutDown();
    fn JS_GetRuntimePrivate(rt : *JSRuntime) -> *void;
    fn JS_SetRuntimePrivate(rt : *JSRuntime, data : *void);

    fn JS_BeginRequest(cx : *JSContext);
    fn JS_EndRequest(cx : *JSContext);
    fn JS_YieldRequest(cx : *JSContext);
    fn JS_SuspendRequest(cx : *JSContext) -> jsrefcount;
    fn JS_ResumeRequest(cx : *JSContext, saveDepth : jsrefcount);
    fn JS_IsInRequest(cx : *JSContext) -> bool;

    fn JS_Lock(rt : *JSRuntime);
    fn JS_Unlock(rt : *JSRuntime);
    // fn JS_SetContextCallback(rt : *JSRuntime,
    //                                 cxCallback : JSContextCallback);
    fn JS_DestroyContext(cx : *JSContext);
    fn JS_DestroyContextNoGC(cx : *JSContext);
    fn JS_DestroyContextMaybeGC(cx : *JSContext);
    fn JS_GetContextPrivate(cx : *JSContext) -> *void;
    fn JS_SetContextPrivate(cx : *JSContext, data : *void);
    fn JS_GetRuntime(cx : *JSContext) -> *JSRuntime;

    fn JS_ContextIterator(rt : *JSRuntime, iterp : **JSContext)
        -> *JSContext;
    fn JS_ContextIteratorUnlocked(rt : *JSRuntime, iterp : **JSContext)
        -> *JSContext;

    fn JS_GetVersion(cx : *JSContext) -> JSVersion;
    fn JS_SetVersion(cx : *JSContext, version : JSVersion)
        -> JSVersion;
    fn JS_VersionToString(version : JSVersion) -> *u8;
    fn JS_StringToVersion(string : *u8) -> JSVersion;

    fn JS_GetOptions(cx : *JSContext) -> u32;
    fn JS_SetOptions(cx : *JSContext, options : u32) -> u32;
    fn JS_ToggleOptions(cx : *JSContext, options : u32) -> u32;

    fn JS_GetImplementationVersion() -> *u8;

    // fn JS_SetCompartmentCallback(rt : *JSRuntime,
    //                                     callback : JSCompartmentCallback)
    //      -> JSCompartmentCallback;
    // fn JS_SetWrapObjectCallbacks(rt : *JSRuntime,
    //                                     callback : JSWrapObjectCallback,
    //                                     precallback : JSPreWrapCallback)
    //      -> JSWrapObjectCallback;

    fn JS_EnterCrossCompartmentCall(cx : *JSContext,
                                           target : *JSObject)
        -> *JSCrossCompartmentCall;
    fn JS_LeaveCrossCompartmentCall(call : *JSCrossCompartmentCall);
    fn JS_SetCompartmentPrivate(cx : *JSContext,
                                       compartment : *JSCompartment,
                                       data : *void) -> *void;
    fn JS_GetCompartmentPrivate(cx : *JSContext,
                                       compartment : *JSCompartment) -> *void;

    fn JS_WrapObject(cx : *JSContext, objp: **JSObject) -> bool;
    fn JS_WrapValue(cx : *JSContext, val : *jsval) -> bool;
    fn JS_TransplantObject(cx : *JSContext, origobj : *JSObject,
                                  target : *JSObject) -> *JSObject;

    fn JS_GetGlobalObject(cx : *JSContext) -> *JSObject;
    fn JS_SetGlobalObject(cx : *JSContext, object : *JSObject);

    fn JS_InitStandardClasses(cx : *JSContext, object : *JSObject)
        -> bool;
    fn JS_ResolveStandardClass(cx : *JSContext, object : *JSObject,
                                      id : jsid, resolved : *bool) -> bool;
    fn JS_EnumerateStandardClasses(cx : *JSContext, object : *JSObject)
        -> bool;
    fn JS_EnumerateResolvedStandardClasses(cx : *JSContext,
                                                  object : *JSObject,
                                                  ida : *jsid) -> *jsid;

    fn JS_GetClassObject(cx : *JSContext, object : *JSObject,
                                key : JSProtoKey, objp : **JSObject) -> bool;
    fn JS_GetGlobalForObject(cx : *JSContext, object : *JSObject)
        -> *JSObject;
    fn JS_GetGlobalForScopeChain(cx : *JSContext) -> *JSObject;

    fn JS_InitReflect(cx : *JSContext, global : *JSObject)
        -> *JSObject;

    fn JS_AddValueRoot(cx : *JSContext, vp : *jsval) -> bool;
    fn JS_AddStringRoot(cx : *JSContext, rp : **JSString) -> bool;
    fn JS_AddObjectRoot(cx : *JSContext, rp : **JSObject) -> bool;
    fn JS_AddGCThingRoot(cx : *JSContext, rp : **void) -> bool;

    /* TODO: Plenty more to add here. */

    fn JS_ValueToSource(cx : *JSContext, v : jsval) -> *JSString;

    /* TODO: Plenty more to add here. */

    fn JS_NewCompartmentAndGlobalObject(cx : *JSContext,
                                               clasp : *JSClass,
                                               principals : *JSPrincipals)
        -> *JSObject;

    /* TODO: Plenty more to add here. */

    fn JS_BufferIsCompilableUnit(cx : *JSContext,
                                        bytes_are_utf8 : bool,
                                        object : *JSObject, bytes : *u8,
                                        length : size_t);
    fn JS_CompileScript(cx : *JSContext, object : *JSObject,
                               bytes : *u8, length : size_t,
                               filename : *u8, lineno : c_uint) -> *JSScript;

    /* TODO: Plenty more to add here. */

    fn JS_ExecuteScript(cx : *JSContext, object : *JSObject,
                               script : *JSScript, rval : *jsval) -> bool;
    fn JS_ExecuteScriptVersion(cx : *JSContext, object : *JSObject,
                                      script : *JSScript, rval : *jsval,
                                      version : JSVersion) -> bool;

    /* TODO: Plenty more to add here. */
    
    fn JS_GetStringCharsZAndLength(cx : *JSContext, jsstr : *JSString,
                                   length : *size_t) -> *u8;

    /* TODO: Plenty more to add here. */

    fn JS_EncodeCharacters(cx : *JSContext, src : *u8, srclen : size_t,
                           dst : *u8, dstlenp : *size_t) -> bool;

    /* TODO: Plenty more to add here. */

}

#[link_args="-L."]
#[link_name="spidermonkeyrustext"]
native mod jsrust {
    /* Bindings to work around Rust's missing features. */
    fn JSRust_GetPropertyStub() -> JSPropertyOp;
    fn JSRust_GetStrictPropertyStub() -> JSStrictPropertyOp;
    fn JSRust_GetEnumerateStub() -> JSEnumerateOp;
    fn JSRust_GetResolveStub() -> JSResolveOp;
    fn JSRust_GetConvertStub() -> JSConvertOp;
    fn JSRust_GetFinalizeStub() -> JSFinalizeOp;

    fn JSRust_GetNullJSClassInternal() -> JSClassInternal;
    fn JSRust_GetNullJSCheckAccessOp() -> JSCheckAccessOp;
    fn JSRust_GetNullJSNative() -> JSNative;
    fn JSRust_GetNullJSXDRObjectOp() -> JSXDRObjectOp;
    fn JSRust_GetNullJSHasInstanceOp() -> JSHasInstanceOp;
    fn JSRust_GetNullJSTraceOp() -> JSTraceOp;

	/* Additional features. */
    fn JSRust_NewContext(rt : *JSRuntime, stackChunkSize : size_t)
        -> *JSContext;
	fn JSRust_SetErrorChannel(cx : *JSContext, chan : chan<error_report>)
		-> bool;
	fn JSRust_SetLogChannel(cx : *JSContext, object : *JSObject, chan : chan<log_message>)
		-> bool;
	fn JSRust_InitRustLibrary(cx : *JSContext, object : *JSObject) -> bool;
}

resource runtime(rt : *JSRuntime) {
    js::JS_Finish(rt);
}

resource context(cx : *JSContext) {
    js::JS_DestroyContext(cx);
}

resource request(cx : *JSContext) {
    js::JS_EndRequest(cx);
}

/* Runtimes */

fn new_runtime(maxbytes : u32) -> runtime {
    ret runtime(js::JS_Init(maxbytes));
}

fn shut_down() {
    js::JS_ShutDown();
}

/* Contexts */

fn new_context(rt : runtime, stack_chunk_size : size_t) -> context {
    ret context(jsrust::JSRust_NewContext(*rt, stack_chunk_size));
}

/* Options */

fn get_options(cx : context) -> u32 {
    ret js::JS_GetOptions(*cx);
}

fn set_options(cx : context, options : u32) {
    let _ = js::JS_SetOptions(*cx, options);
}

fn set_version(cx : context, version : JSVersion) {
    let _ = js::JS_SetVersion(*cx, version);
}

/* Objects */

fn new_compartment_and_global_object(cx : context, class : @class,
                                     principals : principals) -> object {
    let jsclass = ptr::addr_of(class.jsclass);
    let jsobj = js::JS_NewCompartmentAndGlobalObject(*cx, jsclass,
                                                     *principals);
    if jsobj == null() { fail; }
    ret object_priv(jsobj);
}

/* Principals */

fn null_principals() -> principals {
    ret principals_priv(null());
}

/* Classes */

type class_spec = {
    name: str,
    flags: u32
    /* TODO: More to add here. */
};

type class = {
    name: @str,
    jsclass: JSClass
};

fn new_class(spec : class_spec) -> @class unsafe {
    // Root the name separately, and make the JSClass name point into it.
    let name = @spec.name;
    ret @{
        name: name,
        jsclass: {
            name: str::as_buf(*name, { |b| b }),
            flags: spec.flags,

            addProperty: jsrust::JSRust_GetPropertyStub(),
            delProperty: jsrust::JSRust_GetPropertyStub(),
            getProperty: jsrust::JSRust_GetPropertyStub(),
            setProperty: jsrust::JSRust_GetStrictPropertyStub(),
            enumerate: jsrust::JSRust_GetEnumerateStub(),
            resolve: jsrust::JSRust_GetResolveStub(),
            convert: jsrust::JSRust_GetConvertStub(),
            finalize: jsrust::JSRust_GetFinalizeStub(),

            reserved0: ptr::null(),
            checkAccess: ptr::null(),
            call: ptr::null(),
            construct: ptr::null(),
            xdrObject: ptr::null(),
            hasInstance: ptr::null(),
            trace: ptr::null(),

            reserved1: ptr::null(),
            reserved: (ptr::null(),ptr::null(),ptr::null(),ptr::null(),ptr::null(),ptr::null(),ptr::null(),ptr::null(), ptr::null(),ptr::null(),ptr::null(),ptr::null(),ptr::null(),ptr::null(),ptr::null(),ptr::null(),    /* 16 */
                       ptr::null(),ptr::null(),ptr::null(),ptr::null(),ptr::null(),ptr::null(),ptr::null(),ptr::null(), ptr::null(),ptr::null(),ptr::null(),ptr::null(),ptr::null(),ptr::null(),ptr::null(),ptr::null(),    /* 32 */
                       ptr::null(),ptr::null(),ptr::null(),ptr::null(),ptr::null(),ptr::null(),ptr::null(),ptr::null())
        }
    };
}

/* Standard classes */

fn init_standard_classes(cx : context, object : object) {
    if !js::JS_InitStandardClasses(*cx, *object) { fail; }
}

/* Script compilation */

fn compile_script(cx : context, object : object, src : [u8], filename : str,
                  lineno : uint) -> script unsafe {
    let jsscript = str::as_buf(filename, { |buf|
        js::JS_CompileScript(*cx, *object, vec::to_ptr(src),
                             vec::len(src) as size_t, buf, lineno as c_uint)
    });
    if jsscript == ptr::null() {
        fail;   // TODO: this is antisocial
    }
    ret script_priv(jsscript);
}

/* Script execution */

fn execute_script(cx : context, object : object, script : script)
        -> option<jsval> unsafe {
    let rv : jsval = unsafe::reinterpret_cast(0);
    if !js::JS_ExecuteScript(*cx, *object, *script, ptr::addr_of(rv)) {
        ret none;
    }
    ret some(rv);
}

/* Value conversion */

fn value_to_source(cx : context, v : jsval) -> string {
    ret string_priv(js::JS_ValueToSource(*cx, v));
}

/* String conversion */

fn get_string_bytes(cx : context, jsstr : string) -> [u8] unsafe {
    // FIXME: leaks, probably
    let size = 0 as size_t;
    let bytes = js::JS_GetStringCharsZAndLength(*cx, *jsstr,
                                                ptr::addr_of(size));
    ret vec::unsafe::from_buf(bytes, ((size + (1 as size_t)) * (2 as size_t)));
}

fn get_string(cx : context, jsstr : string) -> str unsafe {
    let bytes = get_string_bytes(cx, jsstr);

    // Make a sizing call.
    let len = 0 as size_t;
    if !js::JS_EncodeCharacters(*cx, vec::to_ptr(bytes),
                                (vec::len(bytes) / 2u) as size_t, ptr::null(),
                                ptr::addr_of(len)) {
        fail;
    }

    let buf = vec::init_elt(0u8, (len as uint) + 1u);
    if !js::JS_EncodeCharacters(*cx, vec::to_ptr(bytes),
                                (vec::len(bytes) / 2u) as size_t,
                                vec::to_ptr(buf), ptr::addr_of(len)) {
        fail;
    }

    ret str::unsafe_from_bytes(buf);
}

/** Rust extensions to the JavaScript language bindings. */
mod ext {
	fn set_error_channel(cx : context, chan : chan<error_report>) {
		if !jsrust::JSRust_SetErrorChannel(*cx, chan) { fail; }
	}

	fn set_log_channel(cx : context, object : object, chan : chan<log_message>) {
		if !jsrust::JSRust_SetLogChannel(*cx, *object, chan) { fail; }
	}

	fn init_rust_library(cx : context, object : object) {
		if !jsrust::JSRust_InitRustLibrary(*cx, *object) { fail; }
	}
}

