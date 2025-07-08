import React from 'react';
import {useDraggable} from '@dnd-kit/core';
import {CSS} from '@dnd-kit/utilities';
import {FontAwesomeIcon as Icon} from "@fortawesome/react-fontawesome";
import {faCirclePlay, faCircleStop, faCirclePause, faCircleChevronRight} from "@fortawesome/free-solid-svg-icons";
import * as api from "@/api.ts";
import { DragStep, STEP_RUNNING, STEP_STOPPED, STEP_PAUSED } from "@/models.ts";

type Props = {
  id: DragStep
  isDrop: boolean
}
function DraggableStep(props: Props) {
  const {attributes, listeners, setNodeRef, transform} = useDraggable({
    id: `${props.id.type}:${props.id.stepName}`,
  });
  const style = {
    // Outputs `translate3d(x, y, 0)`
    transform: CSS.Translate.toString(transform),
  };


  return (
    <div className={`step ${props.isDrop ? "drop" : ""}`} ref={setNodeRef} style={style} {...attributes}>
      <div className="btn" onClick={() => api.runStep(props.id.stepName)}><Icon icon={faCirclePlay} /></div>
      <div className="btn" onClick={() => api.updateState(props.id.stepName, STEP_STOPPED)}><Icon icon={faCircleStop} /></div>
      <div className="btn" onClick={() => api.updateState(props.id.stepName, STEP_PAUSED)}><Icon icon={faCirclePause} /></div>
      <div className="btn" onClick={() => api.updateState(props.id.stepName, STEP_RUNNING)}><Icon icon={faCircleChevronRight} /></div>
      <div className="label" {...listeners} >{props.id.stepName}</div>
    </div>
  );
}

export default DraggableStep;