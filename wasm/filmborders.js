let wasm_bindgen;
(function() {
    const __exports = {};
    let wasm;

    const cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });

    cachedTextDecoder.decode();

    let cachedUint8Memory0;
    function getUint8Memory0() {
        if (cachedUint8Memory0.byteLength === 0) {
            cachedUint8Memory0 = new Uint8Array(wasm.memory.buffer);
        }
        return cachedUint8Memory0;
    }

    function getStringFromWasm0(ptr, len) {
        return cachedTextDecoder.decode(getUint8Memory0().subarray(ptr, ptr + len));
    }

    const heap = new Array(32).fill(undefined);

    heap.push(undefined, null, true, false);

    let heap_next = heap.length;

    function addHeapObject(obj) {
        if (heap_next === heap.length) heap.push(heap.length + 1);
        const idx = heap_next;
        heap_next = heap[idx];

        heap[idx] = obj;
        return idx;
    }

function getObject(idx) { return heap[idx]; }

function dropObject(idx) {
    if (idx < 36) return;
    heap[idx] = heap_next;
    heap_next = idx;
}

function takeObject(idx) {
    const ret = getObject(idx);
    dropObject(idx);
    return ret;
}

function debugString(val) {
    // primitive types
    const type = typeof val;
    if (type == 'number' || type == 'boolean' || val == null) {
        return  `${val}`;
    }
    if (type == 'string') {
        return `"${val}"`;
    }
    if (type == 'symbol') {
        const description = val.description;
        if (description == null) {
            return 'Symbol';
        } else {
            return `Symbol(${description})`;
        }
    }
    if (type == 'function') {
        const name = val.name;
        if (typeof name == 'string' && name.length > 0) {
            return `Function(${name})`;
        } else {
            return 'Function';
        }
    }
    // objects
    if (Array.isArray(val)) {
        const length = val.length;
        let debug = '[';
        if (length > 0) {
            debug += debugString(val[0]);
        }
        for(let i = 1; i < length; i++) {
            debug += ', ' + debugString(val[i]);
        }
        debug += ']';
        return debug;
    }
    // Test for built-in
    const builtInMatches = /\[object ([^\]]+)\]/.exec(toString.call(val));
    let className;
    if (builtInMatches.length > 1) {
        className = builtInMatches[1];
    } else {
        // Failed to match the standard '[object ClassName]'
        return toString.call(val);
    }
    if (className == 'Object') {
        // we're a user defined class or Object
        // JSON.stringify avoids problems with cycles, and is generally much
        // easier than looping through ownProperties of `val`.
        try {
            return 'Object(' + JSON.stringify(val) + ')';
        } catch (_) {
            return 'Object';
        }
    }
    // errors
    if (val instanceof Error) {
        return `${val.name}: ${val.message}\n${val.stack}`;
    }
    // TODO we could test for more things here, like `Set`s and `Map`s.
    return className;
}

let WASM_VECTOR_LEN = 0;

const cachedTextEncoder = new TextEncoder('utf-8');

const encodeString = (typeof cachedTextEncoder.encodeInto === 'function'
    ? function (arg, view) {
    return cachedTextEncoder.encodeInto(arg, view);
}
    : function (arg, view) {
    const buf = cachedTextEncoder.encode(arg);
    view.set(buf);
    return {
        read: arg.length,
        written: buf.length
    };
});

function passStringToWasm0(arg, malloc, realloc) {

    if (realloc === undefined) {
        const buf = cachedTextEncoder.encode(arg);
        const ptr = malloc(buf.length);
        getUint8Memory0().subarray(ptr, ptr + buf.length).set(buf);
        WASM_VECTOR_LEN = buf.length;
        return ptr;
    }

    let len = arg.length;
    let ptr = malloc(len);

    const mem = getUint8Memory0();

    let offset = 0;

    for (; offset < len; offset++) {
        const code = arg.charCodeAt(offset);
        if (code > 0x7F) break;
        mem[ptr + offset] = code;
    }

    if (offset !== len) {
        if (offset !== 0) {
            arg = arg.slice(offset);
        }
        ptr = realloc(ptr, len, len = offset + arg.length * 3);
        const view = getUint8Memory0().subarray(ptr + offset, ptr + len);
        const ret = encodeString(arg, view);

        offset += ret.written;
    }

    WASM_VECTOR_LEN = offset;
    return ptr;
}

let cachedInt32Memory0;
function getInt32Memory0() {
    if (cachedInt32Memory0.byteLength === 0) {
        cachedInt32Memory0 = new Int32Array(wasm.memory.buffer);
    }
    return cachedInt32Memory0;
}

function isLikeNone(x) {
    return x === undefined || x === null;
}

let stack_pointer = 32;

function addBorrowedObject(obj) {
    if (stack_pointer == 1) throw new Error('out of js stack');
    heap[--stack_pointer] = obj;
    return stack_pointer;
}

function _assertClass(instance, klass) {
    if (!(instance instanceof klass)) {
        throw new Error(`expected instance of ${klass.name}`);
    }
    return instance.ptr;
}

const u32CvtShim = new Uint32Array(2);

const int64CvtShim = new BigInt64Array(u32CvtShim.buffer);

function passArray8ToWasm0(arg, malloc) {
    const ptr = malloc(arg.length * 1);
    getUint8Memory0().set(arg, ptr / 1);
    WASM_VECTOR_LEN = arg.length;
    return ptr;
}

let cachedUint8ClampedMemory0;
function getUint8ClampedMemory0() {
    if (cachedUint8ClampedMemory0.byteLength === 0) {
        cachedUint8ClampedMemory0 = new Uint8ClampedArray(wasm.memory.buffer);
    }
    return cachedUint8ClampedMemory0;
}

function getClampedArrayU8FromWasm0(ptr, len) {
    return getUint8ClampedMemory0().subarray(ptr / 1, ptr / 1 + len);
}

function handleError(f, args) {
    try {
        return f.apply(this, args);
    } catch (e) {
        wasm.__wbindgen_exn_store(addHeapObject(e));
    }
}
/**
*/
__exports.FitMode = Object.freeze({ Image:0,"0":"Image",Border:1,"1":"Border", });
/**
*/
__exports.Builtin = Object.freeze({ Border120_1:0,"0":"Border120_1", });
/**
*/
__exports.Rotation = Object.freeze({ Rotate0:0,"0":"Rotate0",Rotate90:1,"1":"Rotate90",Rotate180:2,"2":"Rotate180",Rotate270:3,"3":"Rotate270", });
/**
*/
class Border {

    static __wrap(ptr) {
        const obj = Object.create(Border.prototype);
        obj.ptr = ptr;

        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.ptr;
        this.ptr = 0;

        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_border_free(ptr);
    }
    /**
    * @param {ImageData | undefined} custom
    * @param {number | undefined} builtin
    */
    constructor(custom, builtin) {
        const ret = wasm.border_new(isLikeNone(custom) ? 0 : addHeapObject(custom), isLikeNone(builtin) ? 1 : builtin);
        return Border.__wrap(ret);
    }
    /**
    * @param {ImageData} data
    * @returns {Border}
    */
    static from_image_data(data) {
        const ret = wasm.border_from_image_data(addHeapObject(data));
        return Border.__wrap(ret);
    }
    /**
    * @param {number} builtin
    * @returns {Border}
    */
    static builtin(builtin) {
        const ret = wasm.border_builtin(builtin);
        return Border.__wrap(ret);
    }
}
__exports.Border = Border;
/**
*/
class BoundedSize {

    static __wrap(ptr) {
        const obj = Object.create(BoundedSize.prototype);
        obj.ptr = ptr;

        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.ptr;
        this.ptr = 0;

        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_boundedsize_free(ptr);
    }
    /**
    */
    get width() {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.__wbg_get_boundedsize_width(retptr, this.ptr);
            var r0 = getInt32Memory0()[retptr / 4 + 0];
            var r1 = getInt32Memory0()[retptr / 4 + 1];
            return r0 === 0 ? undefined : r1 >>> 0;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
    */
    set width(arg0) {
        wasm.__wbg_set_boundedsize_width(this.ptr, !isLikeNone(arg0), isLikeNone(arg0) ? 0 : arg0);
    }
    /**
    */
    get height() {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.__wbg_get_boundedsize_height(retptr, this.ptr);
            var r0 = getInt32Memory0()[retptr / 4 + 0];
            var r1 = getInt32Memory0()[retptr / 4 + 1];
            return r0 === 0 ? undefined : r1 >>> 0;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
    */
    set height(arg0) {
        wasm.__wbg_set_boundedsize_height(this.ptr, !isLikeNone(arg0), isLikeNone(arg0) ? 0 : arg0);
    }
    /**
    */
    constructor() {
        const ret = wasm.boundedsize_new();
        return BoundedSize.__wrap(ret);
    }
}
__exports.BoundedSize = BoundedSize;
/**
*/
class Color {

    static __wrap(ptr) {
        const obj = Object.create(Color.prototype);
        obj.ptr = ptr;

        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.ptr;
        this.ptr = 0;

        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_color_free(ptr);
    }
    /**
    * @param {string} hex
    */
    constructor(hex) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            const ptr0 = passStringToWasm0(hex, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            wasm.color_hex(retptr, ptr0, len0);
            var r0 = getInt32Memory0()[retptr / 4 + 0];
            var r1 = getInt32Memory0()[retptr / 4 + 1];
            var r2 = getInt32Memory0()[retptr / 4 + 2];
            if (r2) {
                throw takeObject(r1);
            }
            return Color.__wrap(r0);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
    * @param {number} r
    * @param {number} g
    * @param {number} b
    * @returns {Color}
    */
    static rgb(r, g, b) {
        const ret = wasm.color_rgb(r, g, b);
        return Color.__wrap(ret);
    }
    /**
    * @param {number} r
    * @param {number} g
    * @param {number} b
    * @param {number} a
    * @returns {Color}
    */
    static rgba(r, g, b, a) {
        const ret = wasm.color_rgba(r, g, b, a);
        return Color.__wrap(ret);
    }
    /**
    * @returns {Color}
    */
    static clear() {
        const ret = wasm.color_clear();
        return Color.__wrap(ret);
    }
    /**
    * @returns {Color}
    */
    static black() {
        const ret = wasm.color_black();
        return Color.__wrap(ret);
    }
    /**
    * @returns {Color}
    */
    static white() {
        const ret = wasm.color_white();
        return Color.__wrap(ret);
    }
    /**
    * @returns {Color}
    */
    static gray() {
        const ret = wasm.color_gray();
        return Color.__wrap(ret);
    }
}
__exports.Color = Color;
/**
*/
class Image {

    static __wrap(ptr) {
        const obj = Object.create(Image.prototype);
        obj.ptr = ptr;

        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.ptr;
        this.ptr = 0;

        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_image_free(ptr);
    }
    /**
    * @param {HTMLCanvasElement} canvas
    * @param {CanvasRenderingContext2D} ctx
    * @returns {Image}
    */
    static from_canvas(canvas, ctx) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.image_from_canvas(retptr, addBorrowedObject(canvas), addBorrowedObject(ctx));
            var r0 = getInt32Memory0()[retptr / 4 + 0];
            var r1 = getInt32Memory0()[retptr / 4 + 1];
            var r2 = getInt32Memory0()[retptr / 4 + 2];
            if (r2) {
                throw takeObject(r1);
            }
            return Image.__wrap(r0);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
            heap[stack_pointer++] = undefined;
            heap[stack_pointer++] = undefined;
        }
    }
    /**
    * @param {ImageData} data
    * @returns {Image}
    */
    static from_image_data(data) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.image_from_image_data(retptr, addBorrowedObject(data));
            var r0 = getInt32Memory0()[retptr / 4 + 0];
            var r1 = getInt32Memory0()[retptr / 4 + 1];
            var r2 = getInt32Memory0()[retptr / 4 + 2];
            if (r2) {
                throw takeObject(r1);
            }
            return Image.__wrap(r0);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
            heap[stack_pointer++] = undefined;
        }
    }
}
__exports.Image = Image;
/**
*/
class ImageBorders {

    static __wrap(ptr) {
        const obj = Object.create(ImageBorders.prototype);
        obj.ptr = ptr;

        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.ptr;
        this.ptr = 0;

        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_imageborders_free(ptr);
    }
    /**
    * @param {HTMLCanvasElement} canvas
    * @param {CanvasRenderingContext2D} ctx
    * @returns {ImageBorders}
    */
    static from_canvas(canvas, ctx) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.imageborders_from_canvas(retptr, addBorrowedObject(canvas), addBorrowedObject(ctx));
            var r0 = getInt32Memory0()[retptr / 4 + 0];
            var r1 = getInt32Memory0()[retptr / 4 + 1];
            var r2 = getInt32Memory0()[retptr / 4 + 2];
            if (r2) {
                throw takeObject(r1);
            }
            return ImageBorders.__wrap(r0);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
            heap[stack_pointer++] = undefined;
            heap[stack_pointer++] = undefined;
        }
    }
    /**
    * @param {ImageData} data
    * @returns {ImageBorders}
    */
    static from_image_data(data) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.imageborders_from_image_data(retptr, addBorrowedObject(data));
            var r0 = getInt32Memory0()[retptr / 4 + 0];
            var r1 = getInt32Memory0()[retptr / 4 + 1];
            var r2 = getInt32Memory0()[retptr / 4 + 2];
            if (r2) {
                throw takeObject(r1);
            }
            return ImageBorders.__wrap(r0);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
            heap[stack_pointer++] = undefined;
        }
    }
    /**
    * @param {HTMLCanvasElement} canvas
    * @param {CanvasRenderingContext2D} ctx
    * @returns {ImageData}
    */
    static to_image_data(canvas, ctx) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.imageborders_to_image_data(retptr, addBorrowedObject(canvas), addBorrowedObject(ctx));
            var r0 = getInt32Memory0()[retptr / 4 + 0];
            var r1 = getInt32Memory0()[retptr / 4 + 1];
            var r2 = getInt32Memory0()[retptr / 4 + 2];
            if (r2) {
                throw takeObject(r1);
            }
            return takeObject(r0);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
            heap[stack_pointer++] = undefined;
            heap[stack_pointer++] = undefined;
        }
    }
    /**
    * @param {Border} border
    * @param {Options} options
    * @returns {ImageData}
    */
    render(border, options) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            _assertClass(border, Border);
            var ptr0 = border.ptr;
            border.ptr = 0;
            _assertClass(options, Options);
            wasm.imageborders_render(retptr, this.ptr, ptr0, options.ptr);
            var r0 = getInt32Memory0()[retptr / 4 + 0];
            var r1 = getInt32Memory0()[retptr / 4 + 1];
            var r2 = getInt32Memory0()[retptr / 4 + 2];
            if (r2) {
                throw takeObject(r1);
            }
            return takeObject(r0);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
}
__exports.ImageBorders = ImageBorders;
/**
*/
class Options {

    static __wrap(ptr) {
        const obj = Object.create(Options.prototype);
        obj.ptr = ptr;

        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.ptr;
        this.ptr = 0;

        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_options_free(ptr);
    }
    /**
    */
    get output_size() {
        const ret = wasm.__wbg_get_options_output_size(this.ptr);
        return BoundedSize.__wrap(ret);
    }
    /**
    */
    set output_size(arg0) {
        _assertClass(arg0, BoundedSize);
        var ptr0 = arg0.ptr;
        arg0.ptr = 0;
        wasm.__wbg_set_options_output_size(this.ptr, ptr0);
    }
    /**
    */
    get output_size_bounds() {
        const ret = wasm.__wbg_get_options_output_size_bounds(this.ptr);
        return BoundedSize.__wrap(ret);
    }
    /**
    */
    set output_size_bounds(arg0) {
        _assertClass(arg0, BoundedSize);
        var ptr0 = arg0.ptr;
        arg0.ptr = 0;
        wasm.__wbg_set_options_output_size_bounds(this.ptr, ptr0);
    }
    /**
    */
    get scale_factor() {
        const ret = wasm.__wbg_get_options_scale_factor(this.ptr);
        return ret;
    }
    /**
    */
    set scale_factor(arg0) {
        wasm.__wbg_set_options_scale_factor(this.ptr, arg0);
    }
    /**
    */
    get margin() {
        const ret = wasm.__wbg_get_options_margin(this.ptr);
        return ret;
    }
    /**
    */
    set margin(arg0) {
        wasm.__wbg_set_options_margin(this.ptr, arg0);
    }
    /**
    */
    get mode() {
        const ret = wasm.__wbg_get_options_mode(this.ptr);
        return ret >>> 0;
    }
    /**
    */
    set mode(arg0) {
        wasm.__wbg_set_options_mode(this.ptr, arg0);
    }
    /**
    */
    get crop() {
        const ret = wasm.__wbg_get_options_crop(this.ptr);
        return ret === 0 ? undefined : Sides.__wrap(ret);
    }
    /**
    */
    set crop(arg0) {
        let ptr0 = 0;
        if (!isLikeNone(arg0)) {
            _assertClass(arg0, Sides);
            ptr0 = arg0.ptr;
            arg0.ptr = 0;
        }
        wasm.__wbg_set_options_crop(this.ptr, ptr0);
    }
    /**
    */
    get frame_width() {
        const ret = wasm.__wbg_get_options_frame_width(this.ptr);
        return Sides.__wrap(ret);
    }
    /**
    */
    set frame_width(arg0) {
        _assertClass(arg0, Sides);
        var ptr0 = arg0.ptr;
        arg0.ptr = 0;
        wasm.__wbg_set_options_frame_width(this.ptr, ptr0);
    }
    /**
    */
    get image_rotation() {
        const ret = wasm.__wbg_get_options_image_rotation(this.ptr);
        return ret >>> 0;
    }
    /**
    */
    set image_rotation(arg0) {
        wasm.__wbg_set_options_image_rotation(this.ptr, arg0);
    }
    /**
    */
    get border_rotation() {
        const ret = wasm.__wbg_get_options_border_rotation(this.ptr);
        return ret >>> 0;
    }
    /**
    */
    set border_rotation(arg0) {
        wasm.__wbg_set_options_border_rotation(this.ptr, arg0);
    }
    /**
    */
    get frame_color() {
        const ret = wasm.__wbg_get_options_frame_color(this.ptr);
        return Color.__wrap(ret);
    }
    /**
    */
    set frame_color(arg0) {
        _assertClass(arg0, Color);
        var ptr0 = arg0.ptr;
        arg0.ptr = 0;
        wasm.__wbg_set_options_frame_color(this.ptr, ptr0);
    }
    /**
    */
    get background_color() {
        const ret = wasm.__wbg_get_options_background_color(this.ptr);
        return ret === 0 ? undefined : Color.__wrap(ret);
    }
    /**
    */
    set background_color(arg0) {
        let ptr0 = 0;
        if (!isLikeNone(arg0)) {
            _assertClass(arg0, Color);
            ptr0 = arg0.ptr;
            arg0.ptr = 0;
        }
        wasm.__wbg_set_options_background_color(this.ptr, ptr0);
    }
    /**
    */
    get preview() {
        const ret = wasm.__wbg_get_options_preview(this.ptr);
        return ret !== 0;
    }
    /**
    */
    set preview(arg0) {
        wasm.__wbg_set_options_preview(this.ptr, arg0);
    }
    /**
    */
    constructor() {
        const ret = wasm.options_new();
        return Options.__wrap(ret);
    }
    /**
    * @param {string} val
    * @returns {Options}
    */
    static deserialize(val) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            const ptr0 = passStringToWasm0(val, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            wasm.options_deserialize(retptr, ptr0, len0);
            var r0 = getInt32Memory0()[retptr / 4 + 0];
            var r1 = getInt32Memory0()[retptr / 4 + 1];
            var r2 = getInt32Memory0()[retptr / 4 + 2];
            if (r2) {
                throw takeObject(r1);
            }
            return Options.__wrap(r0);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
    * @returns {string}
    */
    serialize() {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.options_serialize(retptr, this.ptr);
            var r0 = getInt32Memory0()[retptr / 4 + 0];
            var r1 = getInt32Memory0()[retptr / 4 + 1];
            var r2 = getInt32Memory0()[retptr / 4 + 2];
            var r3 = getInt32Memory0()[retptr / 4 + 3];
            var ptr0 = r0;
            var len0 = r1;
            if (r3) {
                ptr0 = 0; len0 = 0;
                throw takeObject(r2);
            }
            return getStringFromWasm0(ptr0, len0);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
            wasm.__wbindgen_free(ptr0, len0);
        }
    }
}
__exports.Options = Options;
/**
*/
class Point {

    static __wrap(ptr) {
        const obj = Object.create(Point.prototype);
        obj.ptr = ptr;

        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.ptr;
        this.ptr = 0;

        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_point_free(ptr);
    }
    /**
    */
    get x() {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.__wbg_get_point_x(retptr, this.ptr);
            var r0 = getInt32Memory0()[retptr / 4 + 0];
            var r1 = getInt32Memory0()[retptr / 4 + 1];
            u32CvtShim[0] = r0;
            u32CvtShim[1] = r1;
            const n0 = int64CvtShim[0];
            return n0;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
    */
    set x(arg0) {
        int64CvtShim[0] = arg0;
        const low0 = u32CvtShim[0];
        const high0 = u32CvtShim[1];
        wasm.__wbg_set_point_x(this.ptr, low0, high0);
    }
    /**
    */
    get y() {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.__wbg_get_point_y(retptr, this.ptr);
            var r0 = getInt32Memory0()[retptr / 4 + 0];
            var r1 = getInt32Memory0()[retptr / 4 + 1];
            u32CvtShim[0] = r0;
            u32CvtShim[1] = r1;
            const n0 = int64CvtShim[0];
            return n0;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
    */
    set y(arg0) {
        int64CvtShim[0] = arg0;
        const low0 = u32CvtShim[0];
        const high0 = u32CvtShim[1];
        wasm.__wbg_set_point_y(this.ptr, low0, high0);
    }
    /**
    */
    constructor() {
        const ret = wasm.point_new();
        return Point.__wrap(ret);
    }
    /**
    * @returns {Point}
    */
    static origin() {
        const ret = wasm.point_new();
        return Point.__wrap(ret);
    }
}
__exports.Point = Point;
/**
*/
class Rect {

    __destroy_into_raw() {
        const ptr = this.ptr;
        this.ptr = 0;

        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_rect_free(ptr);
    }
    /**
    */
    get top() {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.__wbg_get_rect_top(retptr, this.ptr);
            var r0 = getInt32Memory0()[retptr / 4 + 0];
            var r1 = getInt32Memory0()[retptr / 4 + 1];
            u32CvtShim[0] = r0;
            u32CvtShim[1] = r1;
            const n0 = int64CvtShim[0];
            return n0;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
    */
    set top(arg0) {
        int64CvtShim[0] = arg0;
        const low0 = u32CvtShim[0];
        const high0 = u32CvtShim[1];
        wasm.__wbg_set_rect_top(this.ptr, low0, high0);
    }
    /**
    */
    get left() {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.__wbg_get_rect_left(retptr, this.ptr);
            var r0 = getInt32Memory0()[retptr / 4 + 0];
            var r1 = getInt32Memory0()[retptr / 4 + 1];
            u32CvtShim[0] = r0;
            u32CvtShim[1] = r1;
            const n0 = int64CvtShim[0];
            return n0;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
    */
    set left(arg0) {
        int64CvtShim[0] = arg0;
        const low0 = u32CvtShim[0];
        const high0 = u32CvtShim[1];
        wasm.__wbg_set_rect_left(this.ptr, low0, high0);
    }
    /**
    */
    get bottom() {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.__wbg_get_rect_bottom(retptr, this.ptr);
            var r0 = getInt32Memory0()[retptr / 4 + 0];
            var r1 = getInt32Memory0()[retptr / 4 + 1];
            u32CvtShim[0] = r0;
            u32CvtShim[1] = r1;
            const n0 = int64CvtShim[0];
            return n0;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
    */
    set bottom(arg0) {
        int64CvtShim[0] = arg0;
        const low0 = u32CvtShim[0];
        const high0 = u32CvtShim[1];
        wasm.__wbg_set_rect_bottom(this.ptr, low0, high0);
    }
    /**
    */
    get right() {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.__wbg_get_rect_right(retptr, this.ptr);
            var r0 = getInt32Memory0()[retptr / 4 + 0];
            var r1 = getInt32Memory0()[retptr / 4 + 1];
            u32CvtShim[0] = r0;
            u32CvtShim[1] = r1;
            const n0 = int64CvtShim[0];
            return n0;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
    */
    set right(arg0) {
        int64CvtShim[0] = arg0;
        const low0 = u32CvtShim[0];
        const high0 = u32CvtShim[1];
        wasm.__wbg_set_rect_right(this.ptr, low0, high0);
    }
}
__exports.Rect = Rect;
/**
*/
class ResultSize {

    __destroy_into_raw() {
        const ptr = this.ptr;
        this.ptr = 0;

        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_resultsize_free(ptr);
    }
}
__exports.ResultSize = ResultSize;
/**
*/
class Sides {

    static __wrap(ptr) {
        const obj = Object.create(Sides.prototype);
        obj.ptr = ptr;

        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.ptr;
        this.ptr = 0;

        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_sides_free(ptr);
    }
    /**
    */
    get top() {
        const ret = wasm.__wbg_get_sides_top(this.ptr);
        return ret;
    }
    /**
    */
    set top(arg0) {
        wasm.__wbg_set_sides_top(this.ptr, arg0);
    }
    /**
    */
    get left() {
        const ret = wasm.__wbg_get_sides_left(this.ptr);
        return ret;
    }
    /**
    */
    set left(arg0) {
        wasm.__wbg_set_sides_left(this.ptr, arg0);
    }
    /**
    */
    get bottom() {
        const ret = wasm.__wbg_get_sides_bottom(this.ptr);
        return ret;
    }
    /**
    */
    set bottom(arg0) {
        wasm.__wbg_set_sides_bottom(this.ptr, arg0);
    }
    /**
    */
    get right() {
        const ret = wasm.__wbg_get_sides_right(this.ptr);
        return ret;
    }
    /**
    */
    set right(arg0) {
        wasm.__wbg_set_sides_right(this.ptr, arg0);
    }
    /**
    */
    constructor() {
        const ret = wasm.sides_new();
        return Sides.__wrap(ret);
    }
    /**
    * @param {number} side
    * @returns {Sides}
    */
    static uniform(side) {
        const ret = wasm.sides_uniform(side);
        return Sides.__wrap(ret);
    }
}
__exports.Sides = Sides;
/**
*/
class Size {

    static __wrap(ptr) {
        const obj = Object.create(Size.prototype);
        obj.ptr = ptr;

        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.ptr;
        this.ptr = 0;

        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_size_free(ptr);
    }
    /**
    */
    get width() {
        const ret = wasm.__wbg_get_size_width(this.ptr);
        return ret >>> 0;
    }
    /**
    */
    set width(arg0) {
        wasm.__wbg_set_size_width(this.ptr, arg0);
    }
    /**
    */
    get height() {
        const ret = wasm.__wbg_get_size_height(this.ptr);
        return ret >>> 0;
    }
    /**
    */
    set height(arg0) {
        wasm.__wbg_set_size_height(this.ptr, arg0);
    }
    /**
    */
    constructor() {
        const ret = wasm.size_new();
        return Size.__wrap(ret);
    }
}
__exports.Size = Size;

async function load(module, imports) {
    if (typeof Response === 'function' && module instanceof Response) {
        if (typeof WebAssembly.instantiateStreaming === 'function') {
            try {
                return await WebAssembly.instantiateStreaming(module, imports);

            } catch (e) {
                if (module.headers.get('Content-Type') != 'application/wasm') {
                    console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n", e);

                } else {
                    throw e;
                }
            }
        }

        const bytes = await module.arrayBuffer();
        return await WebAssembly.instantiate(bytes, imports);

    } else {
        const instance = await WebAssembly.instantiate(module, imports);

        if (instance instanceof WebAssembly.Instance) {
            return { instance, module };

        } else {
            return instance;
        }
    }
}

function getImports() {
    const imports = {};
    imports.wbg = {};
    imports.wbg.__wbindgen_error_new = function(arg0, arg1) {
        const ret = new Error(getStringFromWasm0(arg0, arg1));
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_object_drop_ref = function(arg0) {
        takeObject(arg0);
    };
    imports.wbg.__wbindgen_string_new = function(arg0, arg1) {
        const ret = getStringFromWasm0(arg0, arg1);
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_width_30c1691725fcc572 = function(arg0) {
        const ret = getObject(arg0).width;
        return ret;
    };
    imports.wbg.__wbg_height_f8827c2151271a1a = function(arg0) {
        const ret = getObject(arg0).height;
        return ret;
    };
    imports.wbg.__wbg_data_798d534e165849ee = function(arg0, arg1) {
        const ret = getObject(arg1).data;
        const ptr0 = passArray8ToWasm0(ret, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        getInt32Memory0()[arg0 / 4 + 1] = len0;
        getInt32Memory0()[arg0 / 4 + 0] = ptr0;
    };
    imports.wbg.__wbg_newwithu8clampedarrayandsh_53a701efac2b2e8d = function() { return handleError(function (arg0, arg1, arg2, arg3) {
        const ret = new ImageData(getClampedArrayU8FromWasm0(arg0, arg1), arg2 >>> 0, arg3 >>> 0);
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_getImageData_50f6c1b814306c32 = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
        const ret = getObject(arg0).getImageData(arg1, arg2, arg3, arg4);
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_width_ad2acb326fc35bdb = function(arg0) {
        const ret = getObject(arg0).width;
        return ret;
    };
    imports.wbg.__wbg_height_65ee0c47b0a97297 = function(arg0) {
        const ret = getObject(arg0).height;
        return ret;
    };
    imports.wbg.__wbg_getTime_7c8d3b79f51e2b87 = function(arg0) {
        const ret = getObject(arg0).getTime();
        return ret;
    };
    imports.wbg.__wbg_new0_6b49a1fca8534d39 = function() {
        const ret = new Date();
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_debug_string = function(arg0, arg1) {
        const ret = debugString(getObject(arg1));
        const ptr0 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        getInt32Memory0()[arg0 / 4 + 1] = len0;
        getInt32Memory0()[arg0 / 4 + 0] = ptr0;
    };
    imports.wbg.__wbindgen_throw = function(arg0, arg1) {
        throw new Error(getStringFromWasm0(arg0, arg1));
    };

    return imports;
}

function initMemory(imports, maybe_memory) {

}

function finalizeInit(instance, module) {
    wasm = instance.exports;
    init.__wbindgen_wasm_module = module;
    cachedInt32Memory0 = new Int32Array(wasm.memory.buffer);
    cachedUint8Memory0 = new Uint8Array(wasm.memory.buffer);
    cachedUint8ClampedMemory0 = new Uint8ClampedArray(wasm.memory.buffer);


    return wasm;
}

function initSync(bytes) {
    const imports = getImports();

    initMemory(imports);

    const module = new WebAssembly.Module(bytes);
    const instance = new WebAssembly.Instance(module, imports);

    return finalizeInit(instance, module);
}

async function init(input) {
    if (typeof input === 'undefined') {
        let src;
        if (typeof document === 'undefined') {
            src = location.href;
        } else {
            src = document.currentScript.src;
        }
        input = src.replace(/\.js$/, '_bg.wasm');
    }
    const imports = getImports();

    if (typeof input === 'string' || (typeof Request === 'function' && input instanceof Request) || (typeof URL === 'function' && input instanceof URL)) {
        input = fetch(input);
    }

    initMemory(imports);

    const { instance, module } = await load(await input, imports);

    return finalizeInit(instance, module);
}

wasm_bindgen = Object.assign(init, __exports);

})();
