import { invoke } from "@tauri-apps/api/core";
import {Edge, Setting, Step, TextContent} from "@/bindings.ts";

export const getArgPath = async (): Promise<string | undefined> => {
  return await invoke("get_arg_path")
}

export const readTxt = async (pathStr: string): Promise<TextContent> => {
  return await invoke("read_txt", {pathStr})
}
export const loadCrawler = async (setting: Setting): Promise<void> => {
  return await invoke("load_crawler", {setting})
}

export const runStep = async (stepName: string): Promise<void> => {
  return await invoke("run_step", {stepName})
}