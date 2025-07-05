import { create } from 'zustand'


export interface SettingPathStore {
  settingPath: string | undefined
  setSettingPath: (settingPath?: string) => void
}

export const useSettingPathStore = create<SettingPathStore>((set) => ({
  settingPath: undefined,
  setSettingPath: (settingPath?: string) => set(() => ({ settingPath }))
}))
