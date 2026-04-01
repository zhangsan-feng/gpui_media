

Q4 = {
    utf8: {
        stringToBytes: function stringToBytes(e) {
            return Q4.bin.stringToBytes(unescape(encodeURIComponent(e)));
        },
        bytesToString: function bytesToString(e) {
            return decodeURIComponent(escape(Q4.bin.bytesToString(e)));
        }
    },
    bin: {
        stringToBytes: function stringToBytes(e) {
            for (var t = [], r = 0; r < e.length; r++)
                t.push(e.charCodeAt(r) & 255);
            return t;
        },
        bytesToString: function bytesToString(e) {
            for (var t = [], r = 0; r < e.length; r++)
                t.push(String.fromCharCode(e[r]));
            return t.join("");
        }
    }
}
BR = Q4

t = {
    rotl: function rotl(r, n) {
        return r << n | r >>> 32 - n;
    },
    rotr: function rotr(r, n) {
        return r << 32 - n | r >>> n;
    },
    endian: function endian(r) {
        if (r.constructor == Number)
            return this.rotl(r, 8) & 16711935 | this.rotl(r, 24) & 4278255360;
        for (var n = 0; n < r.length; n++)
            r[n] = this.endian(r[n]);
        return r;
    },
    randomBytes: function randomBytes(r) {
        for (var n = []; r > 0; r--)
            n.push(Math.floor(Math.random() * 256));
        return n;
    },
    bytesToWords: function bytesToWords(r) {
        for (var n = [], i = 0, a = 0; i < r.length; i++,
            a += 8)
            n[a >>> 5] |= r[i] << 24 - a % 32;
        return n;
    },
    wordsToBytes: function wordsToBytes(r) {
        for (var n = [], i = 0; i < r.length * 32; i += 8)
            n.push(r[i >>> 5] >>> 24 - i % 32 & 255);
        return n;
    },
    bytesToHex: function bytesToHex(r) {
        for (var n = [], i = 0; i < r.length; i++)
            n.push((r[i] >>> 4).toString(16)),
                n.push((r[i] & 15).toString(16));
        return n.join("");
    },
    hexToBytes: function hexToBytes(r) {
        for (var n = [], i = 0; i < r.length; i += 2)
            n.push(parseInt(r.substr(i, 2), 16));
        return n;
    },
    bytesToBase64: function bytesToBase64(r) {
        for (var n = [], i = 0; i < r.length; i += 3)
            for (var a = r[i] << 16 | r[i + 1] << 8 | r[i + 2], o = 0; o < 4; o++)
                i * 8 + o * 6 <= r.length * 8 ? n.push(e.charAt(a >>> 6 * (3 - o) & 63)) : n.push("=");
        return n.join("");
    },
    base64ToBytes: function base64ToBytes(r) {
        r = r.replace(/[^A-Z0-9+\/]/ig, "");
        for (var n = [], i = 0, a = 0; i < r.length; a = ++i % 4)
            a != 0 && n.push((e.indexOf(r.charAt(i - 1)) & Math.pow(2, -2 * a + 8) - 1) << a * 2 | e.indexOf(r.charAt(i)) >>> 6 - a * 2);
        return n;
    }
};


x$e = t
_i7 = function i(a, o) {

    t = BR.utf8
    e = x$e
    a.constructor == String ? o && o.encoding === "binary" ? a = n.stringToBytes(a) : a = t.stringToBytes(a) : r(a) ? a = Array.prototype.slice.call(a, 0) : !Array.isArray(a) && a.constructor !== Uint8Array && (a = a.toString());
    for (var s = e.bytesToWords(a), l = a.length * 8, c = 1732584193, d = -271733879, u = -1732584194, f = 271733878, p = 0; p < s.length; p++)
        s[p] = (s[p] << 8 | s[p] >>> 24) & 16711935 | (s[p] << 24 | s[p] >>> 8) & 4278255360;
    s[l >>> 5] |= 128 << l % 32,
        s[(l + 64 >>> 9 << 4) + 14] = l;
    for (var h = _i7._ff, g = _i7._gg, v = _i7._hh, y = _i7._ii, p = 0; p < s.length; p += 16) {
        var b = c
            , m = d
            , w = u
            , x = f;
        c = h(c, d, u, f, s[p + 0], 7, -680876936),
            f = h(f, c, d, u, s[p + 1], 12, -389564586),
            u = h(u, f, c, d, s[p + 2], 17, 606105819),
            d = h(d, u, f, c, s[p + 3], 22, -1044525330),
            c = h(c, d, u, f, s[p + 4], 7, -176418897),
            f = h(f, c, d, u, s[p + 5], 12, 1200080426),
            u = h(u, f, c, d, s[p + 6], 17, -1473231341),
            d = h(d, u, f, c, s[p + 7], 22, -45705983),
            c = h(c, d, u, f, s[p + 8], 7, 1770035416),
            f = h(f, c, d, u, s[p + 9], 12, -1958414417),
            u = h(u, f, c, d, s[p + 10], 17, -42063),
            d = h(d, u, f, c, s[p + 11], 22, -1990404162),
            c = h(c, d, u, f, s[p + 12], 7, 1804603682),
            f = h(f, c, d, u, s[p + 13], 12, -40341101),
            u = h(u, f, c, d, s[p + 14], 17, -1502002290),
            d = h(d, u, f, c, s[p + 15], 22, 1236535329),
            c = g(c, d, u, f, s[p + 1], 5, -165796510),
            f = g(f, c, d, u, s[p + 6], 9, -1069501632),
            u = g(u, f, c, d, s[p + 11], 14, 643717713),
            d = g(d, u, f, c, s[p + 0], 20, -373897302),
            c = g(c, d, u, f, s[p + 5], 5, -701558691),
            f = g(f, c, d, u, s[p + 10], 9, 38016083),
            u = g(u, f, c, d, s[p + 15], 14, -660478335),
            d = g(d, u, f, c, s[p + 4], 20, -405537848),
            c = g(c, d, u, f, s[p + 9], 5, 568446438),
            f = g(f, c, d, u, s[p + 14], 9, -1019803690),
            u = g(u, f, c, d, s[p + 3], 14, -187363961),
            d = g(d, u, f, c, s[p + 8], 20, 1163531501),
            c = g(c, d, u, f, s[p + 13], 5, -1444681467),
            f = g(f, c, d, u, s[p + 2], 9, -51403784),
            u = g(u, f, c, d, s[p + 7], 14, 1735328473),
            d = g(d, u, f, c, s[p + 12], 20, -1926607734),
            c = v(c, d, u, f, s[p + 5], 4, -378558),
            f = v(f, c, d, u, s[p + 8], 11, -2022574463),
            u = v(u, f, c, d, s[p + 11], 16, 1839030562),
            d = v(d, u, f, c, s[p + 14], 23, -35309556),
            c = v(c, d, u, f, s[p + 1], 4, -1530992060),
            f = v(f, c, d, u, s[p + 4], 11, 1272893353),
            u = v(u, f, c, d, s[p + 7], 16, -155497632),
            d = v(d, u, f, c, s[p + 10], 23, -1094730640),
            c = v(c, d, u, f, s[p + 13], 4, 681279174),
            f = v(f, c, d, u, s[p + 0], 11, -358537222),
            u = v(u, f, c, d, s[p + 3], 16, -722521979),
            d = v(d, u, f, c, s[p + 6], 23, 76029189),
            c = v(c, d, u, f, s[p + 9], 4, -640364487),
            f = v(f, c, d, u, s[p + 12], 11, -421815835),
            u = v(u, f, c, d, s[p + 15], 16, 530742520),
            d = v(d, u, f, c, s[p + 2], 23, -995338651),
            c = y(c, d, u, f, s[p + 0], 6, -198630844),
            f = y(f, c, d, u, s[p + 7], 10, 1126891415),
            u = y(u, f, c, d, s[p + 14], 15, -1416354905),
            d = y(d, u, f, c, s[p + 5], 21, -57434055),
            c = y(c, d, u, f, s[p + 12], 6, 1700485571),
            f = y(f, c, d, u, s[p + 3], 10, -1894986606),
            u = y(u, f, c, d, s[p + 10], 15, -1051523),
            d = y(d, u, f, c, s[p + 1], 21, -2054922799),
            c = y(c, d, u, f, s[p + 8], 6, 1873313359),
            f = y(f, c, d, u, s[p + 15], 10, -30611744),
            u = y(u, f, c, d, s[p + 6], 15, -1560198380),
            d = y(d, u, f, c, s[p + 13], 21, 1309151649),
            c = y(c, d, u, f, s[p + 4], 6, -145523070),
            f = y(f, c, d, u, s[p + 11], 10, -1120210379),
            u = y(u, f, c, d, s[p + 2], 15, 718787259),
            d = y(d, u, f, c, s[p + 9], 21, -343485551),
            c = c + b >>> 0,
            d = d + m >>> 0,
            u = u + w >>> 0,
            f = f + x >>> 0;
    }
    return e.endian([c, d, u, f]);
}

_i7._ff = function(a, o, s, l, c, d, u) {
    var f = a + (o & s | ~o & l) + (c >>> 0) + u;
    return (f << d | f >>> 32 - d) + o;
}

_i7._gg = function(a, o, s, l, c, d, u) {
        var f = a + (o & l | s & ~l) + (c >>> 0) + u;
        return (f << d | f >>> 32 - d) + o;
    }

_i7._hh = function(a, o, s, l, c, d, u) {
        var f = a + (o ^ s ^ l) + (c >>> 0) + u;
        return (f << d | f >>> 32 - d) + o;
    }

_i7._ii = function(a, o, s, l, c, d, u) {
        var f = a + (s ^ (o | ~l)) + (c >>> 0) + u;
        return (f << d | f >>> 32 - d) + o;
    }

_i7._blocksize = 16
_i7._digestsize = 16

function wordsToBytes(r) {
    for (var n = [], i = 0; i < r.length * 32; i += 8)
        n.push(r[i >>> 5] >>> 24 - i % 32 & 255);
    return n;
}

function O$e(e) {
    try {
        return localStorage.getItem(e);
    } catch (t) {
        return null;
    }
}

function L$e(e) {
    var o;
    if (e.useAssignKey)
        return {
            imgKey: e.wbiImgKey,
            subKey: e.wbiSubKey
        };
    var t = ((o = O$e("wbi_img_urls")) == null ? void 0 : o.split("-")) || []
        , r = t[0]
        , n = t[1]
        , i = r ? VR(r) : e.wbiImgKey
        , a = n ? VR(n) : e.wbiSubKey;
    return {
        imgKey: i,
        subKey: a
    };
}


function $$e(a, o) {
    if (a == null)
        throw new Error("Illegal argument " + a);
    var s = wordsToBytes(_i7(a, o));
    return o && o.asBytes ? s : o && o.asString ? n.bytesToString(s) : e.bytesToHex(s);
}

function k$e(e) {
    var t = [46, 47, 18, 2, 53, 8, 23, 32, 15, 50, 10, 31, 58, 3, 45, 35, 27, 43, 5, 49, 33, 9, 42, 19, 29, 28, 14, 39, 12, 38, 41, 13, 37, 48, 7, 16, 24, 55, 40, 61, 26, 17, 0, 1, 60, 51, 30, 4, 22, 25, 54, 21, 56, 59, 6, 63, 57, 62, 11, 36, 20, 34, 44, 52]
        , r = [];
    return t.forEach(function(n) {
        e.charAt(n) && r.push(e.charAt(n));
    }),
        r.join("").slice(0, 32);
}

function T$e(e, t) {
    t || (t = {});
    var _L$e = L$e(t)
        , r = _L$e.imgKey
        , n = _L$e.subKey;
    if (r && n) {
        var i = k$e(r + n)
            , a = Math.round(Date.now() / 1e3)
            , o = Object.assign({}, e, {
            wts: a
        })
            , s = Object.keys(o).sort()
            , l = []
            , c = /[!'()*]/g;
        for (var f = 0; f < s.length; f++) {
            var p = s[f];
            var h = o[p];
            h && typeof h == "string" && (h = h.replace(c, "")),
            h != null && l.push("".concat(encodeURIComponent(p), "=").concat(encodeURIComponent(h)));
        }
        var d = l.join("&");
        return {
            w_rid: $$e(d + i),
            // w_rid: $$e('web_location=333.788&wts=1774923745' + 'ea1db124af3c7062474693fa704f4ff8'),
            wts: a.toString()
        };
    }
    return null;
}
e = {"web_location":"333.788"}
t = {
    "wbiImgKey": "c458435a75b1419ca98ab6d88b4c60d4",
    "wbiSubKey": "446140f6859f439e9dd83f7ef858d1cd"
}


function gen_w_rid(){
    return T$e(e, t)
}


// console.log(gen_w_rid())





// l = window.__biliUserFp__
// f = l.queryUserLog
// f.call(l, {})
// p = qf()(d, 4)