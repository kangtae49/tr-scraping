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
  console.log('invoke run_step:', stepName)
  return await invoke("run_step", {stepName})
}

export const stopStep = async (stepName: string): Promise<void> => {
  console.log('invoke stop_step:', stepName)
  return await invoke("stop_step", {stepName})
}

export const saveSetting = async (filePath: string, txt: string) => {
  console.log('invoke save_setting:', filePath, txt)
  return await invoke("save_setting", {filePath, txt})
}
// export const stopOutputHtml = async (): Promise<void> => {
//   console.log('invoke stop_output_html:')
//   return await invoke("stop_output_html")
// }
//
// export const runOutputHtml = async (): Promise<void> => {
//   console.log('invoke run_output_html:')
//   return await invoke("run_output_html")
// }