import { invoke } from "@tauri-apps/api/core";
import { Setting, TextContent } from "@/bindings.ts";

export const getArgPath = async (): Promise<string | undefined> => {
  return await invoke("get_arg_path")
}

export const readTxt = async (pathStr: string): Promise<TextContent> => {
  return await invoke("read_txt", {pathStr})
}
export const loadSetting = async (setting: Setting): Promise<void> => {
  return await invoke("load_setting", {setting})
}

export const runStep = async (stepName: string): Promise<void> => {
  console.log('invoke run_step:', stepName)
  return await invoke("run_step", {stepName})
}


export const saveSetting = async (filePath: string, txt: string): Promise<void> => {
  console.log('invoke save_setting:', filePath, txt)
  return await invoke("save_setting", {filePath, txt})
}


export const updateState = async (stepName: string, val: number): Promise<void> => {
  console.log('updateState:')
  return await invoke("update_state", {stepName, val})
}

