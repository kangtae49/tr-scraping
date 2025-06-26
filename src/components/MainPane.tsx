import React, {useEffect} from "react";
import '@assets/main-pane.css'
import * as api from '@/api'
import {TextContent, Step, Edge, Setting} from "@/bindings.ts";

function MainPane(): React.JSX.Element {
  const runStep = async (stepName: string): Promise<void> => {
    console.log("MainPane.runStep", stepName);
    api.runStep(stepName).then(() => {})
      .catch(e => console.error(e.message))
  }
  useEffect(() => {
    api.getArgPath().then((path) => {
      if (path){
        console.info('path', path);
        api.readTxt(path).then((textContent) => {
          if (textContent.text) {
            // console.log(textContent.text);
            let setting = JSON.parse(textContent.text) as Setting;
            console.log(setting);
            // let env = setting.env;
            // let steps = setting.steps;
            // let edges: Edge[] = []
            api.loadCrawler(setting)
              .then(() => {
                console.info('loadCrawler');
                // runStep("step1").then(()=> {
                //   console.info('step0 ok');
                // })
              })
              .catch((reason) => {
                console.error(reason);
              })
            console.log(setting);
          }
        });
      }
    })
  })
  return (
    <div>
      <h2>Crawler</h2>
      <div onClick={() => runStep("step1")}>Run Step1</div>
      <div onClick={() => runStep("step2")}>Run Step2</div>
      <div onClick={() => runStep("step3")}>Run Step3</div>
    </div>
  )
}

export default MainPane;
