import React, {useEffect, useState} from "react";
import '@assets/main-pane.css'
import * as api from '@/api'
import { Setting } from "@/bindings.ts";
import { FontAwesomeIcon as Icon } from '@fortawesome/react-fontawesome'
import { faCirclePlay, faFolder, faArrowRotateRight } from '@fortawesome/free-solid-svg-icons'
import { open } from '@tauri-apps/plugin-dialog';
import { useSettingPathStore } from "@store/settingPathStore.ts";
import SettingView from "@components/SettingView.tsx";
import { listen } from "@tauri-apps/api/event";
import {DndContext, DragStartEvent, DragEndEvent, useDroppable} from '@dnd-kit/core';
import {arrayMove, horizontalListSortingStrategy, SortableContext} from "@dnd-kit/sortable";
import DraggableStep from './DraggableStep.tsx';
import DroppableStep from './DroppableStep.tsx';

export type StepNotify = { name: string; status: string; message: string }



function MainPane(): React.JSX.Element {
  let [setting, setSetting] = useState<Setting | undefined>(undefined);
  const settingPath = useSettingPathStore((state) => state.settingPath);
  const setSettingPath = useSettingPathStore((state) => state.setSettingPath);
  const [stepProgressNotify, setStepProgressNotify] = useState<StepNotify | undefined>(undefined);
  const [stepStatusNotify, setStepStatusNotify] = useState<StepNotify | undefined>(undefined);
  const [activeStep, setActiveStep] = useState<string | null>(null);
  const [dropSteps, setDropSteps] = useState<string[]>([]);

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

  const runSelectedSteps = async (): Promise<void> => {
    console.log("MainPane.runSelectedSteps");
    for (const v of dropSteps) {
      const stepName = v.split(":").slice(1).join(":");
      await api.runStep(stepName);
    }
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


  useEffect(() => {
    const unlistenProgress = listen<StepNotify>("progress", (event) => {
      setStepProgressNotify(event.payload);
    });

    const unlistenStatus = listen<StepNotify>("status", (event) => {
      setStepStatusNotify(event.payload);
    });

    return () => {
      unlistenProgress.then((f) => f());
      unlistenStatus.then((f) => f());
    };
  }, []);


  function handleDragStart(event: DragStartEvent) {
    console.log(event);
    if (event.active) {
      setActiveStep(String(event.active.id));
    }
  }

  function handleDragEnd(event: DragEndEvent) {
    console.log(activeStep, dropSteps);
    const { active, over } = event;

    const stepName = active.id.toString().split(":").slice(1).join(":");
    let active_id = `drop:${stepName}`;
    if (!over || over.id === 'trash') {
      console.log('trash');
      setDropSteps((prev) => prev.filter((item) => item !== active_id));
      setActiveStep(null);
      return;
    }

    if (!over) {
      setActiveStep(null);
      return;
    }
    const overStepName = over.id.toString().split(":").slice(1).join(":");
    const over_id = `drop:${overStepName}`;
    const activeIndex = dropSteps.indexOf(active_id);
    const overIndex = dropSteps.indexOf(over_id);

    if (activeIndex !== -1 && overIndex !== -1 && active_id !== over_id) {
      setDropSteps((items) => arrayMove(items, activeIndex, overIndex));
    }
    else if (activeIndex === -1) {
      const newSteps = [...dropSteps];
      if (overIndex === -1) {
        newSteps.push(String(active_id));
      } else {
        newSteps.splice(overIndex, 0, active_id);
      }
      setDropSteps(newSteps);
    }

    setActiveStep(null);
  }


  return (
    <div className="main-pane">
      <div className="top">
        <div className="title">
          <h2>Crawler</h2>
        </div>
        <div className="control">
          {setting && (
            <DndContext onDragStart={handleDragStart} onDragEnd={handleDragEnd}>
              <div className="steps">
                {
                  Object.entries(setting.steps).map(([key, _step])  => {
                    const isDrop = dropSteps.some((v) => v.split(':').slice(1).join(':') == key);
                    return (
                      <DraggableStep key={key} id={{type: 'source', stepName: key}} isDrop={isDrop} />
                    )
                  })
                }
              </div>
              <div className="selected_steps">
                <div className="run_all">
                  <div className="btn" onClick={() => runSelectedSteps()}><Icon icon={faCirclePlay} /></div>
                </div>
                <div className="run_steps">
                  <DroppableContainer id="target">
                    <SortableContext items={dropSteps} strategy={horizontalListSortingStrategy}>
                      <div className="dropable steps">
                        {dropSteps.length === 0 ? (
                          <p>Drag Step</p>
                        ): (
                          dropSteps.map((item) => (
                            <DroppableStep key={item} id={{type: 'drop', stepName: item.split(':').slice(1).join(':')}} />
                          ))
                        )}
                      </div>
                    </SortableContext>
                  </DroppableContainer>
                </div>
              </div>
            </DndContext>
          )}
          <div className="notify">
            {stepStatusNotify && (
              <>
                <div className="message">{stepStatusNotify.message}</div>
              </>
            )}
            {stepProgressNotify && (
                <>
                  <div className="message">{stepProgressNotify.message}</div>
                </>
            )}
          </div>
        </div>
      </div>
      <div className="load">
        <div className="btn" onClick={() => openSetting()}><Icon icon={faFolder} /></div>
        <div className="btn" onClick={() => loadJson()}><Icon icon={faArrowRotateRight} /></div>
        <div className="label">LoadJson - {settingPath}</div>
      </div>
      <div className="editor">
        <SettingView />
      </div>
    </div>
  )
}

function DroppableContainer({ id, children }: {
  id: string;
  children: React.ReactNode;
}) {
  const { setNodeRef, isOver } = useDroppable({ id });

  return (
    <div
      ref={setNodeRef}
      style={{
        // border: "1px dashed #aaa",
        // padding: "2px",
        // backgroundColor: isOver ? "#f0f8ff" : "#fafafa",
      }}
    >
      {children}
    </div>
  );
}

export default MainPane;
