

function V1(o, a) {
    if (o == null)
        throw new Error("Illegal argument " + o);
    var s = e.wordsToBytes(i(o, a));
    return a && a.asBytes ? s : a && a.asString ? r.bytesToString(s) : e.bytesToHex(s)
}

e = {
    region_id: 1003,
    web_location: "333.40138"
}
a = Object.assign({}, e, {
    wts: o
})
o = Math.round(Date.now() / 1e3)
l = []

for (let d = 0; d < s.length; d++) {
    const p = s[d];
    let v = a[p];
    v && typeof v == "string" && (v = v.replace(u, "")),
    v != null && l.push("".concat(encodeURIComponent(p), "=").concat(encodeURIComponent(v)))
}

function w_rid(){
    const c = l.join("&");
    return {
        w_rid: V1(c + i),
        wts: o.toString()
    }
}