/* @ts-self-types="./grid_wasm.d.ts" */

export class Grid {
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        GridFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_grid_free(ptr, 0);
    }
    build_pivot() {
        wasm.grid_build_pivot(this.__wbg_ptr);
    }
    /**
     * @returns {number}
     */
    cell_count() {
        const ret = wasm.grid_cell_count(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @param {number} r
     * @param {number} c
     * @returns {string}
     */
    cell_screen_rect(r, c) {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.grid_cell_screen_rect(this.__wbg_ptr, r, c);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @param {number} r
     * @param {number} c
     */
    clear_cell(r, c) {
        wasm.grid_clear_cell(this.__wbg_ptr, r, c);
    }
    /**
     * @param {number} x
     */
    click_h_track(x) {
        wasm.grid_click_h_track(this.__wbg_ptr, x);
    }
    /**
     * @param {number} y
     */
    click_v_track(y) {
        wasm.grid_click_v_track(this.__wbg_ptr, y);
    }
    /**
     * @param {number} r
     * @param {number} c
     */
    edit(r, c) {
        wasm.grid_edit(this.__wbg_ptr, r, c);
    }
    /**
     * @returns {number}
     */
    edit_col() {
        const ret = wasm.grid_edit_col(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    edit_row() {
        const ret = wasm.grid_edit_row(this.__wbg_ptr);
        return ret;
    }
    end_col_resize() {
        wasm.grid_end_col_resize(this.__wbg_ptr);
    }
    end_drag() {
        wasm.grid_end_drag(this.__wbg_ptr);
    }
    /**
     * @param {number} r
     * @param {number} c
     * @returns {string}
     */
    get_cell_text(r, c) {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.grid_get_cell_text(this.__wbg_ptr, r, c);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @returns {number}
     */
    get_scroll_x() {
        const ret = wasm.grid_get_scroll_x(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    get_scroll_y() {
        const ret = wasm.grid_get_scroll_y(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    get_total_cols() {
        const ret = wasm.grid_get_total_cols(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    get_total_rows() {
        const ret = wasm.grid_get_total_rows(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @param {number} cx
     * @param {number} cy
     * @returns {string}
     */
    hit_test(cx, cy) {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.grid_hit_test(this.__wbg_ptr, cx, cy);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @returns {boolean}
     */
    is_dragging_scrollbar() {
        const ret = wasm.grid_is_dragging_scrollbar(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @returns {boolean}
     */
    is_resizing() {
        const ret = wasm.grid_is_resizing(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @param {string} json
     */
    load_data(json) {
        const ptr0 = passStringToWasm0(json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        wasm.grid_load_data(this.__wbg_ptr, ptr0, len0);
    }
    /**
     * @param {number} dr
     * @param {number} dc
     */
    move_selection(dr, dc) {
        wasm.grid_move_selection(this.__wbg_ptr, dr, dc);
    }
    /**
     * @param {number} w
     * @param {number} h
     */
    constructor(w, h) {
        const ret = wasm.grid_new(w, h);
        this.__wbg_ptr = ret >>> 0;
        GridFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * @returns {string}
     */
    render_frame() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.grid_render_frame(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @param {number} dx
     * @param {number} dy
     */
    scroll_by(dx, dy) {
        wasm.grid_scroll_by(this.__wbg_ptr, dx, dy);
    }
    /**
     * @returns {number}
     */
    sel_col() {
        const ret = wasm.grid_sel_col(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    sel_row() {
        const ret = wasm.grid_sel_row(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {number} r
     * @param {number} c
     */
    select(r, c) {
        wasm.grid_select(this.__wbg_ptr, r, c);
    }
    /**
     * @param {number} r
     * @param {number} c
     * @param {string} t
     */
    set_cell(r, c, t) {
        const ptr0 = passStringToWasm0(t, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        wasm.grid_set_cell(this.__wbg_ptr, r, c, ptr0, len0);
    }
    /**
     * @param {number} c
     * @param {number} w
     */
    set_col_width(c, w) {
        wasm.grid_set_col_width(this.__wbg_ptr, c, w);
    }
    /**
     * @param {string} json
     */
    set_pivot_config(json) {
        const ptr0 = passStringToWasm0(json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        wasm.grid_set_pivot_config(this.__wbg_ptr, ptr0, len0);
    }
    /**
     * @param {number} x
     * @param {number} y
     */
    set_scroll(x, y) {
        wasm.grid_set_scroll(this.__wbg_ptr, x, y);
    }
    /**
     * @param {number} w
     * @param {number} h
     */
    set_viewport(w, h) {
        wasm.grid_set_viewport(this.__wbg_ptr, w, h);
    }
    /**
     * @param {number} c
     * @param {number} x
     */
    start_col_resize(c, x) {
        wasm.grid_start_col_resize(this.__wbg_ptr, c, x);
    }
    /**
     * @param {number} x
     */
    start_h_drag(x) {
        wasm.grid_start_h_drag(this.__wbg_ptr, x);
    }
    /**
     * @param {number} y
     */
    start_v_drag(y) {
        wasm.grid_start_v_drag(this.__wbg_ptr, y);
    }
    /**
     * @param {string} k
     */
    toggle_col_collapse(k) {
        const ptr0 = passStringToWasm0(k, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        wasm.grid_toggle_col_collapse(this.__wbg_ptr, ptr0, len0);
    }
    /**
     * @param {string} k
     */
    toggle_row_collapse(k) {
        const ptr0 = passStringToWasm0(k, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        wasm.grid_toggle_row_collapse(this.__wbg_ptr, ptr0, len0);
    }
    /**
     * @param {number} x
     */
    update_col_resize(x) {
        wasm.grid_update_col_resize(this.__wbg_ptr, x);
    }
    /**
     * @param {number} x
     */
    update_h_drag(x) {
        wasm.grid_update_h_drag(this.__wbg_ptr, x);
    }
    /**
     * @param {number} y
     */
    update_v_drag(y) {
        wasm.grid_update_v_drag(this.__wbg_ptr, y);
    }
}
if (Symbol.dispose) Grid.prototype[Symbol.dispose] = Grid.prototype.free;

function __wbg_get_imports() {
    const import0 = {
        __proto__: null,
        __wbg___wbindgen_throw_6ddd609b62940d55: function(arg0, arg1) {
            throw new Error(getStringFromWasm0(arg0, arg1));
        },
        __wbindgen_init_externref_table: function() {
            const table = wasm.__wbindgen_externrefs;
            const offset = table.grow(4);
            table.set(0, undefined);
            table.set(offset + 0, undefined);
            table.set(offset + 1, null);
            table.set(offset + 2, true);
            table.set(offset + 3, false);
        },
    };
    return {
        __proto__: null,
        "./grid_wasm_bg.js": import0,
    };
}

const GridFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_grid_free(ptr >>> 0, 1));

function getStringFromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return decodeText(ptr, len);
}

let cachedUint8ArrayMemory0 = null;
function getUint8ArrayMemory0() {
    if (cachedUint8ArrayMemory0 === null || cachedUint8ArrayMemory0.byteLength === 0) {
        cachedUint8ArrayMemory0 = new Uint8Array(wasm.memory.buffer);
    }
    return cachedUint8ArrayMemory0;
}

function passStringToWasm0(arg, malloc, realloc) {
    if (realloc === undefined) {
        const buf = cachedTextEncoder.encode(arg);
        const ptr = malloc(buf.length, 1) >>> 0;
        getUint8ArrayMemory0().subarray(ptr, ptr + buf.length).set(buf);
        WASM_VECTOR_LEN = buf.length;
        return ptr;
    }

    let len = arg.length;
    let ptr = malloc(len, 1) >>> 0;

    const mem = getUint8ArrayMemory0();

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
        ptr = realloc(ptr, len, len = offset + arg.length * 3, 1) >>> 0;
        const view = getUint8ArrayMemory0().subarray(ptr + offset, ptr + len);
        const ret = cachedTextEncoder.encodeInto(arg, view);

        offset += ret.written;
        ptr = realloc(ptr, len, offset, 1) >>> 0;
    }

    WASM_VECTOR_LEN = offset;
    return ptr;
}

let cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
cachedTextDecoder.decode();
const MAX_SAFARI_DECODE_BYTES = 2146435072;
let numBytesDecoded = 0;
function decodeText(ptr, len) {
    numBytesDecoded += len;
    if (numBytesDecoded >= MAX_SAFARI_DECODE_BYTES) {
        cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
        cachedTextDecoder.decode();
        numBytesDecoded = len;
    }
    return cachedTextDecoder.decode(getUint8ArrayMemory0().subarray(ptr, ptr + len));
}

const cachedTextEncoder = new TextEncoder();

if (!('encodeInto' in cachedTextEncoder)) {
    cachedTextEncoder.encodeInto = function (arg, view) {
        const buf = cachedTextEncoder.encode(arg);
        view.set(buf);
        return {
            read: arg.length,
            written: buf.length
        };
    };
}

let WASM_VECTOR_LEN = 0;

let wasmModule, wasm;
function __wbg_finalize_init(instance, module) {
    wasm = instance.exports;
    wasmModule = module;
    cachedUint8ArrayMemory0 = null;
    wasm.__wbindgen_start();
    return wasm;
}

async function __wbg_load(module, imports) {
    if (typeof Response === 'function' && module instanceof Response) {
        if (typeof WebAssembly.instantiateStreaming === 'function') {
            try {
                return await WebAssembly.instantiateStreaming(module, imports);
            } catch (e) {
                const validResponse = module.ok && expectedResponseType(module.type);

                if (validResponse && module.headers.get('Content-Type') !== 'application/wasm') {
                    console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve Wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n", e);

                } else { throw e; }
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

    function expectedResponseType(type) {
        switch (type) {
            case 'basic': case 'cors': case 'default': return true;
        }
        return false;
    }
}

function initSync(module) {
    if (wasm !== undefined) return wasm;


    if (module !== undefined) {
        if (Object.getPrototypeOf(module) === Object.prototype) {
            ({module} = module)
        } else {
            console.warn('using deprecated parameters for `initSync()`; pass a single object instead')
        }
    }

    const imports = __wbg_get_imports();
    if (!(module instanceof WebAssembly.Module)) {
        module = new WebAssembly.Module(module);
    }
    const instance = new WebAssembly.Instance(module, imports);
    return __wbg_finalize_init(instance, module);
}

async function __wbg_init(module_or_path) {
    if (wasm !== undefined) return wasm;


    if (module_or_path !== undefined) {
        if (Object.getPrototypeOf(module_or_path) === Object.prototype) {
            ({module_or_path} = module_or_path)
        } else {
            console.warn('using deprecated parameters for the initialization function; pass a single object instead')
        }
    }

    if (module_or_path === undefined) {
        module_or_path = new URL('grid_wasm_bg.wasm', import.meta.url);
    }
    const imports = __wbg_get_imports();

    if (typeof module_or_path === 'string' || (typeof Request === 'function' && module_or_path instanceof Request) || (typeof URL === 'function' && module_or_path instanceof URL)) {
        module_or_path = fetch(module_or_path);
    }

    const { instance, module } = await __wbg_load(await module_or_path, imports);

    return __wbg_finalize_init(instance, module);
}

export { initSync, __wbg_init as default };
