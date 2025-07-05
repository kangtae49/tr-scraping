import React, {useEffect, useState} from "react";
import '@assets/main-pane.css'
import * as api from '@/api'
import {TextContent, Step, Edge, Setting} from "@/bindings.ts";
import { FontAwesomeIcon as Icon } from '@fortawesome/react-fontawesome'
import { faCirclePlay, faCirclePause, faFolder, faArrowRotateRight } from '@fortawesome/free-solid-svg-icons'
import { open } from '@tauri-apps/plugin-dialog';
import { useSettingPathStore } from "@store/settingPathStore.ts";
import SettingView from "@components/SettingView.tsx";

function MainPane(): React.JSX.Element {
  let [setting, setSetting] = useState<Setting | undefined>(undefined);
  const settingPath = useSettingPathStore((state) => state.settingPath);
  const setSettingPath = useSettingPathStore((state) => state.setSettingPath);

  const loadJson = async (): Promise<void> => {
    if (settingPath) {
      api.readTxt(settingPath).then((textContent) => {
        if (textContent.text) {
          // console.log(textContent.text);
          let setting = JSON.parse(textContent.text);
          api.loadCrawler(setting)
            .then(() => {
              setSetting(setting);
              console.info('loadCrawler');
            })
            .catch((reason) => {
              console.error(reason);
            })
        }
      });
    }
  }

  const runStep = async (stepName: string): Promise<void> => {
    console.log("MainPane.runStep", stepName);
    api.runStep(stepName).then(() => {})
      .catch(e => console.error(e.message))
  }

  const stopStep = async (stepName: string): Promise<void> => {
    console.log("MainPane.stopStep", stepName);
    api.stopStep(stepName).then(() => {})
      .catch(e => console.error(e.message))
  }


  const openSetting = async (): Promise<void> => {
    open({
      multiple: false,
      directory: false,
    }).then(path => {
      if (path) {
        setSettingPath(path);
      }
    });
  }

  useEffect(() => {
    if (setting) {
      console.log("steps: " , setting.steps);
    }
  }, [setting])

  useEffect(() => {
    api.getArgPath().then((path) => {
      if(path) {
        setSettingPath(path);
      }
    })
  }, []);

  return (
    <div className="main-pane">
      <div className="top">
        <div>
          <h2>Crawler</h2>
        </div>
        <div className="load">
          <div className="btn" onClick={() => openSetting()}><Icon icon={faFolder} /></div>
          <div className="btn" onClick={() => loadJson()}><Icon icon={faArrowRotateRight} /></div>
          <div className="label">LoadJson - {settingPath}</div>
        </div>
        {setting && (
          <div className="steps">
            {
              Object.entries(setting.steps).map(([key, _step])  => {
                return (
                  <div className="step" key={key}>
                    <div className="btn" onClick={() => runStep(key)}><Icon icon={faCirclePlay} /></div>
                    <div className="btn" onClick={() => stopStep(key)}><Icon icon={faCirclePause} /></div>
                    <div className="label">Run {key}</div>
                  </div>
                )
              })
            }
          </div>
        )}
      </div>
      <div className="editor">
        <SettingView />
      </div>
    </div>
  )
}

export default MainPane;
