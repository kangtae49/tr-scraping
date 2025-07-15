import React, {useEffect, useRef, useState} from "react";
import * as api from '@/api'
import * as monaco from 'monaco-editor'
import {Setting} from "@/bindings.ts";
import {saveSetting} from "@/api";
import { useSettingPathStore } from "@store/settingPathStore.ts";
import {useSettingTextStore} from "@store/settingTextStore.ts";

self.MonacoEnvironment = {
  getWorkerUrl(_, label) {
    const basePath = '.'
    if (label === 'json') {
      return `${basePath}/monaco-editor/esm/vs/language/json/json.worker.js`
    }
    if (label === 'css') {
      return `${basePath}/monaco-editor/esm/vs/language/css/css.worker.js`
    }
    if (label === 'html') {
      return `${basePath}/monaco-editor/esm/vs/language/html/html.worker.js`
    }
    if (label === 'typescript' || label === 'javascript') {
      return `${basePath}/monaco-editor/esm/vs/language/typescript/ts.worker.js`
    }
    return `${basePath}/monaco-editor/esm/vs/editor/editor.worker.js`
  }
}

function SettingView(): React.JSX.Element {
  const settingPath = useSettingPathStore((state) => state.settingPath);
  const setSettingPath = useSettingPathStore((state) => state.setSettingPath);
  const settingText = useSettingTextStore((state) => state.settingText);
  const setSettingText = useSettingTextStore((state) => state.setSettingText);
  const editorRef = useRef<HTMLDivElement>(null);
  const monacoEditorRef = useRef<monaco.editor.IStandaloneCodeEditor | null>(null);
  // const [content, setContent] = useState<string | null>(null);

  // useEffect(() => {
  //   if (settingPath) {
  //     api
  //       .readTxt(settingPath)
  //       .then((txtContent) => {
  //         setContent(txtContent?.text || '')
  //       })
  //       .catch((e) => {
  //         console.error(e)
  //         setContent('')
  //       });
  //   }
  // }, [settingPath]);

  useEffect(() => {
    if (settingPath && settingText) {
      let setting = JSON.parse(settingText);
      const dirPath = settingPath.substring(0, settingPath.lastIndexOf('\\') + 1);
      const schemaFile = `${dirPath}\\${setting["$schema"]}`;
      console.log(settingPath);
      console.log(schemaFile);
      api
        .readTxt(schemaFile)
        .then((txtContent) => {
          if (txtContent.text) {
            let jsonSchema = JSON.parse(txtContent.text);
            monaco.languages.json.jsonDefaults.setDiagnosticsOptions({
              validate: true,
              schemas: [
                {
                  uri: 'inmemory://model/setting.schema.json',
                  fileMatch: ['*.json'],
                  schema: jsonSchema
                }
              ]
            });
          }

        })
        .catch((e) => {
          console.error(e)
        });
    }

  }, [settingText, settingPath])

  useEffect(() => {
    if ( settingText && editorRef && editorRef.current) {
      if (monacoEditorRef?.current) {
        monacoEditorRef.current.dispose()
      }
      monacoEditorRef.current = monaco.editor.create(editorRef.current, {
        // model,
        value: settingText,
        // language: 'plaintext',
        language: getMonacoLanguage("json"),
        theme: 'vs-dark',
        // readOnly: true,
        automaticLayout: true,
        scrollBeyondLastLine: false
      })
      monacoEditorRef.current.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyCode.KeyS, () => {
        const txt = monacoEditorRef.current?.getValue();
        if (txt && settingPath) {
          api.saveSetting(settingPath, txt).then(() => {
              setSettingText(txt);
              console.log("save ok", settingPath)
            })
            .catch((e) => {
              console.error(e)
            })
          ;
        }
      })
    }
  }, [settingText]);

  return <div className="view-monaco" ref={editorRef} />

}

export function getMonacoLanguage(ext?: string): string {
  let language = 'plaintext'
  if (!ext) {
    return 'plaintext'
  }
  const languages = monaco.languages.getLanguages()
  // console.log('languages', languages)
  const lang = languages.find((lang) => lang.extensions?.includes(`.${ext}`))
  if (lang) {
    language = lang.id
  }
  return language
}


export default SettingView;