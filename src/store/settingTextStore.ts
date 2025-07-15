import { create } from 'zustand'


export interface SettingTextStore {
  settingText: string | undefined
  setSettingText: (settingText?: string) => void
}

export const useSettingTextStore = create<SettingTextStore>((set) => ({
  settingText: undefined,
  setSettingText: (settingText?: string) => set(() => ({ settingText }))
}))
