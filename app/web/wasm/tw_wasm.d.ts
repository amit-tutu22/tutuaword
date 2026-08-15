/* tslint:disable */
/* eslint-disable */

/**
 * Browser engine handle (inline executor). Register fonts before opening documents.
 */
export class TwEngine {
    free(): void;
    [Symbol.dispose](): void;
    accept_all_revisions(): number;
    accept_revision_at(caret_run: string): number;
    accessibility_issues_json(): string;
    add_bibliography_source(key: string, author: string, title: string, year: string): number;
    adjacent_revision_run(caret_run: string, forward: boolean): string;
    apply_bullet_list(caret_run_id: string): number;
    apply_document_theme(theme_name: string): number;
    apply_heading1(caret_run_id: string): number;
    apply_mail_merge_row(values_json: string): number;
    apply_normal_style(caret_run_id: string): number;
    apply_numbered_list(caret_run_id: string): number;
    apply_paragraph_style(caret_run_id: string, style_name: string): number;
    apply_section_format(format_json: string, caret_run_id: string): number;
    atlas_bytes(): Uint8Array;
    atlas_generation(): number;
    atlas_height(): number;
    atlas_width(): number;
    autofit_table(caret_run_id: string): number;
    bookmarks_json(): string;
    caret_at(page: number, run_id: string, offset: number): string | undefined;
    caret_format_json(run_id: string): string | undefined;
    caret_geometry(page: number, x: number, y: number): string | undefined;
    clear_digital_signatures(): number;
    clear_format(start_run: string, start_offset: number, end_run: string, end_offset: number): number;
    compare_document_text(other: string): string;
    compress_image(image_id: string, quality: number): number;
    delete_block(block_id: string): number;
    delete_table_column(caret_run_id: string): number;
    delete_table_row(caret_run_id: string): number;
    digital_signatures_json(): string;
    dispatch(data: Uint8Array): number;
    display_list_bytes(): Uint8Array;
    display_list_page_count(): number;
    display_list_page_height(): number;
    display_list_page_width(): number;
    display_list_version(): number;
    document_inspect_json(): string;
    document_outline_json(): string;
    document_properties_json(): string;
    document_tail_hit(page: number): string | undefined;
    ensure_header_footer(caret_run_id: string, is_header: boolean, page_index: number): number;
    even_and_odd_headers_enabled(): boolean;
    export_pdf(): Uint8Array;
    export_pdf_for_print(scale_mode: number, scale_percent: number, margin_left: number, margin_right: number, margin_top: number, margin_bottom: number, duplex: number, pages_per_sheet: number, booklet: number): Uint8Array;
    export_pdf_for_print_selection(start_run_id: string, start_offset: number, end_run_id: string, end_offset: number, scale_mode: number, scale_percent: number, margin_left: number, margin_right: number, margin_top: number, margin_bottom: number, duplex: number, pages_per_sheet: number, booklet: number): Uint8Array;
    find_matches(query: string, match_case: boolean, use_regex: boolean, use_wildcards: boolean, format_json: string): string;
    first_line_advance(): number;
    get_chart_data_json(shape_id: string): string;
    get_office_math_xml(run_id: string): string;
    get_section_format_json(caret_run_id: string): string;
    grammar_check(): string;
    header_footer_linked(caret_run_id: string, is_header: boolean, page_index: number): boolean;
    header_footer_seed_run(caret_run_id: string, is_header: boolean, page_index: number): string;
    hit_test(page: number, x: number, y: number): string | undefined;
    image_alt_text(image_id: string): string;
    insert_bibliography(caret_run_id: string): number;
    insert_bookmark(run_id: string, offset: number, name: string): number;
    insert_chart(chart_type: number): number;
    insert_citation(run_id: string, offset: number, source_key: string): number;
    insert_comment(run_id: string, offset: number, body_text: string): number;
    insert_cross_reference(run_id: string, offset: number, bookmark_name: string): number;
    insert_diagram(diagram_type: number): number;
    insert_field(run_id: string, offset: number, field_type: string): number;
    insert_footnote(run_id: string, offset: number): number;
    insert_form_field(run_id: string, offset: number, kind: string, name: string, initial_value: string): number;
    insert_hyperlink(run_id: string, offset: number, url: string, text: string, tooltip: string): number;
    insert_image(width: number, height: number): number;
    insert_image_bytes(data: Uint8Array, mime_type: string): number;
    insert_image_caption(image_id: string): number;
    insert_index(caret_run_id: string): number;
    insert_merge_field(run_id: string, offset: number, name: string): number;
    insert_nested_table(caret_run_id: string, rows: number, cols: number): number;
    insert_office_math(run_id: string, offset: number, xml: string): number;
    insert_office_math_display(caret_run_id: string | null | undefined, xml: string): number;
    insert_page_break(caret_run_id: string): number;
    insert_section_break(caret_run_id: string): number;
    insert_shape(shape_type: number): number;
    insert_table(rows: number, cols: number, caret_run_id: string): number;
    insert_table_of_contents(caret_run_id: string): number;
    insert_table_sum_field(caret_run_id: string): number;
    insert_text_box(): number;
    insert_word_art(text: string): number;
    is_page_stale(page: number): boolean;
    is_read_only(): boolean;
    last_error(): string;
    last_request_id(): number;
    last_split_caret(): string | undefined;
    latest_chart_id(): string;
    latest_office_math_run_id(): string;
    merge_table_cells(caret_run_id: string): number;
    constructor();
    new_document(): void;
    open_document(data: Uint8Array): void;
    /**
     * Open document bytes; [password] decrypts encrypted DOCX when non-empty (F22.S1).
     */
    open_document_with_password(data: Uint8Array, path: string, password: string): void;
    open_document_with_path(data: Uint8Array, path: string): void;
    page_count(): number;
    page_display_list_bytes(page: number): Uint8Array | undefined;
    page_display_list_page_height(page: number): number | undefined;
    page_display_list_page_width(page: number): number | undefined;
    page_display_list_version(page: number): number | undefined;
    paste_docx(run_id: string, offset: number, data: Uint8Array): number;
    paste_html(run_id: string, offset: number, html: string): number;
    pop_event(): string | undefined;
    pump(): number;
    redo(): number;
    register_font(family: string, bold: boolean, italic: boolean, data: Uint8Array): void;
    reject_all_revisions(): number;
    reject_revision_at(caret_run: string): number;
    remove_inspect_findings(comments: boolean, metadata: boolean, hidden_text: boolean): number;
    replace_image_bytes(image_id: string, data: Uint8Array, mime_type: string): number;
    resize_table_column(caret_run_id: string, width: number): number;
    save_document(): Uint8Array;
    save_document_as(extension: string): Uint8Array;
    selection_rects(page: number, start_x: number, start_y: number, end_x: number, end_y: number): Float32Array;
    semantic_tree_json(): string;
    set_chart_data_json(shape_id: string, chart_json: string): number;
    set_current_page(page: number): number;
    /**
     * Set or clear DOCX encryption password for subsequent saves (F22.S2).
     * Empty string clears the password.
     */
    set_encryption_password(password: string): number;
    set_even_and_odd_headers(enabled: boolean): number;
    set_form_field_value(run_id: string, value: string): number;
    set_header_footer_link(caret_run_id: string, is_header: boolean, page_index: number, linked: boolean): number;
    set_image_alt_text(image_id: string, alt_text?: string | null): number;
    set_image_anchor(image_id: string, x: number, y: number, origin_x: number, origin_y: number): number;
    set_image_size(image_id: string, width: number, height: number): number;
    set_image_transform(image_id: string, rotation_deg: number, crop_left: number, crop_top: number, crop_right: number, crop_bottom: number, opacity: number): number;
    set_image_wrap(image_id: string, wrap: number): number;
    set_office_math_xml(run_id: string, xml: string): number;
    set_read_only(enabled: boolean): number;
    set_table_border(caret_run_id: string, width: number, color_r: number, color_g: number, color_b: number, color_a: number): number;
    set_table_cell_shading(caret_run_id: string, color_r: number, color_g: number, color_b: number, color_a: number): number;
    set_track_changes(enabled: boolean): number;
    sign_document(name: string, email: string, organization?: string | null): number;
    sort_table_rows(caret_run_id: string, ascending: boolean): number;
    spell_check(): string;
    split_table_cell(caret_run_id: string): number;
    text(): string;
    text_in_range(start_run: string, start_offset: number, end_run: string, end_offset: number): string | undefined;
    undo(): number;
    verify_signatures_json(): string;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __wbg_twengine_free: (a: number, b: number) => void;
    readonly twengine_accept_all_revisions: (a: number) => [number, number, number];
    readonly twengine_accept_revision_at: (a: number, b: number, c: number) => [number, number, number];
    readonly twengine_accessibility_issues_json: (a: number) => [number, number];
    readonly twengine_add_bibliography_source: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number) => [number, number, number];
    readonly twengine_adjacent_revision_run: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly twengine_apply_bullet_list: (a: number, b: number, c: number) => [number, number, number];
    readonly twengine_apply_document_theme: (a: number, b: number, c: number) => [number, number, number];
    readonly twengine_apply_heading1: (a: number, b: number, c: number) => [number, number, number];
    readonly twengine_apply_mail_merge_row: (a: number, b: number, c: number) => [number, number, number];
    readonly twengine_apply_normal_style: (a: number, b: number, c: number) => [number, number, number];
    readonly twengine_apply_numbered_list: (a: number, b: number, c: number) => [number, number, number];
    readonly twengine_apply_paragraph_style: (a: number, b: number, c: number, d: number, e: number) => [number, number, number];
    readonly twengine_apply_section_format: (a: number, b: number, c: number, d: number, e: number) => [number, number, number];
    readonly twengine_atlas_bytes: (a: number) => [number, number];
    readonly twengine_atlas_generation: (a: number) => number;
    readonly twengine_atlas_height: (a: number) => number;
    readonly twengine_atlas_width: (a: number) => number;
    readonly twengine_autofit_table: (a: number, b: number, c: number) => [number, number, number];
    readonly twengine_bookmarks_json: (a: number) => [number, number];
    readonly twengine_caret_at: (a: number, b: number, c: number, d: number, e: number) => [number, number];
    readonly twengine_caret_format_json: (a: number, b: number, c: number) => [number, number];
    readonly twengine_caret_geometry: (a: number, b: number, c: number, d: number) => [number, number];
    readonly twengine_clear_digital_signatures: (a: number) => [number, number, number];
    readonly twengine_clear_format: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => [number, number, number];
    readonly twengine_compare_document_text: (a: number, b: number, c: number) => [number, number];
    readonly twengine_compress_image: (a: number, b: number, c: number, d: number) => [number, number, number];
    readonly twengine_delete_block: (a: number, b: number, c: number) => [number, number, number];
    readonly twengine_delete_table_column: (a: number, b: number, c: number) => [number, number, number];
    readonly twengine_delete_table_row: (a: number, b: number, c: number) => [number, number, number];
    readonly twengine_digital_signatures_json: (a: number) => [number, number];
    readonly twengine_dispatch: (a: number, b: number, c: number) => [number, number, number];
    readonly twengine_display_list_bytes: (a: number) => [number, number];
    readonly twengine_display_list_page_count: (a: number) => number;
    readonly twengine_display_list_page_height: (a: number) => number;
    readonly twengine_display_list_page_width: (a: number) => number;
    readonly twengine_display_list_version: (a: number) => number;
    readonly twengine_document_inspect_json: (a: number) => [number, number];
    readonly twengine_document_outline_json: (a: number) => [number, number];
    readonly twengine_document_properties_json: (a: number) => [number, number];
    readonly twengine_document_tail_hit: (a: number, b: number) => [number, number];
    readonly twengine_ensure_header_footer: (a: number, b: number, c: number, d: number, e: number) => [number, number, number];
    readonly twengine_even_and_odd_headers_enabled: (a: number) => number;
    readonly twengine_export_pdf: (a: number) => [number, number, number, number];
    readonly twengine_export_pdf_for_print: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number) => [number, number, number, number];
    readonly twengine_export_pdf_for_print_selection: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number, k: number, l: number, m: number, n: number, o: number, p: number) => [number, number, number, number];
    readonly twengine_find_matches: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => [number, number];
    readonly twengine_first_line_advance: (a: number) => number;
    readonly twengine_get_chart_data_json: (a: number, b: number, c: number) => [number, number, number, number];
    readonly twengine_get_office_math_xml: (a: number, b: number, c: number) => [number, number, number, number];
    readonly twengine_get_section_format_json: (a: number, b: number, c: number) => [number, number, number, number];
    readonly twengine_grammar_check: (a: number) => [number, number, number, number];
    readonly twengine_header_footer_linked: (a: number, b: number, c: number, d: number, e: number) => number;
    readonly twengine_header_footer_seed_run: (a: number, b: number, c: number, d: number, e: number) => [number, number, number, number];
    readonly twengine_hit_test: (a: number, b: number, c: number, d: number) => [number, number];
    readonly twengine_image_alt_text: (a: number, b: number, c: number) => [number, number, number, number];
    readonly twengine_insert_bibliography: (a: number, b: number, c: number) => [number, number, number];
    readonly twengine_insert_bookmark: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number];
    readonly twengine_insert_chart: (a: number, b: number) => [number, number, number];
    readonly twengine_insert_citation: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number];
    readonly twengine_insert_comment: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number];
    readonly twengine_insert_cross_reference: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number];
    readonly twengine_insert_diagram: (a: number, b: number) => [number, number, number];
    readonly twengine_insert_field: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number];
    readonly twengine_insert_footnote: (a: number, b: number, c: number, d: number) => [number, number, number];
    readonly twengine_insert_form_field: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number) => [number, number, number];
    readonly twengine_insert_hyperlink: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number) => [number, number, number];
    readonly twengine_insert_image: (a: number, b: number, c: number) => [number, number, number];
    readonly twengine_insert_image_bytes: (a: number, b: number, c: number, d: number, e: number) => [number, number, number];
    readonly twengine_insert_image_caption: (a: number, b: number, c: number) => [number, number, number];
    readonly twengine_insert_index: (a: number, b: number, c: number) => [number, number, number];
    readonly twengine_insert_merge_field: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number];
    readonly twengine_insert_nested_table: (a: number, b: number, c: number, d: number, e: number) => [number, number, number];
    readonly twengine_insert_office_math: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number];
    readonly twengine_insert_office_math_display: (a: number, b: number, c: number, d: number, e: number) => [number, number, number];
    readonly twengine_insert_page_break: (a: number, b: number, c: number) => [number, number, number];
    readonly twengine_insert_section_break: (a: number, b: number, c: number) => [number, number, number];
    readonly twengine_insert_shape: (a: number, b: number) => [number, number, number];
    readonly twengine_insert_table: (a: number, b: number, c: number, d: number, e: number) => [number, number, number];
    readonly twengine_insert_table_of_contents: (a: number, b: number, c: number) => [number, number, number];
    readonly twengine_insert_table_sum_field: (a: number, b: number, c: number) => [number, number, number];
    readonly twengine_insert_text_box: (a: number) => [number, number, number];
    readonly twengine_insert_word_art: (a: number, b: number, c: number) => [number, number, number];
    readonly twengine_is_page_stale: (a: number, b: number) => number;
    readonly twengine_is_read_only: (a: number) => number;
    readonly twengine_last_error: (a: number) => [number, number];
    readonly twengine_last_request_id: (a: number) => number;
    readonly twengine_last_split_caret: (a: number) => [number, number];
    readonly twengine_latest_chart_id: (a: number) => [number, number, number, number];
    readonly twengine_latest_office_math_run_id: (a: number) => [number, number, number, number];
    readonly twengine_merge_table_cells: (a: number, b: number, c: number) => [number, number, number];
    readonly twengine_new: () => number;
    readonly twengine_new_document: (a: number) => [number, number];
    readonly twengine_open_document: (a: number, b: number, c: number) => [number, number];
    readonly twengine_open_document_with_password: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => [number, number];
    readonly twengine_open_document_with_path: (a: number, b: number, c: number, d: number, e: number) => [number, number];
    readonly twengine_page_count: (a: number) => number;
    readonly twengine_page_display_list_bytes: (a: number, b: number) => [number, number];
    readonly twengine_page_display_list_page_height: (a: number, b: number) => number;
    readonly twengine_page_display_list_page_width: (a: number, b: number) => number;
    readonly twengine_page_display_list_version: (a: number, b: number) => [number, number];
    readonly twengine_paste_docx: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number];
    readonly twengine_paste_html: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number];
    readonly twengine_pop_event: (a: number) => [number, number];
    readonly twengine_pump: (a: number) => number;
    readonly twengine_redo: (a: number) => [number, number, number];
    readonly twengine_register_font: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => [number, number];
    readonly twengine_reject_all_revisions: (a: number) => [number, number, number];
    readonly twengine_reject_revision_at: (a: number, b: number, c: number) => [number, number, number];
    readonly twengine_remove_inspect_findings: (a: number, b: number, c: number, d: number) => [number, number, number];
    readonly twengine_replace_image_bytes: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => [number, number, number];
    readonly twengine_resize_table_column: (a: number, b: number, c: number, d: number) => [number, number, number];
    readonly twengine_save_document: (a: number) => [number, number, number, number];
    readonly twengine_save_document_as: (a: number, b: number, c: number) => [number, number, number, number];
    readonly twengine_selection_rects: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number];
    readonly twengine_semantic_tree_json: (a: number) => [number, number];
    readonly twengine_set_chart_data_json: (a: number, b: number, c: number, d: number, e: number) => [number, number, number];
    readonly twengine_set_current_page: (a: number, b: number) => [number, number, number];
    readonly twengine_set_encryption_password: (a: number, b: number, c: number) => [number, number, number];
    readonly twengine_set_even_and_odd_headers: (a: number, b: number) => [number, number, number];
    readonly twengine_set_form_field_value: (a: number, b: number, c: number, d: number, e: number) => [number, number, number];
    readonly twengine_set_header_footer_link: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number];
    readonly twengine_set_image_alt_text: (a: number, b: number, c: number, d: number, e: number) => [number, number, number];
    readonly twengine_set_image_anchor: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => [number, number, number];
    readonly twengine_set_image_size: (a: number, b: number, c: number, d: number, e: number) => [number, number, number];
    readonly twengine_set_image_transform: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number) => [number, number, number];
    readonly twengine_set_image_wrap: (a: number, b: number, c: number, d: number) => [number, number, number];
    readonly twengine_set_office_math_xml: (a: number, b: number, c: number, d: number, e: number) => [number, number, number];
    readonly twengine_set_read_only: (a: number, b: number) => [number, number, number];
    readonly twengine_set_table_border: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => [number, number, number];
    readonly twengine_set_table_cell_shading: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => [number, number, number];
    readonly twengine_set_track_changes: (a: number, b: number) => [number, number, number];
    readonly twengine_sign_document: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => [number, number, number];
    readonly twengine_sort_table_rows: (a: number, b: number, c: number, d: number) => [number, number, number];
    readonly twengine_spell_check: (a: number) => [number, number, number, number];
    readonly twengine_split_table_cell: (a: number, b: number, c: number) => [number, number, number];
    readonly twengine_text: (a: number) => [number, number];
    readonly twengine_text_in_range: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => [number, number];
    readonly twengine_undo: (a: number) => [number, number, number];
    readonly twengine_verify_signatures_json: (a: number) => [number, number];
    readonly __wbindgen_exn_store: (a: number) => void;
    readonly __externref_table_alloc: () => number;
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __externref_table_dealloc: (a: number) => void;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
