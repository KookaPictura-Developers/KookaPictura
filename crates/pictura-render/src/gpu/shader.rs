pub(super) const SHADER: &str = r#"
struct Params {
    mode: u32,
    opacity: f32,
    fill: f32,
    count: u32,
    adj_kind: u32,
    p0: i32,
    p1: i32,
    p2: i32,
    flags: u32,
    src_x0: u32,
    src_y0: u32,
    src_w: u32,
    src_h: u32,
    region_x0: u32,
    region_y0: u32,
    region_w: u32,
    stride: u32,
};

@group(0) @binding(0) var<storage, read_write> canvas: array<u32>;
@group(0) @binding(1) var<storage, read> src: array<u32>;
@group(0) @binding(2) var<storage, read> mask: array<u32>;
@group(0) @binding(3) var<uniform> params: Params;

// Both the packed canvas and a group's inner canvas are one u32 per pixel with
// bytes laid out R,G,B,A (little-endian word). Planar pixel-layer sources and
// the mask plane are packed four bytes per u32 and read through the byte
// helpers, so a sample's byte offset needs no per-plane alignment.
fn unpack_word(w: u32) -> vec4<f32> {
    return vec4<f32>(
        f32(w & 0xFFu),
        f32((w >> 8u) & 0xFFu),
        f32((w >> 16u) & 0xFFu),
        f32((w >> 24u) & 0xFFu),
    ) / 255.0;
}

fn src_byte(idx: u32) -> u32 {
    return (src[idx / 4u] >> ((idx % 4u) * 8u)) & 0xFFu;
}

fn mask_byte(idx: u32) -> u32 {
    return (mask[idx / 4u] >> ((idx % 4u) * 8u)) & 0xFFu;
}

fn color_burn(cb: f32, cs: f32) -> f32 {
    if (cb >= 1.0) { return 1.0; }
    if (cs <= 0.0) { return 0.0; }
    return 1.0 - min((1.0 - cb) / cs, 1.0);
}

fn color_dodge(cb: f32, cs: f32) -> f32 {
    if (cb <= 0.0) { return 0.0; }
    if (cs >= 1.0) { return 1.0; }
    return min(cb / (1.0 - cs), 1.0);
}

fn hard_light(cb: f32, cs: f32) -> f32 {
    if (cs <= 0.5) { return 2.0 * cb * cs; }
    return 1.0 - 2.0 * (1.0 - cb) * (1.0 - cs);
}

fn soft_light_d(x: f32) -> f32 {
    if (x <= 0.25) { return ((16.0 * x - 12.0) * x + 4.0) * x; }
    return sqrt(x);
}

fn soft_light(cb: f32, cs: f32) -> f32 {
    if (cs <= 0.5) { return cb - (1.0 - 2.0 * cs) * cb * (1.0 - cb); }
    return cb + (2.0 * cs - 1.0) * (soft_light_d(cb) - cb);
}

fn vivid_light(cb: f32, cs: f32) -> f32 {
    if (cs <= 0.5) { return color_burn(cb, 2.0 * cs); }
    return color_dodge(cb, 2.0 * cs - 1.0);
}

fn pin_light(cb: f32, cs: f32) -> f32 {
    if (cs <= 0.5) { return min(cb, 2.0 * cs); }
    return max(cb, 2.0 * cs - 1.0);
}

fn sep(mode: u32, cb: f32, cs: f32) -> f32 {
    switch mode {
        case 2u: { return min(cb, cs); }
        case 3u: { return cb * cs; }
        case 4u: { return color_burn(cb, cs); }
        case 5u: { return max(cb + cs - 1.0, 0.0); }
        case 7u: { return max(cb, cs); }
        case 8u: { return 1.0 - (1.0 - cb) * (1.0 - cs); }
        case 9u: { return color_dodge(cb, cs); }
        case 10u: { return min(cb + cs, 1.0); }
        case 12u: { return hard_light(cs, cb); }
        case 13u: { return soft_light(cb, cs); }
        case 14u: { return hard_light(cb, cs); }
        case 15u: { return vivid_light(cb, cs); }
        case 16u: { return clamp(cb + 2.0 * cs - 1.0, 0.0, 1.0); }
        case 17u: { return pin_light(cb, cs); }
        case 18u: {
            if (cb + cs >= 1.0) { return 1.0; }
            return 0.0;
        }
        case 19u: { return abs(cb - cs); }
        case 20u: { return cb + cs - 2.0 * cb * cs; }
        case 21u: { return max(cb - cs, 0.0); }
        case 22u: {
            if (cs == 0.0) { return 1.0; }
            return min(cb / cs, 1.0);
        }
        default: { return cs; }
    }
}

fn lum(c: vec3<f32>) -> f32 {
    return 0.3 * c.x + 0.59 * c.y + 0.11 * c.z;
}

fn sat(c: vec3<f32>) -> f32 {
    return max(c.x, max(c.y, c.z)) - min(c.x, min(c.y, c.z));
}

fn clip_color(c: vec3<f32>) -> vec3<f32> {
    let l = lum(c);
    let n = min(c.x, min(c.y, c.z));
    let x = max(c.x, max(c.y, c.z));
    var out = c;
    if (n < 0.0) {
        let d = l - n;
        if (d != 0.0) { out = vec3<f32>(l) + (out - vec3<f32>(l)) * (l / d); }
    }
    if (x > 1.0) {
        let d = x - l;
        if (d != 0.0) { out = vec3<f32>(l) + (out - vec3<f32>(l)) * ((1.0 - l) / d); }
    }
    return out;
}

fn set_lum(c: vec3<f32>, l: f32) -> vec3<f32> {
    let d = l - lum(c);
    return clip_color(c + vec3<f32>(d));
}

fn set_sat(c: vec3<f32>, s: f32) -> vec3<f32> {
    let mx = max(c.x, max(c.y, c.z));
    let mn = min(c.x, min(c.y, c.z));
    if (mx <= mn) { return vec3<f32>(0.0); }
    return (c - vec3<f32>(mn)) * s / (mx - mn);
}

fn blend(mode: u32, cb: vec3<f32>, cs: vec3<f32>) -> vec3<f32> {
    if (mode == 6u) {
        if (cb.x + cb.y + cb.z <= cs.x + cs.y + cs.z) { return cb; }
        return cs;
    }
    if (mode == 11u) {
        if (cb.x + cb.y + cb.z >= cs.x + cs.y + cs.z) { return cb; }
        return cs;
    }
    if (mode == 23u) { return set_lum(set_sat(cs, sat(cb)), lum(cb)); }
    if (mode == 24u) { return set_lum(set_sat(cb, sat(cs)), lum(cb)); }
    if (mode == 25u) { return set_lum(cs, lum(cb)); }
    if (mode == 26u) { return set_lum(cb, lum(cs)); }
    return vec3<f32>(sep(mode, cb.x, cs.x), sep(mode, cb.y, cs.y), sep(mode, cb.z, cs.z));
}

// --- adjustment helpers -----------------------------------------------------

fn rem_euclid(x: f32, m: f32) -> f32 {
    return x - m * floor(x / m);
}

fn quant(v: f32) -> u32 {
    return u32(round(clamp(v, 0.0, 1.0) * 255.0));
}

fn to_f(v: u32) -> f32 {
    return f32(v) / 255.0;
}

fn pack_word(c: vec3<f32>, a: f32) -> u32 {
    return quant(c.x) | (quant(c.y) << 8u) | (quant(c.z) << 16u) | (quant(a) << 24u);
}

// Map a region-local invocation index to its document pixel. The packed canvas
// and every source plane are region-local; only the layer-rect test below needs
// document coordinates.
fn doc_x(i: u32) -> u32 {
    return params.region_x0 + i % params.region_w;
}

fn doc_y(i: u32) -> u32 {
    return params.region_y0 + i / params.region_w;
}

// Resolve one canvas pixel's source sample. A packed group source is a
// region-local canvas word; a pixel layer is planar planes over its layer-rect
// intersection with the region, with the grayscale colour replicated from plane
// 0 and alpha defaulted by the host.
fn sample_src(i: u32) -> vec4<f32> {
    if ((params.flags & 1u) != 0u) {
        return unpack_word(src[i]);
    }
    let x = doc_x(i);
    let y = doc_y(i);
    if (x < params.src_x0 || x >= params.src_x0 + params.src_w
        || y < params.src_y0 || y >= params.src_y0 + params.src_h) {
        return vec4<f32>(0.0);
    }
    let li = (y - params.src_y0) * params.src_w + (x - params.src_x0);
    let n = params.src_w * params.src_h;
    var rgb = vec3<f32>(
        f32(src_byte(li)), f32(src_byte(li)), f32(src_byte(li)),
    ) / 255.0;
    if ((params.flags & 2u) == 0u) {
        rgb = vec3<f32>(
            f32(src_byte(li)),
            f32(src_byte(n + li)),
            f32(src_byte(2u * n + li)),
        ) / 255.0;
    }
    let a_plane = select(3u, 1u, (params.flags & 2u) != 0u);
    return vec4<f32>(rgb, f32(src_byte(a_plane * n + li)) / 255.0);
}

fn adj_posterize(v: u32, levels: i32) -> u32 {
    if (levels >= 255) { return v; }
    let d = f32(levels - 1);
    let q = round(f32(v) * d / 255.0);
    return u32(round(q * 255.0 / d));
}

fn adj_threshold(v: vec3<u32>, level: i32) -> u32 {
    let y = 0.299 * f32(v.x) + 0.587 * f32(v.y) + 0.114 * f32(v.z);
    if (y > f32(level)) { return 255u; }
    return 0u;
}

fn adj_bc(v: u32, brightness: i32, contrast: i32) -> u32 {
    let c = f32(contrast) / 100.0;
    let b = f32(brightness) / 150.0;
    let gamma = pow(2.0, b);
    let y = pow(f32(v) / 255.0, 1.0 / gamma);
    var res = 0.0;
    if (y <= 0.5) {
        let t = 2.0 * y;
        res = 0.5 * t + 0.5 * c * (t * t * t - t * t);
    } else {
        let t = 2.0 * y - 1.0;
        res = 0.5 * (1.0 + t) + 0.5 * c * (t * t * t - 2.0 * t * t + t);
    }
    return u32(round(clamp(res, 0.0, 1.0) * 255.0));
}

fn hue2rgb(p: f32, q: f32, t_in: f32) -> f32 {
    let t = rem_euclid(t_in, 1.0);
    if (t < 1.0 / 6.0) { return p + (q - p) * 6.0 * t; }
    if (t < 0.5) { return q; }
    if (t < 2.0 / 3.0) { return p + (q - p) * (2.0 / 3.0 - t) * 6.0; }
    return p;
}

fn adj_hs(rgb: vec3<f32>, hue: i32, saturation: i32, lightness: i32) -> vec3<f32> {
    let ds = f32(saturation) / 100.0;
    let dl = f32(lightness) / 100.0;
    let mx = max(rgb.x, max(rgb.y, rgb.z));
    let mn = min(rgb.x, min(rgb.y, rgb.z));
    let l0 = (mx + mn) / 2.0;
    var h = 0.0;
    var s = 0.0;
    if (abs(mx - mn) >= 1e-12) {
        let d = mx - mn;
        if (l0 > 0.5) { s = d / (2.0 - mx - mn); } else { s = d / (mx + mn); }
        if (mx == rgb.x) { h = (rgb.y - rgb.z) / d; }
        else if (mx == rgb.y) { h = (rgb.z - rgb.x) / d + 2.0; }
        else { h = (rgb.x - rgb.y) / d + 4.0; }
        h = h * 60.0;
    }
    h = rem_euclid(h + f32(hue), 360.0);
    s = clamp(s * (1.0 + ds), 0.0, 1.0);
    var l = l0;
    if (dl >= 0.0) { l = l + dl * (1.0 - l); } else { l = l + dl * l; }
    l = clamp(l, 0.0, 1.0);
    if (s <= 0.0) { return vec3<f32>(l); }
    var q = l * (1.0 + s);
    if (l >= 0.5) { q = l + s - l * s; }
    let p = 2.0 * l - q;
    let hk = h / 360.0;
    return vec3<f32>(
        hue2rgb(p, q, hk + 1.0 / 3.0),
        hue2rgb(p, q, hk),
        hue2rgb(p, q, hk - 1.0 / 3.0),
    );
}

fn adjust(kind: u32, rgb: vec3<f32>, p0: i32, p1: i32, p2: i32) -> vec3<f32> {
    let q = vec3<u32>(quant(rgb.x), quant(rgb.y), quant(rgb.z));
    if (kind == 1u) {
        return vec3<f32>(to_f(255u - q.x), to_f(255u - q.y), to_f(255u - q.z));
    }
    if (kind == 2u) {
        return vec3<f32>(
            to_f(adj_posterize(q.x, p0)),
            to_f(adj_posterize(q.y, p0)),
            to_f(adj_posterize(q.z, p0)),
        );
    }
    if (kind == 3u) {
        let v = adj_threshold(q, p0);
        return vec3<f32>(to_f(v));
    }
    if (kind == 4u) {
        return vec3<f32>(
            to_f(adj_bc(q.x, p0, p1)),
            to_f(adj_bc(q.y, p0, p1)),
            to_f(adj_bc(q.z, p0, p1)),
        );
    }
    if (kind == 5u) {
        return adj_hs(vec3<f32>(to_f(q.x), to_f(q.y), to_f(q.z)), p0, p1, p2);
    }
    return rgb;
}

@compute @workgroup_size(64)
fn cs_main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x + gid.y * params.stride;
    if (i >= params.count) { return; }
    let mask_a = f32(mask_byte(i)) / 255.0;

    if (params.adj_kind != 0u) {
        let cb = unpack_word(canvas[i]);
        let ab = cb.a;
        if (ab <= 0.0) { return; }
        let as_ = ab * params.opacity * params.fill * mask_a;
        if (as_ <= 0.0) { return; }
        let cs = adjust(params.adj_kind, cb.rgb, params.p0, params.p1, params.p2);
        let b = blend(params.mode, cb.rgb, cs);
        let ao = as_ + ab * (1.0 - as_);
        if (ao <= 0.0) {
            canvas[i] = 0u;
            return;
        }
        let co = ((1.0 - ab) * as_ * cs + as_ * ab * b + (1.0 - as_) * ab * cb.rgb) / ao;
        canvas[i] = pack_word(co, ao);
        return;
    }

    let s = sample_src(i);
    if (s.a <= 0.0) { return; }
    let as_ = s.a * params.opacity * params.fill * mask_a;
    if (as_ <= 0.0) { return; }

    let cb = unpack_word(canvas[i]);
    let ab = cb.a;
    let b = blend(params.mode, cb.rgb, s.rgb);
    let ao = as_ + ab * (1.0 - as_);
    if (ao <= 0.0) {
        canvas[i] = 0u;
        return;
    }
    let co = ((1.0 - ab) * as_ * s.rgb + as_ * ab * b + (1.0 - as_) * ab * cb.rgb) / ao;
    canvas[i] = pack_word(co, ao);
}
"#;

/// Fused planar readback: one invocation de-interleaves four consecutive packed
/// RGBA8 canvas pixels into one word of each of the four channel planes, so the
/// host copies plane bytes instead of gathering a channel per pixel. Its own
/// module because `SHADER`'s bindings are module-scoped and cannot be
/// redeclared. Byte-identical to the host `to_pixel_buffer` gather.
pub(super) const PLANAR_SHADER: &str = r#"
struct P { n: u32, pw: u32, stride: u32 };
@group(0) @binding(0) var<storage, read> packed_canvas: array<u32>;
@group(0) @binding(1) var<storage, read_write> planar: array<u32>;
@group(0) @binding(2) var<uniform> p: P;

@compute @workgroup_size(64)
fn cs_planar(@builtin(global_invocation_id) gid: vec3<u32>) {
    let j = gid.x + gid.y * p.stride;
    if (j >= p.pw) { return; }
    var w0 = 0u; var w1 = 0u; var w2 = 0u; var w3 = 0u;
    for (var k = 0u; k < 4u; k = k + 1u) {
        let i = j * 4u + k;
        var w = 0u;
        if (i < p.n) { w = packed_canvas[i]; }
        let sh = k * 8u;
        w0 = w0 | ((w & 0xFFu) << sh);
        w1 = w1 | (((w >> 8u) & 0xFFu) << sh);
        w2 = w2 | (((w >> 16u) & 0xFFu) << sh);
        w3 = w3 | (((w >> 24u) & 0xFFu) << sh);
    }
    planar[j] = w0;
    planar[p.pw + j] = w1;
    planar[2u * p.pw + j] = w2;
    planar[3u * p.pw + j] = w3;
}
"#;
