
// This file was generated by [tauri-specta](https://github.com/oscartbeaumont/tauri-specta). Do not edit this file manually.

/** user-defined commands **/


export const commands = {
async greet(name: string) : Promise<string> {
    return await TAURI_INVOKE("greet", { name });
},
async getArgPath() : Promise<Result<string | null, ApiError>> {
    try {
    return { status: "ok", data: await TAURI_INVOKE("get_arg_path") };
} catch (e) {
    if(e instanceof Error) throw e;
    else return { status: "error", error: e  as any };
}
},
async readTxt(pathStr: string) : Promise<Result<TextContent, ApiError>> {
    try {
    return { status: "ok", data: await TAURI_INVOKE("read_txt", { pathStr }) };
} catch (e) {
    if(e instanceof Error) throw e;
    else return { status: "error", error: e  as any };
}
},
async loadSetting(setting: Setting) : Promise<Result<null, ApiError>> {
    try {
    return { status: "ok", data: await TAURI_INVOKE("load_setting", { setting }) };
} catch (e) {
    if(e instanceof Error) throw e;
    else return { status: "error", error: e  as any };
}
},
async runStep(stepName: string) : Promise<Result<null, ApiError>> {
    try {
    return { status: "ok", data: await TAURI_INVOKE("run_step", { stepName }) };
} catch (e) {
    if(e instanceof Error) throw e;
    else return { status: "error", error: e  as any };
}
},
async saveSetting(filePath: string, txt: string) : Promise<Result<null, ApiError>> {
    try {
    return { status: "ok", data: await TAURI_INVOKE("save_setting", { filePath, txt }) };
} catch (e) {
    if(e instanceof Error) throw e;
    else return { status: "error", error: e  as any };
}
},
async updateState(stepName: string, val: number) : Promise<Result<null, ApiError>> {
    try {
    return { status: "ok", data: await TAURI_INVOKE("update_state", { stepName, val }) };
} catch (e) {
    if(e instanceof Error) throw e;
    else return { status: "error", error: e  as any };
}
}
}

/** user-defined events **/



/** user-defined constants **/



/** user-defined types **/

export type ApiError = { ScrapingError: string } | { TemplateError: string } | { ReqwestError: string } | { Io: string } | { ParseError: string } | { JsonError: string } | { GlobError: string }
export type CsvJob = { keys: string[]; sep: string; filename: string; output: string }
export type HtmlJob = { json_map: Partial<{ [key in string]: ([string, string])[] }>; output_template_file: string; output_template: string | null; filename: string; output: string }
export type HttpJob = { url: string; method: string; header: Partial<{ [key in string]: string }>; filename: string; output: string }
export type IterGlobJsonPattern = { glob_pattern: string; item_pattern: string; env_pattern: Partial<{ [key in string]: string }> }
export type IterJsonRangePattern = { name: string; file_pattern: string; offset_pattern: string; take_pattern: string }
export type IterList = { name: string; val: string[] }
export type IterPattern = { name: string; glob_pattern: string; content_pattern: string }
export type IterRange = { name: string; offset: string; take: string }
export type IterRangePattern = { name: string; glob_pattern: string; offset: string; take: string }
export type Job = { HttpJob: HttpJob } | { HtmlJob: HtmlJob } | { ShellJob: ShellJob } | { CsvJob: CsvJob }
export type Setting = { env: Partial<{ [key in string]: string }>; header: Partial<{ [key in string]: string }>; steps: Partial<{ [key in string]: Step }> }
export type ShellJob = { shell: string; args: string[]; working_dir: string; encoding: string }
export type Step = { name: string; task_iters: TaskIter[]; job: Job; concurrency_limit: number }
export type TaskIter = { Range: IterRange } | { Pattern: IterPattern } | { RangePattern: IterRangePattern } | { Vec: IterList } | { GlobJsonPattern: IterGlobJsonPattern } | { GlobJsonRangePattern: IterJsonRangePattern }
export type TextContent = { path: string; mimetype: string; enc?: string | null; text?: string | null }

/** tauri-specta globals **/

import {
	invoke as TAURI_INVOKE,
	Channel as TAURI_CHANNEL,
} from "@tauri-apps/api/core";
import * as TAURI_API_EVENT from "@tauri-apps/api/event";
import { type WebviewWindow as __WebviewWindow__ } from "@tauri-apps/api/webviewWindow";

type __EventObj__<T> = {
	listen: (
		cb: TAURI_API_EVENT.EventCallback<T>,
	) => ReturnType<typeof TAURI_API_EVENT.listen<T>>;
	once: (
		cb: TAURI_API_EVENT.EventCallback<T>,
	) => ReturnType<typeof TAURI_API_EVENT.once<T>>;
	emit: null extends T
		? (payload?: T) => ReturnType<typeof TAURI_API_EVENT.emit>
		: (payload: T) => ReturnType<typeof TAURI_API_EVENT.emit>;
};

export type Result<T, E> =
	| { status: "ok"; data: T }
	| { status: "error"; error: E };

function __makeEvents__<T extends Record<string, any>>(
	mappings: Record<keyof T, string>,
) {
	return new Proxy(
		{} as unknown as {
			[K in keyof T]: __EventObj__<T[K]> & {
				(handle: __WebviewWindow__): __EventObj__<T[K]>;
			};
		},
		{
			get: (_, event) => {
				const name = mappings[event as keyof T];

				return new Proxy((() => {}) as any, {
					apply: (_, __, [window]: [__WebviewWindow__]) => ({
						listen: (arg: any) => window.listen(name, arg),
						once: (arg: any) => window.once(name, arg),
						emit: (arg: any) => window.emit(name, arg),
					}),
					get: (_, command: keyof __EventObj__<any>) => {
						switch (command) {
							case "listen":
								return (arg: any) => TAURI_API_EVENT.listen(name, arg);
							case "once":
								return (arg: any) => TAURI_API_EVENT.once(name, arg);
							case "emit":
								return (arg: any) => TAURI_API_EVENT.emit(name, arg);
						}
					},
				});
			},
		},
	);
}
