use spidermonkey;
import spidermonkey::js;

import ctypes::size_t;
import comm::{ port, chan, recv, send };

use std;

import std::{ io, os, treemap, uvtmp };

enum child_message {
    set_msg(chan<js::jsrust_message>),
    got_msg(js::jsrust_message),
    io_cb(u32, u32, u32, u32, str),
    stdout(str),
    stderr(str),
    spawn(str, str),
    cast(str, str),
    load_url(str),
    load_script(str),
    exitproc,
    done,
}

fn make_children(msg_chan : chan<child_message>, senduv_chan: chan<chan<uvtmp::iomsg>>) {
	let CONN = 0u32,
		SEND = 1u32,
		RECV = 2u32,
		CLOSE = 3u32,
		TIME = 8u32;
	
    task::spawn {||
        let js_port = port::<js::jsrust_message>();
        send(msg_chan, set_msg(chan(js_port)));

        while true {
            let msg = recv(js_port);
            if msg.level == 9u32 {
                send(msg_chan, exitproc);
                break;
            } else {
                send(msg_chan, got_msg(msg));
            }
        }
    };

    task::spawn {||
        let uv_port = port::<uvtmp::iomsg>();
        send(senduv_chan, chan(uv_port));
        while true {
            let msg = recv(uv_port);
            alt msg {
                uvtmp::connected(cd) {
                    send(msg_chan, io_cb(CONN, uvtmp::get_req_id(cd), 0u32, 0u32, "onconnect"));
                }
                uvtmp::wrote(cd) {
                    send(msg_chan, io_cb(SEND, uvtmp::get_req_id(cd), 0u32, 0u32, "onsend"));
                }
                uvtmp::read(cd, buf, len) {
                    if len == -1 {
                        send(msg_chan, io_cb(CLOSE, uvtmp::get_req_id(cd), 0u32, 0u32, "onclose"));
                    } else {
                        unsafe {
                            let vecbuf = vec::unsafe::from_buf(buf, len as uint);
                            let bufstr = str::from_bytes(vecbuf);
                            send(msg_chan, io_cb(RECV, uvtmp::get_req_id(cd), 0u32, 0u32, bufstr));
                            uvtmp::delete_buf(buf);
                        }
                    }
                }
                uvtmp::timer(req_id) {
                    send(msg_chan, io_cb(TIME, req_id, 0u32, 0u32, "ontimer"));
                }
                uvtmp::whatever {
                
                }
                uvtmp::exit {
                    send(msg_chan, done);
                    break;
                }
            }
        }
    };
}


fn make_actor(myid : int, myurl : str, thread : uvtmp::thread, maxbytes : u32, out : chan<child_message>, sendchan : chan<(int, chan<child_message>)>) {
    let rt = js::get_thread_runtime(maxbytes);
    let msg_port = port::<child_message>();
    let msg_chan = chan(msg_port);
    send(sendchan, (myid, msg_chan));
    let senduv_port = port::<chan<uvtmp::iomsg>>();
    make_children(chan(msg_port), chan(senduv_port));
    let uv_chan = recv(senduv_port);

    let cx = js::new_context(rt, maxbytes as size_t);
    js::set_options(cx, js::options::varobjfix | js::options::methodjit);
    js::set_version(cx, 185u);

    let globclass = js::new_class({
		name: "global",
		flags: js::ext::get_global_class_flags() });
    let global = js::new_compartment_and_global_object(
        cx, globclass, js::null_principals());

    js::init_standard_classes(cx, global);
    js::ext::init_rust_library(cx, global);

    let exit = false;
    let setup = 0;
    let childid = 0;

    while !exit {
        let msg = recv(msg_port);
        alt msg {
            set_msg(ch) {
                js::ext::set_msg_channel(
                    cx, global, ch);
                setup += 1;
            }
            load_url(x) {
                //log(core::error, ("LOAD URL", x));
                //js::begin_request(*cx);
                js::set_data_property(cx, global, x);
                let code = "try { _resume(5, _data, 0) } catch (e) { print(e + '\\n' + e.stack) } _data = undefined;";
                let script = js::compile_script(cx, global, str::bytes(code), "io", 0u);
                js::execute_script(cx, global, script);
                //js::end_request(*cx);
            }
            load_script(script) {
                alt std::io::read_whole_file(script) {
                    result::ok(file) {
                        let script = js::compile_script(
                            cx, global,
							str::bytes(#fmt("try { %s } catch (e) { print(e + '\\n' + e.stack); }", str::from_bytes(file))),
							script, 0u);
                        js::execute_script(cx, global, script);
                        let checkwait = js::compile_script(
                        cx, global, str::bytes("if (XMLHttpRequest.requests_outstanding === 0)  jsrust_exit();"), "io", 0u);
                        js::execute_script(cx, global, checkwait);
                    }
                    _ {
                        log(core::error, #fmt("File not found: %s", script));
                        js::ext::rust_exit_now(0);
                    }
                }
            }
            got_msg(m) {                
                // messages from javascript
                alt m.level{
                    0u32 { // CONNECT
                        uvtmp::connect(
                            thread, m.tag, m.message, uv_chan);
                    }
                    1u32 { // SEND
						//log(core::error, ("send", m.tag, m.message));
                        uvtmp::write(
                            thread, m.tag,
                            str::bytes(m.message),
                            uv_chan);
                    }
                    2u32 { // RECV
						//log(core::error, ("recv", m.tag));
                        uvtmp::read_start(thread, m.tag, uv_chan);
                    }
                    3u32 { // CLOSE
                        //log(core::error, "close");
                        uvtmp::close_connection(thread, m.tag);
                    }
                    4u32 { // stdout
                    send(out, stdout(
                        #fmt("[Actor %d] %s",
                        myid, m.message)));
                    }
                    5u32 { // stderr
                        send(out, stderr(
                            #fmt("[ERROR %d] %s",
                            myid, m.message)));
                    }
                    6u32 { // spawn
                        send(out, spawn(
                            #fmt("%d:%d", myid, childid),
                            m.message));
                        childid = childid + 1;
                    }
                    7u32 { // cast
                    }
                    8u32 { // SETTIMEOUT
						//log(core::error, ("time", m.tag));
                        uvtmp::timer_start(thread, m.timeout, m.tag, uv_chan);
                    }
					9u32 { // exit
					}
                    _ {
                        log(core::error, "...");
                    }
                }
            }
            io_cb(level, tag, timeout, _p, buf) {
                //log(core::error, ("io_cb", level, tag, timeout, buf));
                js::begin_request(*cx);
                js::set_data_property(cx, global, buf);
                let code = #fmt("try { _resume(%u, _data, %u); } catch (e) { print(e + '\\n' + e.stack); }; _data = undefined;", level as uint, tag as uint);
                let script = js::compile_script(cx, global, str::bytes(code), "io", 0u);
                js::execute_script(cx, global, script);
                js::end_request(*cx);
            }
            exitproc {
                send(uv_chan, uvtmp::exit);
            }
            done {
                exit = true;
                send(out, done);
            }
            _ { fail "unexpected case" }
        }
        if setup == 1 {
            setup = 2;
            alt std::io::read_whole_file("xmlhttprequest.js") {
                result::ok(file) {
                    let script = js::compile_script(
                        cx, global, file, "xmlhttprequest.js", 0u);
                    js::execute_script(cx, global, script);
                }
                _ { fail }
            }
            alt std::io::read_whole_file("dom.js") {
                result::ok(file) {
                    let script = js::compile_script(
                        cx, global, file, "dom.js", 0u);
                    js::execute_script(cx, global, script);
                }
                _ { fail }
            }
            if str::len_bytes(myurl) > 4u && str::eq(str::slice(myurl, 0u, 4u), "http") {
                send(msg_chan, load_url(myurl));
            } else {
                send(msg_chan, load_script(myurl));
            }
        }
    }
}


fn main(args : [str]) {
    let maxbytes = 32u32 * 1024u32 * 1024u32;
    let thread = uvtmp::create_thread();
    uvtmp::start_thread(thread);

    let stdoutport = port::<child_message>();
    let stdoutchan = chan(stdoutport);

    let sendchanport = port::<(int, chan<child_message>)>();
    let sendchanchan = chan(sendchanport);

    let map = treemap::init();

    let argc = vec::len(args);
    let argv = if argc == 1u {
        ["test.js"]
    } else {
        vec::slice(args, 1u, argc)
    };

    let left = 0;

    for x in argv {
        left += 1;
        task::spawn {||
            make_actor(left, x, thread, maxbytes, stdoutchan, sendchanchan);
        };
    }
    let actorid = left;

    for _x in argv {
        let (theid, thechan) = recv(sendchanport);
        treemap::insert(map, theid, thechan);
    }

    while true {
        alt recv(stdoutport) {
            stdout(x) { log(core::error, x); }
            stderr(x) { log(core::error, x); }
            spawn(id, src) {
                log(core::error, ("spawn", id, src));
                actorid = actorid + 1;
                left = left + 1;
                task::spawn {||
                    make_actor(actorid, src, thread, maxbytes, stdoutchan, sendchanchan);
                };
            }
            cast(id, msg) {}
            exitproc {
                left = left - 1;
                if left == 0 {
                    let n = @mutable 0;
                    fn t(n: @mutable int, &&_k: int, &&v: chan<child_message>) {
                        send(v, exitproc);
                        *n += 1;
                    }
                    treemap::traverse(map, bind t(n, _, _));
                    left = *n;
                }
            }
            done {
                left = left - 1;
                if left == 0 {
                    break;
                }
            }
            _ { fail "unexpected case" }
        }
    }
    // temp hack: join never returns right now
    js::ext::rust_exit_now(0);
    uvtmp::join_thread(thread);
    uvtmp::delete_thread(thread);
}

