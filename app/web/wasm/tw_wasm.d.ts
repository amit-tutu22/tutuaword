/* tslint:disable */
/* eslint-disable */

/**
 * Browser engine handle (inline executor). Register fonts before opening documents.
 */
export class TwEngine {
    free(): void;
    [Symbol.dispose](): void;
    accept_all_revisions(): number;
    apply_bullet_list(caret_run_id: string): number;
    apply_heading1(caret_run_id: string): number;
    apply_normal_style(caret_run_id: string): number;
    apply_numbered_list(caret_run_id: string): number;
    atlas_bytes(): Uint8Array;
    atlas_generation(): number;
    atlas_height(): number;
    atlas_width(): number;
    caret_at(page: number, run_id: string, offset: number): string | undefined;
    caret_format_json(run_id: string): string | undefined;
    caret_geometry(page: number, x: number, y: number): string | undefined;
    clear_format(start_run: string, start_offset: number, end_run: string, end_offset: number): number;
    dispatch(data: Uint8Array): number;
    display_list_bytes(): Uint8Array;
    display_list_page_count(): number;
    display_list_page_height(): number;
    display_list_page_width(): number;
    display_list_version(): number;
    document_properties_json(): string;
    document_tail_hit(page: number): string | undefined;
    export_pdf(): Uint8Array;
    first_line_advance(): number;
    hit_test(page: number, x: number, y: number): string | undefined;
    insert_image(width: number, height: number): number;
    insert_page_break(caret_run_id: string): number;
    insert_table(rows: number, cols: number): number;
    is_page_stale(page: number): boolean;
    is_read_only(): boolean;
    last_error(): string;
    last_request_id(): number;
    constructor();
    new_document(): void;
    open_document(data: Uint8Array): void;
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
    save_document(): Uint8Array;
    save_document_as(extension: string): Uint8Array;
    selection_rects(page: number, start_x: number, start_y: number, end_x: number, end_y: number): Float32Array;
    set_current_page(page: number): number;
    set_track_changes(enabled: boolean): number;
    spell_check(): string;
    text(): string;
    text_in_range(start_run: string, start_offset: number, end_run: string, end_offset: number): string | undefined;
    undo(): number;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __wbg_twengine_free: (a: number, b: number) => void;
    readonly twengine_accept_all_revisions: (a: number) => [number, number, number];
    readonly twengine_apply_bullet_list: (a: number, b: number, c: number) => [number, number, number];
    readonly twengine_apply_heading1: (a: number, b: number, c: number) => [number, number, number];
    readonly twengine_apply_normal_style: (a: number, b: number, c: number) => [number, number, number];
    readonly twengine_apply_numbered_list: (a: number, b: number, c: number) => [number, number, number];
    readonly twengine_atlas_bytes: (a: number) => [number, number];
    readonly twengine_atlas_generation: (a: number) => number;
    readonly twengine_atlas_height: (a: number) => number;
    readonly twengine_atlas_width: (a: number) => number;
    readonly twengine_caret_at: (a: number, b: number, c: number, d: number, e: number) => [number, number];
    readonly twengine_caret_format_json: (a: number, b: number, c: number) => [number, number];
    readonly twengine_caret_geometry: (a: number, b: number, c: number, d: number) => [number, number];
    readonly twengine_clear_format: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => [number, number, number];
    readonly twengine_dispatch: (a: number, b: number, c: number) => [number, number, number];
    readonly twengine_display_list_bytes: (a: number) => [number, number];
    readonly twengine_display_list_page_count: (a: number) => number;
    readonly twengine_display_list_page_height: (a: number) => number;
    readonly twengine_display_list_page_width: (a: number) => number;
    readonly twengine_display_list_version: (a: number) => number;
    readonly twengine_document_properties_json: (a: number) => [number, number];
    readonly twengine_document_tail_hit: (a: number, b: number) => [number, number];
    readonly twengine_export_pdf: (a: number) => [number, number, number, number];
    readonly twengine_first_line_advance: (a: number) => number;
    readonly twengine_hit_test: (a: number, b: number, c: number, d: number) => [number, number];
    readonly twengine_insert_image: (a: number, b: number, c: number) => [number, number, number];
    readonly twengine_insert_page_break: (a: number, b: number, c: number) => [number, number, number];
    readonly twengine_insert_table: (a: number, b: number, c: number) => [number, number, number];
    readonly twengine_is_page_stale: (a: number, b: number) => number;
    readonly twengine_is_read_only: (a: number) => number;
    readonly twengine_last_error: (a: number) => [number, number];
    readonly twengine_last_request_id: (a: number) => number;
    readonly twengine_new: () => number;
    readonly twengine_new_document: (a: number) => [number, number];
    readonly twengine_open_document: (a: number, b: number, c: number) => [number, number];
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
    readonly twengine_save_document: (a: number) => [number, number, number, number];
    readonly twengine_save_document_as: (a: number, b: number, c: number) => [number, number, number, number];
    readonly twengine_selection_rects: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number];
    readonly twengine_set_current_page: (a: number, b: number) => [number, number, number];
    readonly twengine_set_track_changes: (a: number, b: number) => [number, number, number];
    readonly twengine_spell_check: (a: number) => [number, number, number, number];
    readonly twengine_text: (a: number) => [number, number];
    readonly twengine_text_in_range: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => [number, number];
    readonly twengine_undo: (a: number) => [number, number, number];
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
