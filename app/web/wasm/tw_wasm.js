/* @ts-self-types="./tw_wasm.d.ts" */

/**
 * Browser engine handle (inline executor). Register fonts before opening documents.
 */
export class TwEngine {
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        TwEngineFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_twengine_free(ptr, 0);
    }
    /**
     * @returns {number}
     */
    accept_all_revisions() {
        const ret = wasm.twengine_accept_all_revisions(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * @param {string} caret_run
     * @returns {number}
     */
    accept_revision_at(caret_run) {
        const ptr0 = passStringToWasm0(caret_run, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.twengine_accept_revision_at(this.__wbg_ptr, ptr0, len0);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * @returns {string}
     */
    accessibility_issues_json() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.twengine_accessibility_issues_json(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @param {string} key
     * @param {string} author
     * @param {string} title
     * @param {string} year
     * @returns {number}
     */
    add_bibliography_source(key, author, title, year) {
        const ptr0 = passStringToWasm0(key, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(author, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        const ptr2 = passStringToWasm0(title, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len2 = WASM_VECTOR_LEN;
        const ptr3 = passStringToWasm0(year, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len3 = WASM_VECTOR_LEN;
        const ret = wasm.twengine_add_bibliography_source(this.__wbg_ptr, ptr0, len0, ptr1, len1, ptr2, len2, ptr3, len3);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * @param {string} caret_run
     * @param {boolean} forward
     * @returns {string}
     */
    adjacent_revision_run(caret_run, forward) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(caret_run, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.twengine_adjacent_revision_run(this.__wbg_ptr, ptr0, len0, forward);
            var ptr2 = ret[0];
            var len2 = ret[1];
            if (ret[3]) {
                ptr2 = 0; len2 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred3_0 = ptr2;
            deferred3_1 = len2;
            return getStringFromWasm0(ptr2, len2);
        } finally {
            wasm.__wbindgen_free(deferred3_0, deferred3_1, 1);
        }
    }
    /**
     * @param {string} caret_run_id
     * @returns {number}
     */
    apply_bullet_list(caret_run_id) {
        const ptr0 = passStringToWasm0(caret_run_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.twengine_apply_bullet_list(this.__wbg_ptr, ptr0, len0);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * @param {string} theme_name
     * @returns {number}
     */
    apply_document_theme(theme_name) {
        const ptr0 = passStringToWasm0(theme_name, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.twengine_apply_document_theme(this.__wbg_ptr, ptr0, len0);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * @param {string} caret_run_id
     * @returns {number}
     */
    apply_heading1(caret_run_id) {
        const ptr0 = passStringToWasm0(caret_run_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.twengine_apply_heading1(this.__wbg_ptr, ptr0, len0);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * @param {string} values_json
     * @returns {number}
     */
    apply_mail_merge_row(values_json) {
        const ptr0 = passStringToWasm0(values_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.twengine_apply_mail_merge_row(this.__wbg_ptr, ptr0, len0);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * @param {string} caret_run_id
     * @returns {number}
     */
    apply_normal_style(caret_run_id) {
        const ptr0 = passStringToWasm0(caret_run_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.twengine_apply_normal_style(this.__wbg_ptr, ptr0, len0);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * @param {string} caret_run_id
     * @returns {number}
     */
    apply_numbered_list(caret_run_id) {
        const ptr0 = passStringToWasm0(caret_run_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.twengine_apply_numbered_list(this.__wbg_ptr, ptr0, len0);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * @param {string} caret_run_id
     * @param {string} style_name
     * @returns {number}
     */
    apply_paragraph_style(caret_run_id, style_name) {
        const ptr0 = passStringToWasm0(caret_run_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(style_name, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        const ret = wasm.twengine_apply_paragraph_style(this.__wbg_ptr, ptr0, len0, ptr1, len1);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * @param {string} format_json
     * @param {string} caret_run_id
     * @returns {number}
     */
    apply_section_format(format_json, caret_run_id) {
        const ptr0 = passStringToWasm0(format_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(caret_run_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        const ret = wasm.twengine_apply_section_format(this.__wbg_ptr, ptr0, len0, ptr1, len1);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * @returns {Uint8Array}
     */
    atlas_bytes() {
        const ret = wasm.twengine_atlas_bytes(this.__wbg_ptr);
        var v1 = getArrayU8FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        return v1;
    }
    /**
     * @returns {number}
     */
    atlas_generation() {
        const ret = wasm.twengine_atlas_generation(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    atlas_height() {
        const ret = wasm.twengine_atlas_height(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    atlas_width() {
        const ret = wasm.twengine_atlas_width(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @param {string} caret_run_id
     * @returns {number}
     */
    autofit_table(caret_run_id) {
        const ptr0 = passStringToWasm0(caret_run_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.twengine_autofit_table(this.__wbg_ptr, ptr0, len0);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * @returns {string}
     */
    bookmarks_json() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.twengine_bookmarks_json(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @param {number} page
     * @param {string} run_id
     * @param {number} offset
     * @returns {string | undefined}
     */
    caret_at(page, run_id, offset) {
        const ptr0 = passStringToWasm0(run_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.twengine_caret_at(this.__wbg_ptr, page, ptr0, len0, offset);
        let v2;
        if (ret[0] !== 0) {
            v2 = getStringFromWasm0(ret[0], ret[1]).slice();
            wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        }
        return v2;
    }
    /**
     * @param {string} run_id
     * @returns {string | undefined}
     */
    caret_format_json(run_id) {
        const ptr0 = passStringToWasm0(run_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.twengine_caret_format_json(this.__wbg_ptr, ptr0, len0);
        let v2;
        if (ret[0] !== 0) {
            v2 = getStringFromWasm0(ret[0], ret[1]).slice();
            wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        }
        return v2;
    }
    /**
     * @param {number} page
     * @param {number} x
     * @param {number} y
     * @returns {string | undefined}
     */
    caret_geometry(page, x, y) {
        const ret = wasm.twengine_caret_geometry(this.__wbg_ptr, page, x, y);
        let v1;
        if (ret[0] !== 0) {
            v1 = getStringFromWasm0(ret[0], ret[1]).slice();
            wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        }
        return v1;
    }
    /**
     * @returns {number}
     */
    clear_digital_signatures() {
        const ret = wasm.twengine_clear_digital_signatures(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * @param {string} start_run
     * @param {number} start_offset
     * @param {string} end_run
     * @param {number} end_offset
     * @returns {number}
     */
    clear_format(start_run, start_offset, end_run, end_offset) {
        const ptr0 = passStringToWasm0(start_run, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(end_run, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        const ret = wasm.twengine_clear_format(this.__wbg_ptr, ptr0, len0, start_offset, ptr1, len1, end_offset);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * @param {string} other
     * @returns {string}
     */
    compare_document_text(other) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ptr0 = passStringToWasm0(other, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.twengine_compare_document_text(this.__wbg_ptr, ptr0, len0);
            deferred2_0 = ret[0];
            deferred2_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * @param {string} image_id
     * @param {number} quality
     * @returns {number}
     */
    compress_image(image_id, quality) {
        const ptr0 = passStringToWasm0(image_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.twengine_compress_image(this.__wbg_ptr, ptr0, len0, quality);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * @param {string} block_id
     * @returns {number}
     */
    delete_block(block_id) {
        const ptr0 = passStringToWasm0(block_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.twengine_delete_block(this.__wbg_ptr, ptr0, len0);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * @param {string} caret_run_id
     * @returns {number}
     */
    delete_table_column(caret_run_id) {
        const ptr0 = passStringToWasm0(caret_run_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.twengine_delete_table_column(this.__wbg_ptr, ptr0, len0);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * @param {string} caret_run_id
     * @returns {number}
     */
    delete_table_row(caret_run_id) {
        const ptr0 = passStringToWasm0(caret_run_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.twengine_delete_table_row(this.__wbg_ptr, ptr0, len0);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * @returns {string}
     */
    digital_signatures_json() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.twengine_digital_signatures_json(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @param {Uint8Array} data
     * @returns {number}
     */
    dispatch(data) {
        const ptr0 = passArray8ToWasm0(data, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.twengine_dispatch(this.__wbg_ptr, ptr0, len0);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * @returns {Uint8Array}
     */
    display_list_bytes() {
        const ret = wasm.twengine_display_list_bytes(this.__wbg_ptr);
        var v1 = getArrayU8FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        return v1;
    }
    /**
     * @returns {number}
     */
    display_list_page_count() {
        const ret = wasm.twengine_display_list_page_count(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    display_list_page_height() {
        const ret = wasm.twengine_display_list_page_height(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    display_list_page_width() {
        const ret = wasm.twengine_display_list_page_width(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    display_list_version() {
        const ret = wasm.twengine_display_list_version(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {string}
     */
    document_inspect_json() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.twengine_document_inspect_json(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @returns {string}
     */
    document_outline_json() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.twengine_document_outline_json(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @returns {string}
     */
    document_properties_json() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.twengine_document_properties_json(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @param {number} page
     * @returns {string | undefined}
     */
    document_tail_hit(page) {
        const ret = wasm.twengine_document_tail_hit(this.__wbg_ptr, page);
        let v1;
        if (ret[0] !== 0) {
            v1 = getStringFromWasm0(ret[0], ret[1]).slice();
            wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        }
        return v1;
    }
    /**
     * @param {string} caret_run_id
     * @param {boolean} is_header
     * @param {number} page_index
     * @returns {number}
     */
    ensure_header_footer(caret_run_id, is_header, page_index) {
        const ptr0 = passStringToWasm0(caret_run_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.twengine_ensure_header_footer(this.__wbg_ptr, ptr0, len0, is_header, page_index);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * @returns {boolean}
     */
    even_and_odd_headers_enabled() {
        const ret = wasm.twengine_even_and_odd_headers_enabled(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @returns {Uint8Array}
     */
    export_pdf() {
        const ret = wasm.twengine_export_pdf(this.__wbg_ptr);
        if (ret[3]) {
            throw takeFromExternrefTable0(ret[2]);
        }
        var v1 = getArrayU8FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        return v1;
    }
    /**
     * @param {number} scale_mode
     * @param {number} scale_percent
     * @param {number} margin_left
     * @param {number} margin_right
     * @param {number} margin_top
     * @param {number} margin_bottom
     * @param {number} duplex
     * @param {number} pages_per_sheet
     * @param {number} booklet
     * @returns {Uint8Array}
     */
    export_pdf_for_print(scale_mode, scale_percent, margin_left, margin_right, margin_top, margin_bottom, duplex, pages_per_sheet, booklet) {
        const ret = wasm.twengine_export_pdf_for_print(this.__wbg_ptr, scale_mode, scale_percent, margin_left, margin_right, margin_top, margin_bottom, duplex, pages_per_sheet, booklet);
        if (ret[3]) {
            throw takeFromExternrefTable0(ret[2]);
        }
        var v1 = getArrayU8FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        return v1;
    }
    /**
     * @param {string} start_run_id
     * @param {number} start_offset
     * @param {string} end_run_id
     * @param {number} end_offset
     * @param {number} scale_mode
     * @param {number} scale_percent
     * @param {number} margin_left
     * @param {number} margin_right
     * @param {number} margin_top
     * @param {number} margin_bottom
     * @param {number} duplex
     * @param {number} pages_per_sheet
     * @param {number} booklet
     * @returns {Uint8Array}
     */
    export_pdf_for_print_selection(start_run_id, start_offset, end_run_id, end_offset, scale_mode, scale_percent, margin_left, margin_right, margin_top, margin_bottom, duplex, pages_per_sheet, booklet) {
        const ptr0 = passStringToWasm0(start_run_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(end_run_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        const ret = wasm.twengine_export_pdf_for_print_selection(this.__wbg_ptr, ptr0, len0, start_offset, ptr1, len1, end_offset, scale_mode, scale_percent, margin_left, margin_right, margin_top, margin_bottom, duplex, pages_per_sheet, booklet);
        if (ret[3]) {
            throw takeFromExternrefTable0(ret[2]);
        }
        var v3 = getArrayU8FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        return v3;
    }
    /**
     * @param {string} query
     * @param {boolean} match_case
     * @param {boolean} use_regex
     * @param {boolean} use_wildcards
     * @param {string} format_json
     * @returns {string}
     */
    find_matches(query, match_case, use_regex, use_wildcards, format_json) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(query, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ptr1 = passStringToWasm0(format_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            const ret = wasm.twengine_find_matches(this.__wbg_ptr, ptr0, len0, match_case, use_regex, use_wildcards, ptr1, len1);
            deferred3_0 = ret[0];
            deferred3_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred3_0, deferred3_1, 1);
        }
    }
    /**
     * @returns {number}
     */
    first_line_advance() {
        const ret = wasm.twengine_first_line_advance(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {string} shape_id
     * @returns {string}
     */
    get_chart_data_json(shape_id) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(shape_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.twengine_get_chart_data_json(this.__wbg_ptr, ptr0, len0);
            var ptr2 = ret[0];
            var len2 = ret[1];
            if (ret[3]) {
                ptr2 = 0; len2 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred3_0 = ptr2;
            deferred3_1 = len2;
            return getStringFromWasm0(ptr2, len2);
        } finally {
            wasm.__wbindgen_free(deferred3_0, deferred3_1, 1);
        }
    }
    /**
     * @param {string} run_id
     * @returns {string}
     */
    get_office_math_xml(run_id) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(run_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.twengine_get_office_math_xml(this.__wbg_ptr, ptr0, len0);
            var ptr2 = ret[0];
            var len2 = ret[1];
            if (ret[3]) {
                ptr2 = 0; len2 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred3_0 = ptr2;
            deferred3_1 = len2;
            return getStringFromWasm0(ptr2, len2);
        } finally {
            wasm.__wbindgen_free(deferred3_0, deferred3_1, 1);
        }
    }
    /**
     * @param {string} caret_run_id
     * @returns {string}
     */
    get_section_format_json(caret_run_id) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(caret_run_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.twengine_get_section_format_json(this.__wbg_ptr, ptr0, len0);
            var ptr2 = ret[0];
            var len2 = ret[1];
            if (ret[3]) {
                ptr2 = 0; len2 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred3_0 = ptr2;
            deferred3_1 = len2;
            return getStringFromWasm0(ptr2, len2);
        } finally {
            wasm.__wbindgen_free(deferred3_0, deferred3_1, 1);
        }
    }
    /**
     * @returns {string}
     */
    grammar_check() {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.twengine_grammar_check(this.__wbg_ptr);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * @param {string} caret_run_id
     * @param {boolean} is_header
     * @param {number} page_index
     * @returns {boolean}
     */
    header_footer_linked(caret_run_id, is_header, page_index) {
        const ptr0 = passStringToWasm0(caret_run_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.twengine_header_footer_linked(this.__wbg_ptr, ptr0, len0, is_header, page_index);
        return ret !== 0;
    }
    /**
     * @param {string} caret_run_id
     * @param {boolean} is_header
     * @param {number} page_index
     * @returns {string}
     */
    header_footer_seed_run(caret_run_id, is_header, page_index) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(caret_run_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.twengine_header_footer_seed_run(this.__wbg_ptr, ptr0, len0, is_header, page_index);
            var ptr2 = ret[0];
            var len2 = ret[1];
            if (ret[3]) {
                ptr2 = 0; len2 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred3_0 = ptr2;
            deferred3_1 = len2;
            return getStringFromWasm0(ptr2, len2);
        } finally {
            wasm.__wbindgen_free(deferred3_0, deferred3_1, 1);
        }
    }
    /**
     * @param {number} page
     * @param {number} x
     * @param {number} y
     * @returns {string | undefined}
     */
    hit_test(page, x, y) {
        const ret = wasm.twengine_hit_test(this.__wbg_ptr, page, x, y);
        let v1;
        if (ret[0] !== 0) {
            v1 = getStringFromWasm0(ret[0], ret[1]).slice();
            wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        }
        return v1;
    }
    /**
     * @param {string} image_id
     * @returns {string}
     */
    image_alt_text(image_id) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(image_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.twengine_image_alt_text(this.__wbg_ptr, ptr0, len0);
            var ptr2 = ret[0];
            var len2 = ret[1];
            if (ret[3]) {
                ptr2 = 0; len2 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred3_0 = ptr2;
            deferred3_1 = len2;
            return getStringFromWasm0(ptr2, len2);
        } finally {
            wasm.__wbindgen_free(deferred3_0, deferred3_1, 1);
        }
    }
    /**
     * @param {string} caret_run_id
     * @returns {number}
     */
    insert_bibliography(caret_run_id) {
        const ptr0 = passStringToWasm0(caret_run_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.twengine_insert_bibliography(this.__wbg_ptr, ptr0, len0);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * @param {string} run_id
     * @param {number} offset
     * @param {string} name
     * @returns {number}
     */
    insert_bookmark(run_id, offset, name) {
        const ptr0 = passStringToWasm0(run_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(name, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        const ret = wasm.twengine_insert_bookmark(this.__wbg_ptr, ptr0, len0, offset, ptr1, len1);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * @param {number} chart_type
     * @returns {number}
     */
    insert_chart(chart_type) {
        const ret = wasm.twengine_insert_chart(this.__wbg_ptr, chart_type);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * @param {string} run_id
     * @param {number} offset
     * @param {string} source_key
     * @returns {number}
     */
    insert_citation(run_id, offset, source_key) {
        const ptr0 = passStringToWasm0(run_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(source_key, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        const ret = wasm.twengine_insert_citation(this.__wbg_ptr, ptr0, len0, offset, ptr1, len1);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * @param {string} run_id
     * @param {number} offset
     * @param {string} body_text
     * @returns {number}
     */
    insert_comment(run_id, offset, body_text) {
        const ptr0 = passStringToWasm0(run_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(body_text, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        const ret = wasm.twengine_insert_comment(this.__wbg_ptr, ptr0, len0, offset, ptr1, len1);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * @param {string} run_id
     * @param {number} offset
     * @param {string} bookmark_name
     * @returns {number}
     */
    insert_cross_reference(run_id, offset, bookmark_name) {
        const ptr0 = passStringToWasm0(run_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(bookmark_name, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        const ret = wasm.twengine_insert_cross_reference(this.__wbg_ptr, ptr0, len0, offset, ptr1, len1);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * @param {number} diagram_type
     * @returns {number}
     */
    insert_diagram(diagram_type) {
        const ret = wasm.twengine_insert_diagram(this.__wbg_ptr, diagram_type);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * @param {string} run_id
     * @param {number} offset
     * @param {string} field_type
     * @returns {number}
     */
    insert_field(run_id, offset, field_type) {
        const ptr0 = passStringToWasm0(run_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(field_type, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        const ret = wasm.twengine_insert_field(this.__wbg_ptr, ptr0, len0, offset, ptr1, len1);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * @param {string} run_id
     * @param {number} offset
     * @returns {number}
     */
    insert_footnote(run_id, offset) {
        const ptr0 = passStringToWasm0(run_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.twengine_insert_footnote(this.__wbg_ptr, ptr0, len0, offset);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * @param {string} run_id
     * @param {number} offset
     * @param {string} kind
     * @param {string} name
     * @param {string} initial_value
     * @returns {number}
     */
    insert_form_field(run_id, offset, kind, name, initial_value) {
        const ptr0 = passStringToWasm0(run_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(kind, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        const ptr2 = passStringToWasm0(name, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len2 = WASM_VECTOR_LEN;
        const ptr3 = passStringToWasm0(initial_value, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len3 = WASM_VECTOR_LEN;
        const ret = wasm.twengine_insert_form_field(this.__wbg_ptr, ptr0, len0, offset, ptr1, len1, ptr2, len2, ptr3, len3);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * @param {string} run_id
     * @param {number} offset
     * @param {string} url
     * @param {string} text
     * @param {string} tooltip
     * @returns {number}
     */
    insert_hyperlink(run_id, offset, url, text, tooltip) {
        const ptr0 = passStringToWasm0(run_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(url, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        const ptr2 = passStringToWasm0(text, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len2 = WASM_VECTOR_LEN;
        const ptr3 = passStringToWasm0(tooltip, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len3 = WASM_VECTOR_LEN;
        const ret = wasm.twengine_insert_hyperlink(this.__wbg_ptr, ptr0, len0, offset, ptr1, len1, ptr2, len2, ptr3, len3);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * @param {number} width
     * @param {number} height
     * @returns {number}
     */
    insert_image(width, height) {
        const ret = wasm.twengine_insert_image(this.__wbg_ptr, width, height);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * @param {Uint8Array} data
     * @param {string} mime_type
     * @returns {number}
     */
    insert_image_bytes(data, mime_type) {
        const ptr0 = passArray8ToWasm0(data, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(mime_type, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        const ret = wasm.twengine_insert_image_bytes(this.__wbg_ptr, ptr0, len0, ptr1, len1);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * @param {string} image_id
     * @returns {number}
     */
    insert_image_caption(image_id) {
        const ptr0 = passStringToWasm0(image_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.twengine_insert_image_caption(this.__wbg_ptr, ptr0, len0);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * @param {string} caret_run_id
     * @returns {number}
     */
    insert_index(caret_run_id) {
        const ptr0 = passStringToWasm0(caret_run_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.twengine_insert_index(this.__wbg_ptr, ptr0, len0);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * @param {string} run_id
     * @param {number} offset
     * @param {string} name
     * @returns {number}
     */
    insert_merge_field(run_id, offset, name) {
        const ptr0 = passStringToWasm0(run_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(name, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        const ret = wasm.twengine_insert_merge_field(this.__wbg_ptr, ptr0, len0, offset, ptr1, len1);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * @param {string} caret_run_id
     * @param {number} rows
     * @param {number} cols
     * @returns {number}
     */
    insert_nested_table(caret_run_id, rows, cols) {
        const ptr0 = passStringToWasm0(caret_run_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.twengine_insert_nested_table(this.__wbg_ptr, ptr0, len0, rows, cols);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * @param {string} run_id
     * @param {number} offset
     * @param {string} xml
     * @returns {number}
     */
    insert_office_math(run_id, offset, xml) {
        const ptr0 = passStringToWasm0(run_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(xml, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        const ret = wasm.twengine_insert_office_math(this.__wbg_ptr, ptr0, len0, offset, ptr1, len1);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * @param {string | null | undefined} caret_run_id
     * @param {string} xml
     * @returns {number}
     */
    insert_office_math_display(caret_run_id, xml) {
        var ptr0 = isLikeNone(caret_run_id) ? 0 : passStringToWasm0(caret_run_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(xml, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        const ret = wasm.twengine_insert_office_math_display(this.__wbg_ptr, ptr0, len0, ptr1, len1);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * @param {string} caret_run_id
     * @returns {number}
     */
    insert_page_break(caret_run_id) {
        const ptr0 = passStringToWasm0(caret_run_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.twengine_insert_page_break(this.__wbg_ptr, ptr0, len0);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * @param {string} caret_run_id
     * @returns {number}
     */
    insert_section_break(caret_run_id) {
        const ptr0 = passStringToWasm0(caret_run_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.twengine_insert_section_break(this.__wbg_ptr, ptr0, len0);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * @param {number} shape_type
     * @returns {number}
     */
    insert_shape(shape_type) {
        const ret = wasm.twengine_insert_shape(this.__wbg_ptr, shape_type);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * @param {number} rows
     * @param {number} cols
     * @param {string} caret_run_id
     * @returns {number}
     */
    insert_table(rows, cols, caret_run_id) {
        const ptr0 = passStringToWasm0(caret_run_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.twengine_insert_table(this.__wbg_ptr, rows, cols, ptr0, len0);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * @param {string} caret_run_id
     * @returns {number}
     */
    insert_table_of_contents(caret_run_id) {
        const ptr0 = passStringToWasm0(caret_run_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.twengine_insert_table_of_contents(this.__wbg_ptr, ptr0, len0);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * @param {string} caret_run_id
     * @returns {number}
     */
    insert_table_sum_field(caret_run_id) {
        const ptr0 = passStringToWasm0(caret_run_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.twengine_insert_table_sum_field(this.__wbg_ptr, ptr0, len0);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * @returns {number}
     */
    insert_text_box() {
        const ret = wasm.twengine_insert_text_box(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * @param {string} text
     * @returns {number}
     */
    insert_word_art(text) {
        const ptr0 = passStringToWasm0(text, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.twengine_insert_word_art(this.__wbg_ptr, ptr0, len0);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * @param {number} page
     * @returns {boolean}
     */
    is_page_stale(page) {
        const ret = wasm.twengine_is_page_stale(this.__wbg_ptr, page);
        return ret !== 0;
    }
    /**
     * @returns {boolean}
     */
    is_read_only() {
        const ret = wasm.twengine_is_read_only(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @returns {string}
     */
    last_error() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.twengine_last_error(this.__wbg_ptr);
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
    last_request_id() {
        const ret = wasm.twengine_last_request_id(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {string | undefined}
     */
    last_split_caret() {
        const ret = wasm.twengine_last_split_caret(this.__wbg_ptr);
        let v1;
        if (ret[0] !== 0) {
            v1 = getStringFromWasm0(ret[0], ret[1]).slice();
            wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        }
        return v1;
    }
    /**
     * @returns {string}
     */
    latest_chart_id() {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.twengine_latest_chart_id(this.__wbg_ptr);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * @returns {string}
     */
    latest_office_math_run_id() {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.twengine_latest_office_math_run_id(this.__wbg_ptr);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * @param {string} caret_run_id
     * @returns {number}
     */
    merge_table_cells(caret_run_id) {
        const ptr0 = passStringToWasm0(caret_run_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.twengine_merge_table_cells(this.__wbg_ptr, ptr0, len0);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    constructor() {
        const ret = wasm.twengine_new();
        this.__wbg_ptr = ret;
        TwEngineFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    new_document() {
        const ret = wasm.twengine_new_document(this.__wbg_ptr);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * @param {Uint8Array} data
     */
    open_document(data) {
        const ptr0 = passArray8ToWasm0(data, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.twengine_open_document(this.__wbg_ptr, ptr0, len0);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Open document bytes; [password] decrypts encrypted DOCX when non-empty (F22.S1).
     * @param {Uint8Array} data
     * @param {string} path
     * @param {string} password
     */
    open_document_with_password(data, path, password) {
        const ptr0 = passArray8ToWasm0(data, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(path, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        const ptr2 = passStringToWasm0(password, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len2 = WASM_VECTOR_LEN;
        const ret = wasm.twengine_open_document_with_password(this.__wbg_ptr, ptr0, len0, ptr1, len1, ptr2, len2);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * @param {Uint8Array} data
     * @param {string} path
     */
    open_document_with_path(data, path) {
        const ptr0 = passArray8ToWasm0(data, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(path, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        const ret = wasm.twengine_open_document_with_path(this.__wbg_ptr, ptr0, len0, ptr1, len1);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * @returns {number}
     */
    page_count() {
        const ret = wasm.twengine_page_count(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @param {number} page
     * @returns {Uint8Array | undefined}
     */
    page_display_list_bytes(page) {
        const ret = wasm.twengine_page_display_list_bytes(this.__wbg_ptr, page);
        let v1;
        if (ret[0] !== 0) {
            v1 = getArrayU8FromWasm0(ret[0], ret[1]).slice();
            wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        }
        return v1;
    }
    /**
     * @param {number} page
     * @returns {number | undefined}
     */
    page_display_list_page_height(page) {
        const ret = wasm.twengine_page_display_list_page_height(this.__wbg_ptr, page);
        return ret === Number.MAX_SAFE_INTEGER ? undefined : ret;
    }
    /**
     * @param {number} page
     * @returns {number | undefined}
     */
    page_display_list_page_width(page) {
        const ret = wasm.twengine_page_display_list_page_width(this.__wbg_ptr, page);
        return ret === Number.MAX_SAFE_INTEGER ? undefined : ret;
    }
    /**
     * @param {number} page
     * @returns {number | undefined}
     */
    page_display_list_version(page) {
        const ret = wasm.twengine_page_display_list_version(this.__wbg_ptr, page);
        return ret[0] === 0 ? undefined : ret[1];
    }
    /**
     * @param {string} run_id
     * @param {number} offset
     * @param {Uint8Array} data
     * @returns {number}
     */
    paste_docx(run_id, offset, data) {
        const ptr0 = passStringToWasm0(run_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passArray8ToWasm0(data, wasm.__wbindgen_malloc);
        const len1 = WASM_VECTOR_LEN;
        const ret = wasm.twengine_paste_docx(this.__wbg_ptr, ptr0, len0, offset, ptr1, len1);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * @param {string} run_id
     * @param {number} offset
     * @param {string} html
     * @returns {number}
     */
    paste_html(run_id, offset, html) {
        const ptr0 = passStringToWasm0(run_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(html, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        const ret = wasm.twengine_paste_html(this.__wbg_ptr, ptr0, len0, offset, ptr1, len1);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * @returns {string | undefined}
     */
    pop_event() {
        const ret = wasm.twengine_pop_event(this.__wbg_ptr);
        let v1;
        if (ret[0] !== 0) {
            v1 = getStringFromWasm0(ret[0], ret[1]).slice();
            wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        }
        return v1;
    }
    /**
     * @returns {number}
     */
    pump() {
        const ret = wasm.twengine_pump(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    redo() {
        const ret = wasm.twengine_redo(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * @param {string} family
     * @param {boolean} bold
     * @param {boolean} italic
     * @param {Uint8Array} data
     */
    register_font(family, bold, italic, data) {
        const ptr0 = passStringToWasm0(family, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passArray8ToWasm0(data, wasm.__wbindgen_malloc);
        const len1 = WASM_VECTOR_LEN;
        const ret = wasm.twengine_register_font(this.__wbg_ptr, ptr0, len0, bold, italic, ptr1, len1);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * @returns {number}
     */
    reject_all_revisions() {
        const ret = wasm.twengine_reject_all_revisions(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * @param {string} caret_run
     * @returns {number}
     */
    reject_revision_at(caret_run) {
        const ptr0 = passStringToWasm0(caret_run, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.twengine_reject_revision_at(this.__wbg_ptr, ptr0, len0);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * @param {boolean} comments
     * @param {boolean} metadata
     * @param {boolean} hidden_text
     * @returns {number}
     */
    remove_inspect_findings(comments, metadata, hidden_text) {
        const ret = wasm.twengine_remove_inspect_findings(this.__wbg_ptr, comments, metadata, hidden_text);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * @param {string} image_id
     * @param {Uint8Array} data
     * @param {string} mime_type
     * @returns {number}
     */
    replace_image_bytes(image_id, data, mime_type) {
        const ptr0 = passStringToWasm0(image_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passArray8ToWasm0(data, wasm.__wbindgen_malloc);
        const len1 = WASM_VECTOR_LEN;
        const ptr2 = passStringToWasm0(mime_type, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len2 = WASM_VECTOR_LEN;
        const ret = wasm.twengine_replace_image_bytes(this.__wbg_ptr, ptr0, len0, ptr1, len1, ptr2, len2);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * @param {string} caret_run_id
     * @param {number} width
     * @returns {number}
     */
    resize_table_column(caret_run_id, width) {
        const ptr0 = passStringToWasm0(caret_run_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.twengine_resize_table_column(this.__wbg_ptr, ptr0, len0, width);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * @returns {Uint8Array}
     */
    save_document() {
        const ret = wasm.twengine_save_document(this.__wbg_ptr);
        if (ret[3]) {
            throw takeFromExternrefTable0(ret[2]);
        }
        var v1 = getArrayU8FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        return v1;
    }
    /**
     * @param {string} extension
     * @returns {Uint8Array}
     */
    save_document_as(extension) {
        const ptr0 = passStringToWasm0(extension, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.twengine_save_document_as(this.__wbg_ptr, ptr0, len0);
        if (ret[3]) {
            throw takeFromExternrefTable0(ret[2]);
        }
        var v2 = getArrayU8FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        return v2;
    }
    /**
     * @param {number} page
     * @param {number} start_x
     * @param {number} start_y
     * @param {number} end_x
     * @param {number} end_y
     * @returns {Float32Array}
     */
    selection_rects(page, start_x, start_y, end_x, end_y) {
        const ret = wasm.twengine_selection_rects(this.__wbg_ptr, page, start_x, start_y, end_x, end_y);
        var v1 = getArrayF32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * @returns {string}
     */
    semantic_tree_json() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.twengine_semantic_tree_json(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @param {string} shape_id
     * @param {string} chart_json
     * @returns {number}
     */
    set_chart_data_json(shape_id, chart_json) {
        const ptr0 = passStringToWasm0(shape_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(chart_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        const ret = wasm.twengine_set_chart_data_json(this.__wbg_ptr, ptr0, len0, ptr1, len1);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * @param {number} page
     * @returns {number}
     */
    set_current_page(page) {
        const ret = wasm.twengine_set_current_page(this.__wbg_ptr, page);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * Set or clear DOCX encryption password for subsequent saves (F22.S2).
     * Empty string clears the password.
     * @param {string} password
     * @returns {number}
     */
    set_encryption_password(password) {
        const ptr0 = passStringToWasm0(password, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.twengine_set_encryption_password(this.__wbg_ptr, ptr0, len0);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * @param {boolean} enabled
     * @returns {number}
     */
    set_even_and_odd_headers(enabled) {
        const ret = wasm.twengine_set_even_and_odd_headers(this.__wbg_ptr, enabled);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * @param {string} run_id
     * @param {string} value
     * @returns {number}
     */
    set_form_field_value(run_id, value) {
        const ptr0 = passStringToWasm0(run_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(value, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        const ret = wasm.twengine_set_form_field_value(this.__wbg_ptr, ptr0, len0, ptr1, len1);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * @param {string} caret_run_id
     * @param {boolean} is_header
     * @param {number} page_index
     * @param {boolean} linked
     * @returns {number}
     */
    set_header_footer_link(caret_run_id, is_header, page_index, linked) {
        const ptr0 = passStringToWasm0(caret_run_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.twengine_set_header_footer_link(this.__wbg_ptr, ptr0, len0, is_header, page_index, linked);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * @param {string} image_id
     * @param {string | null} [alt_text]
     * @returns {number}
     */
    set_image_alt_text(image_id, alt_text) {
        const ptr0 = passStringToWasm0(image_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        var ptr1 = isLikeNone(alt_text) ? 0 : passStringToWasm0(alt_text, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len1 = WASM_VECTOR_LEN;
        const ret = wasm.twengine_set_image_alt_text(this.__wbg_ptr, ptr0, len0, ptr1, len1);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * @param {string} image_id
     * @param {number} x
     * @param {number} y
     * @param {number} origin_x
     * @param {number} origin_y
     * @returns {number}
     */
    set_image_anchor(image_id, x, y, origin_x, origin_y) {
        const ptr0 = passStringToWasm0(image_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.twengine_set_image_anchor(this.__wbg_ptr, ptr0, len0, x, y, origin_x, origin_y);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * @param {string} image_id
     * @param {number} width
     * @param {number} height
     * @returns {number}
     */
    set_image_size(image_id, width, height) {
        const ptr0 = passStringToWasm0(image_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.twengine_set_image_size(this.__wbg_ptr, ptr0, len0, width, height);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * @param {string} image_id
     * @param {number} rotation_deg
     * @param {number} crop_left
     * @param {number} crop_top
     * @param {number} crop_right
     * @param {number} crop_bottom
     * @param {number} opacity
     * @returns {number}
     */
    set_image_transform(image_id, rotation_deg, crop_left, crop_top, crop_right, crop_bottom, opacity) {
        const ptr0 = passStringToWasm0(image_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.twengine_set_image_transform(this.__wbg_ptr, ptr0, len0, rotation_deg, crop_left, crop_top, crop_right, crop_bottom, opacity);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * @param {string} image_id
     * @param {number} wrap
     * @returns {number}
     */
    set_image_wrap(image_id, wrap) {
        const ptr0 = passStringToWasm0(image_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.twengine_set_image_wrap(this.__wbg_ptr, ptr0, len0, wrap);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * @param {string} run_id
     * @param {string} xml
     * @returns {number}
     */
    set_office_math_xml(run_id, xml) {
        const ptr0 = passStringToWasm0(run_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(xml, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        const ret = wasm.twengine_set_office_math_xml(this.__wbg_ptr, ptr0, len0, ptr1, len1);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * @param {boolean} enabled
     * @returns {number}
     */
    set_read_only(enabled) {
        const ret = wasm.twengine_set_read_only(this.__wbg_ptr, enabled);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * @param {string} caret_run_id
     * @param {number} width
     * @param {number} color_r
     * @param {number} color_g
     * @param {number} color_b
     * @param {number} color_a
     * @returns {number}
     */
    set_table_border(caret_run_id, width, color_r, color_g, color_b, color_a) {
        const ptr0 = passStringToWasm0(caret_run_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.twengine_set_table_border(this.__wbg_ptr, ptr0, len0, width, color_r, color_g, color_b, color_a);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * @param {string} caret_run_id
     * @param {number} color_r
     * @param {number} color_g
     * @param {number} color_b
     * @param {number} color_a
     * @returns {number}
     */
    set_table_cell_shading(caret_run_id, color_r, color_g, color_b, color_a) {
        const ptr0 = passStringToWasm0(caret_run_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.twengine_set_table_cell_shading(this.__wbg_ptr, ptr0, len0, color_r, color_g, color_b, color_a);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * @param {boolean} enabled
     * @returns {number}
     */
    set_track_changes(enabled) {
        const ret = wasm.twengine_set_track_changes(this.__wbg_ptr, enabled);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * @param {string} name
     * @param {string} email
     * @param {string | null} [organization]
     * @returns {number}
     */
    sign_document(name, email, organization) {
        const ptr0 = passStringToWasm0(name, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(email, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        var ptr2 = isLikeNone(organization) ? 0 : passStringToWasm0(organization, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len2 = WASM_VECTOR_LEN;
        const ret = wasm.twengine_sign_document(this.__wbg_ptr, ptr0, len0, ptr1, len1, ptr2, len2);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * @param {string} caret_run_id
     * @param {boolean} ascending
     * @returns {number}
     */
    sort_table_rows(caret_run_id, ascending) {
        const ptr0 = passStringToWasm0(caret_run_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.twengine_sort_table_rows(this.__wbg_ptr, ptr0, len0, ascending);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * @returns {string}
     */
    spell_check() {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.twengine_spell_check(this.__wbg_ptr);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * @param {string} caret_run_id
     * @returns {number}
     */
    split_table_cell(caret_run_id) {
        const ptr0 = passStringToWasm0(caret_run_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.twengine_split_table_cell(this.__wbg_ptr, ptr0, len0);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * @returns {string}
     */
    text() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.twengine_text(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @param {string} start_run
     * @param {number} start_offset
     * @param {string} end_run
     * @param {number} end_offset
     * @returns {string | undefined}
     */
    text_in_range(start_run, start_offset, end_run, end_offset) {
        const ptr0 = passStringToWasm0(start_run, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(end_run, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        const ret = wasm.twengine_text_in_range(this.__wbg_ptr, ptr0, len0, start_offset, ptr1, len1, end_offset);
        let v3;
        if (ret[0] !== 0) {
            v3 = getStringFromWasm0(ret[0], ret[1]).slice();
            wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        }
        return v3;
    }
    /**
     * @returns {number}
     */
    undo() {
        const ret = wasm.twengine_undo(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * @returns {string}
     */
    verify_signatures_json() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.twengine_verify_signatures_json(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
}
if (Symbol.dispose) TwEngine.prototype[Symbol.dispose] = TwEngine.prototype.free;
function __wbg_get_imports() {
    const import0 = {
        __proto__: null,
        __wbg___wbindgen_is_function_1ff95bcc5517c252: function(arg0) {
            const ret = typeof(arg0) === 'function';
            return ret;
        },
        __wbg___wbindgen_is_object_a27215656b807791: function(arg0) {
            const val = arg0;
            const ret = typeof(val) === 'object' && val !== null;
            return ret;
        },
        __wbg___wbindgen_is_string_ea5e6cc2e4141dfe: function(arg0) {
            const ret = typeof(arg0) === 'string';
            return ret;
        },
        __wbg___wbindgen_is_undefined_c05833b95a3cf397: function(arg0) {
            const ret = arg0 === undefined;
            return ret;
        },
        __wbg___wbindgen_throw_344f42d3211c4765: function(arg0, arg1) {
            throw new Error(getStringFromWasm0(arg0, arg1));
        },
        __wbg_call_a6e5c5dce5018821: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = arg0.call(arg1, arg2);
            return ret;
        }, arguments); },
        __wbg_crypto_38df2bab126b63dc: function(arg0) {
            const ret = arg0.crypto;
            return ret;
        },
        __wbg_getRandomValues_3f44b700395062e5: function() { return handleError(function (arg0, arg1) {
            globalThis.crypto.getRandomValues(getArrayU8FromWasm0(arg0, arg1));
        }, arguments); },
        __wbg_getRandomValues_c44a50d8cfdaebeb: function() { return handleError(function (arg0, arg1) {
            arg0.getRandomValues(arg1);
        }, arguments); },
        __wbg_getRandomValues_ceb34d8ffce7e87f: function() { return handleError(function (arg0, arg1) {
            globalThis.crypto.getRandomValues(getArrayU8FromWasm0(arg0, arg1));
        }, arguments); },
        __wbg_getTime_d6f070c088c9b5ed: function(arg0) {
            const ret = arg0.getTime();
            return ret;
        },
        __wbg_length_1f0964f4a5e2c6d8: function(arg0) {
            const ret = arg0.length;
            return ret;
        },
        __wbg_msCrypto_bd5a034af96bcba6: function(arg0) {
            const ret = arg0.msCrypto;
            return ret;
        },
        __wbg_new_0_3da9e97f24fc69be: function() {
            const ret = new Date();
            return ret;
        },
        __wbg_new_with_length_e6785c33c8e4cce8: function(arg0) {
            const ret = new Uint8Array(arg0 >>> 0);
            return ret;
        },
        __wbg_node_84ea875411254db1: function(arg0) {
            const ret = arg0.node;
            return ret;
        },
        __wbg_now_e7c6795a7f81e10f: function(arg0) {
            const ret = arg0.now();
            return ret;
        },
        __wbg_performance_3fcf6e32a7e1ed0a: function(arg0) {
            const ret = arg0.performance;
            return ret;
        },
        __wbg_process_44c7a14e11e9f69e: function(arg0) {
            const ret = arg0.process;
            return ret;
        },
        __wbg_prototypesetcall_4770620bbe4688a0: function(arg0, arg1, arg2) {
            Uint8Array.prototype.set.call(getArrayU8FromWasm0(arg0, arg1), arg2);
        },
        __wbg_randomFillSync_6c25eac9869eb53c: function() { return handleError(function (arg0, arg1) {
            arg0.randomFillSync(arg1);
        }, arguments); },
        __wbg_require_b4edbdcf3e2a1ef0: function() { return handleError(function () {
            const ret = module.require;
            return ret;
        }, arguments); },
        __wbg_static_accessor_GLOBAL_4ef717fb391d88b7: function() {
            const ret = typeof global === 'undefined' ? null : global;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_static_accessor_GLOBAL_THIS_8d1badc68b5a74f4: function() {
            const ret = typeof globalThis === 'undefined' ? null : globalThis;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_static_accessor_SELF_146583524fe1469b: function() {
            const ret = typeof self === 'undefined' ? null : self;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_static_accessor_WINDOW_f2829a2234d7819e: function() {
            const ret = typeof window === 'undefined' ? null : window;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_subarray_3ed232c8a6baee09: function(arg0, arg1, arg2) {
            const ret = arg0.subarray(arg1 >>> 0, arg2 >>> 0);
            return ret;
        },
        __wbg_versions_276b2795b1c6a219: function(arg0) {
            const ret = arg0.versions;
            return ret;
        },
        __wbindgen_cast_0000000000000001: function(arg0, arg1) {
            // Cast intrinsic for `Ref(Slice(U8)) -> NamedExternref("Uint8Array")`.
            const ret = getArrayU8FromWasm0(arg0, arg1);
            return ret;
        },
        __wbindgen_cast_0000000000000002: function(arg0, arg1) {
            // Cast intrinsic for `Ref(String) -> Externref`.
            const ret = getStringFromWasm0(arg0, arg1);
            return ret;
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
        "./tw_wasm_bg.js": import0,
    };
}

const TwEngineFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_twengine_free(ptr, 1));

function addToExternrefTable0(obj) {
    const idx = wasm.__externref_table_alloc();
    wasm.__wbindgen_externrefs.set(idx, obj);
    return idx;
}

function getArrayF32FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getFloat32ArrayMemory0().subarray(ptr / 4, ptr / 4 + len);
}

function getArrayU8FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getUint8ArrayMemory0().subarray(ptr / 1, ptr / 1 + len);
}

let cachedFloat32ArrayMemory0 = null;
function getFloat32ArrayMemory0() {
    if (cachedFloat32ArrayMemory0 === null || cachedFloat32ArrayMemory0.byteLength === 0) {
        cachedFloat32ArrayMemory0 = new Float32Array(wasm.memory.buffer);
    }
    return cachedFloat32ArrayMemory0;
}

function getStringFromWasm0(ptr, len) {
    return decodeText(ptr >>> 0, len);
}

let cachedUint8ArrayMemory0 = null;
function getUint8ArrayMemory0() {
    if (cachedUint8ArrayMemory0 === null || cachedUint8ArrayMemory0.byteLength === 0) {
        cachedUint8ArrayMemory0 = new Uint8Array(wasm.memory.buffer);
    }
    return cachedUint8ArrayMemory0;
}

function handleError(f, args) {
    try {
        return f.apply(this, args);
    } catch (e) {
        const idx = addToExternrefTable0(e);
        wasm.__wbindgen_exn_store(idx);
    }
}

function isLikeNone(x) {
    return x === undefined || x === null;
}

function passArray8ToWasm0(arg, malloc) {
    const ptr = malloc(arg.length * 1, 1) >>> 0;
    getUint8ArrayMemory0().set(arg, ptr / 1);
    WASM_VECTOR_LEN = arg.length;
    return ptr;
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

function takeFromExternrefTable0(idx) {
    const value = wasm.__wbindgen_externrefs.get(idx);
    wasm.__externref_table_dealloc(idx);
    return value;
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

let wasmModule, wasmInstance, wasm;
function __wbg_finalize_init(instance, module) {
    wasmInstance = instance;
    wasm = instance.exports;
    wasmModule = module;
    cachedFloat32ArrayMemory0 = null;
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
        module_or_path = new URL('tw_wasm_bg.wasm', import.meta.url);
    }
    const imports = __wbg_get_imports();

    if (typeof module_or_path === 'string' || (typeof Request === 'function' && module_or_path instanceof Request) || (typeof URL === 'function' && module_or_path instanceof URL)) {
        module_or_path = fetch(module_or_path);
    }

    const { instance, module } = await __wbg_load(await module_or_path, imports);

    return __wbg_finalize_init(instance, module);
}

export { initSync, __wbg_init as default };
