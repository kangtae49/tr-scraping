import React from 'react';
import { useSortable } from "@dnd-kit/sortable";
import {DragStep, STEP_PAUSED, STEP_RUNNING, STEP_STOPPED} from "@/models.ts";
import {CSS} from "@dnd-kit/utilities";
import * as api from "@/api.ts";
import {FontAwesomeIcon as Icon} from "@fortawesome/react-fontawesome";
import {faCircleChevronRight, faCirclePause, faCirclePlay, faCircleStop} from "@fortawesome/free-solid-svg-icons";

type Props = {
  id: DragStep
}
function DroppableStep(props: Props) {
  const { attributes, listeners, setNodeRef, transform } = useSortable({
    id: `${props.id.type}:${props.id.stepName}`,
  });
  const style = {
    // Outputs `translate3d(x, y, 0)`
    transform: CSS.Translate.toString(transform),
  };

  // {
    // transform: transform
    //   ? `translate(${transform.x}px, ${transform.y}px)`
    //   : undefined,
    // border: "1px solid blue",
    // margin: "4px",
    // padding: "8px",
    // backgroundColor: "#d0f",
  // }

  return (
    <div className="step" ref={setNodeRef} {...attributes}  style={style}>
      <div className="btn" onClick={() => api.runStep(props.id.stepName)}><Icon icon={faCirclePlay} /></div>
      <div className="btn" onClick={() => api.updateState(props.id.stepName, STEP_STOPPED)}><Icon icon={faCircleStop} /></div>
      <div className="btn" onClick={() => api.updateState(props.id.stepName, STEP_PAUSED)}><Icon icon={faCirclePause} /></div>
      <div className="btn" onClick={() => api.updateState(props.id.stepName, STEP_RUNNING)}><Icon icon={faCircleChevronRight} /></div>
      <div className="label" {...listeners}>{props.id.stepName}</div>

    </div>
  );
}

export default DroppableStep;