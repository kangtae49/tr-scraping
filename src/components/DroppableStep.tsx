import React from 'react';
import {useDroppable} from '@dnd-kit/core';
import {SortableContext, horizontalListSortingStrategy, useSortable} from "@dnd-kit/sortable";
import {FontAwesomeIcon as Icon} from "@fortawesome/react-fontawesome";
import {faCirclePlay, faCircleStop} from "@fortawesome/free-solid-svg-icons";
import * as api from "@/api.ts";
import { DragStep } from "@/models.ts";

type Props = {
  id: DragStep
}
function DroppableStep(props: Props) {
  const { attributes, listeners, setNodeRef, transform } = useSortable({
    id: `${props.id.type}:${props.id.stepName}`,
  });

  return (
    <div className="step" ref={setNodeRef} {...attributes} {...listeners} style={{
      transform: transform
        ? `translate(${transform.x}px, ${transform.y}px)`
        : undefined,
      border: "1px solid blue",
      // margin: "4px",
      // padding: "8px",
      // backgroundColor: "#d0f",
    }}>
      <div className="label">{props.id.stepName}</div>
    </div>
  );
}

export default DroppableStep;