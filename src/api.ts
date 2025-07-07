import { invoke } from "@tauri-apps/api/core";
import { Setting, TextContent } from "@/bindings.ts";

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

// export const stopStep = async (stepName: string): Promise<void> => {
//   console.log('invoke stop_step:', stepName)
//   return await invoke("stop_step", {stepName})
// }

export const saveSetting = async (filePath: string, txt: string): Promise<void> => {
  console.log('invoke save_setting:', filePath, txt)
  return await invoke("save_setting", {filePath, txt})
}

export const stopStep = async (stepName: string): Promise<void> => {
  console.log('stopStep:')
  return await invoke("stop_step", {stepName})
}

export const getStopStep = async (stepName: string): Promise<boolean> => {
  console.log('getStopStep:')
  return await invoke("get_stop_step", {stepName})
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