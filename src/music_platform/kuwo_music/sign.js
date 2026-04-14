
globalThis.window = globalThis;
globalThis.document = {};

function f(t, e) {
    if (null == e || e.length <= 0)
        return null;
    for (var n = "", i = 0; i < e.length; i++)
        n += e.charCodeAt(i).toString();
    var o = Math.floor(n.length / 5)
        , r = parseInt(n.charAt(o) + n.charAt(2 * o) + n.charAt(3 * o) + n.charAt(4 * o) + n.charAt(5 * o))
        , c = Math.ceil(e.length / 2)
        , l = Math.pow(2, 31) - 1;
    if (r < 2)
        return null;
    var d = Math.round(1e9 * Math.random()) % 1e8;
    for (n += d; n.length > 10; )
        n = (parseInt(n.substring(0, 10)) + parseInt(n.substring(10, n.length))).toString();
    n = (r * n + c) % l;
    var f = ""
        , h = "";
    for (i = 0; i < t.length; i++)
        h += (f = parseInt(t.charCodeAt(i) ^ Math.floor(n / l * 255))) < 16 ? "0" + f.toString(16) : f.toString(16),
            n = (r * n + c) % l;
    for (d = d.toString(16); d.length < 8; )
        d = "0" + d;
    return h += d
}


function set_e(t) {
    var e = document.cookie
        , n = e.indexOf(t + "=");
    if (-1 != n) {
        n = n + t.length + 1;
        var o = e.indexOf(";", n);
        return -1 == o && (o = e.length),
            unescape(e.substring(n, o))
    }
    return null
}



var JZQ=undefined;
!function(e) {
    function r(data) {
        for (var r, n, f = data[0], l = data[1], d = data[2], i = 0, v = []; i < f.length; i++)
            n = f[i],
            Object.prototype.hasOwnProperty.call(o, n) && o[n] && v.push(o[n][0]),
                o[n] = 0;
        for (r in l)
            Object.prototype.hasOwnProperty.call(l, r) && (e[r] = l[r]);
        for (h && h(data); v.length; )
            v.shift()();
        return c.push.apply(c, d || []),
            t()
    }
    function t() {
        for (var e, i = 0; i < c.length; i++) {
            for (var r = c[i], t = !0, n = 1; n < r.length; n++) {
                var l = r[n];
                0 !== o[l] && (t = !1)
            }
            t && (c.splice(i--, 1),
                e = f(f.s = r[0]))
        }
        return e
    }
    var n = {}
        , o = {
        32: 0
    }
        , c = [];
    function f(r) {
        if (n[r])
            return n[r].exports;
        var t = n[r] = {
            i: r,
            l: !1,
            exports: {}
        };
        // console.log(r)
        return e[r].call(t.exports, t, t.exports, f),
            t.l = !0,
            t.exports
    }
    f.e = function(e) {
        var r = []
            , t = o[e];
        if (0 !== t)
            if (t)
                r.push(t[2]);
            else {
                var n = new Promise((function(r, n) {
                        t = o[e] = [r, n]
                    }
                ));
                r.push(t[2] = n);
                var c, script = document.createElement("script");
                script.charset = "utf-8",
                    script.timeout = 120,
                f.nc && script.setAttribute("nonce", f.nc),
                    script.src = function(e) {
                        return f.p + "" + {
                            0: "484bc49",
                            1: "e0235d5",
                            2: "aa15399",
                            5: "60a313c",
                            6: "179aeb3",
                            7: "9cc517f",
                            8: "2c990c1",
                            9: "1a5d23c",
                            10: "6cd57c7",
                            11: "5dafb21",
                            12: "ec771db",
                            13: "be2cd93",
                            14: "213786f",
                            15: "097e70b",
                            16: "e6d8d6d",
                            17: "cd86c6e",
                            18: "22367d4",
                            19: "8d4b846",
                            20: "614240c",
                            21: "5027f25",
                            22: "efee0d2",
                            23: "7016da9",
                            24: "5b1eacf",
                            25: "043c3bb",
                            26: "df44b65",
                            27: "89ba656",
                            28: "1b0264d",
                            29: "38c98f3",
                            30: "718797f",
                            31: "9c42768",
                            34: "959c1c2"
                        }[e] + ".js"
                    }(e);
                var l = new Error;
                c = function(r) {
                    script.onerror = script.onload = null,
                        clearTimeout(d);
                    var t = o[e];
                    if (0 !== t) {
                        if (t) {
                            var n = r && ("load" === r.type ? "missing" : r.type)
                                , c = r && r.target && r.target.src;
                            l.message = "Loading chunk " + e + " failed.\n(" + n + ": " + c + ")",
                                l.name = "ChunkLoadError",
                                l.type = n,
                                l.request = c,
                                t[1](l)
                        }
                        o[e] = void 0
                    }
                }
                ;
                var d = setTimeout((function() {
                        c({
                            type: "timeout",
                            target: script
                        })
                    }
                ), 12e4);
                script.onerror = script.onload = c,
                    document.head.appendChild(script)
            }
        return Promise.all(r)
    }
        ,
        f.m = e,
        f.c = n,
        f.d = function(e, r, t) {
            f.o(e, r) || Object.defineProperty(e, r, {
                enumerable: !0,
                get: t
            })
        }
        ,
        f.r = function(e) {
            "undefined" != typeof Symbol && Symbol.toStringTag && Object.defineProperty(e, Symbol.toStringTag, {
                value: "Module"
            }),
                Object.defineProperty(e, "__esModule", {
                    value: !0
                })
        }
        ,
        f.t = function(e, r) {
            if (1 & r && (e = f(e)),
            8 & r)
                return e;
            if (4 & r && "object" == typeof e && e && e.__esModule)
                return e;
            var t = Object.create(null);
            if (f.r(t),
                Object.defineProperty(t, "default", {
                    enumerable: !0,
                    value: e
                }),
            2 & r && "string" != typeof e)
                for (var n in e)
                    f.d(t, n, function(r) {
                        return e[r]
                    }
                        .bind(null, n));
            return t
        }
        ,
        f.n = function(e) {
            var r = e && e.__esModule ? function() {
                        return e.default
                    }
                    : function() {
                        return e
                    }
            ;
            return f.d(r, "a", r),
                r
        }
        ,
        f.o = function(object, e) {
            return Object.prototype.hasOwnProperty.call(object, e)
        }
        ,
        f.p = "https://h5s.kuwo.cn/www/kw-www/",
        f.oe = function(e) {
            throw console.error(e),
                e
        }
    ;
    var l = window.webpackJsonp = window.webpackJsonp || []
        , d = l.push.bind(l);
    l.push = r,
        l = l.slice();
    for (var i = 0; i < l.length; i++)
        r(l[i]);
    var h = d;
    t()
    JZQ = f
}({
    "113":function(e, t, r) {
        var n, o, l = r(148), c = r(149), d = 0, h = 0;
        e.exports = function(e, t, r) {
            var i = t && r || 0
                , b = t || []
                , f = (e = e || {}).node || n
                , v = void 0 !== e.clockseq ? e.clockseq : o;
            if (null == f || null == v) {
                var m = l();
                null == f && (f = n = [1 | m[0], m[1], m[2], m[3], m[4], m[5]]),
                null == v && (v = o = 16383 & (m[6] << 8 | m[7]))
            }
            var y = void 0 !== e.msecs ? e.msecs : (new Date).getTime()
                , w = void 0 !== e.nsecs ? e.nsecs : h + 1
                , dt = y - d + (w - h) / 1e4;
            if (dt < 0 && void 0 === e.clockseq && (v = v + 1 & 16383),
            (dt < 0 || y > d) && void 0 === e.nsecs && (w = 0),
            w >= 1e4)
                throw new Error("uuid.v1(): Can't create more than 10M uuids/sec");
            d = y,
                h = w,
                o = v;
            var A = (1e4 * (268435455 & (y += 122192928e5)) + w) % 4294967296;
            b[i++] = A >>> 24 & 255,
                b[i++] = A >>> 16 & 255,
                b[i++] = A >>> 8 & 255,
                b[i++] = 255 & A;
            var x = y / 4294967296 * 1e4 & 268435455;
            b[i++] = x >>> 8 & 255,
                b[i++] = 255 & x,
                b[i++] = x >>> 24 & 15 | 16,
                b[i++] = x >>> 16 & 255,
                b[i++] = v >>> 8 | 128,
                b[i++] = 255 & v;
            for (var T = 0; T < 6; ++T)
                b[i + T] = f[T];
            return t || c(b)
        }
    },
    "148":function(e, t) {
        var r = "undefined" != typeof crypto && crypto.getRandomValues && crypto.getRandomValues.bind(crypto) || "undefined" != typeof msCrypto && "function" == typeof window.msCrypto.getRandomValues && msCrypto.getRandomValues.bind(msCrypto);
        if (r) {
            var n = new Uint8Array(16);
            e.exports = function() {
                return r(n),
                    n
            }
        } else {
            var o = new Array(16);
            e.exports = function() {
                for (var e, i = 0; i < 16; i++)
                    3 & i || (e = 4294967296 * Math.random()),
                        o[i] = e >>> ((3 & i) << 3) & 255;
                return o
            }
        }
    },
    "149":function(e, t) {
        for (var r = [], i = 0; i < 256; ++i)
            r[i] = (i + 256).toString(16).substr(1);
        e.exports = function(e, t) {
            var i = t || 0
                , n = r;
            return [n[e[i++]], n[e[i++]], n[e[i++]], n[e[i++]], "-", n[e[i++]], n[e[i++]], "-", n[e[i++]], n[e[i++]], "-", n[e[i++]], n[e[i++]], "-", n[e[i++]], n[e[i++]], n[e[i++]], n[e[i++]], n[e[i++]], n[e[i++]]].join("")
        }
    },

});

function getSecret(cookie) {
    document.cookie = cookie
    let t= "Hm_Iuvt_cdb524f42f23cer9b268564v7y735ewrq2324"
    let e = set_e(t)
    return f(e, t)
}

function getReqId(){
    return JZQ("113")()
}

// console.log(getReqId())
// console.log(getSecret("Hm_Iuvt_cdb524f42f23cer9b268564v7y735ewrq2324=3BMRNKWwW7bksP24P7TFZDWtpxH2z3NZ"))