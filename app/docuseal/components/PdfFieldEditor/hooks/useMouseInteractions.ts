import { useState, useEffect, useCallback } from 'react';
import { Field, DragState, InitialPosition } from '../types';

export const useMouseInteractions = (
  touchEnabled: boolean,
  isMobile: boolean,
  fields: Field[],
  selectedFieldTempId: string | null,
  updateField: (tempId: string, updates: Partial<Field>) => void,
  mobilePdfDimensions: { width: number; height: number }[]
) => {
  const [isDragging, setIsDragging] = useState(false);
  const [isResizing, setIsResizing] = useState(false);
  const [dragStart, setDragStart] = useState<DragState>({ x: 0, y: 0 });
  const [initialPosition, setInitialPosition] = useState<InitialPosition>({ x: 0, y: 0, width: 0, height: 0 });
  const [resizeHandle, setResizeHandle] = useState<string | null>(null);

  const handleMouseDown = (e: React.MouseEvent, tempId: string, isResizeHandle?: boolean, handle?: string) => {
    if (touchEnabled) return; // Touch devices use touch events
    e.preventDefault();
    const field = fields.find(f => f.tempId === tempId);
    if (!field || !field.position) return;

    setDragStart({ x: e.clientX, y: e.clientY });
    setInitialPosition({
      x: field.position.x,
      y: field.position.y,
      width: field.position.width,
      height: field.position.height
    });

    if (isResizeHandle && handle) {
      setIsResizing(true);
      setResizeHandle(handle);
    } else {
      setIsDragging(true);
    }
  };

  const handleMouseMove = useCallback((e: MouseEvent) => {
    if (touchEnabled || (!isDragging && !isResizing) || !selectedFieldTempId) return;
    e.preventDefault();
    const deltaX = e.clientX - dragStart.x;
    const deltaY = e.clientY - dragStart.y;

    // Find the current page container based on the selected field's page
    const field = fields.find(f => f.tempId === selectedFieldTempId);
    if (!field || !field.position) return;

    const pageIndex = field.position.page;
    const pageContainers = document.querySelectorAll('.relative.border.border-gray-300.rounded-lg.overflow-hidden');
    const currentPageElement = pageContainers[pageIndex] as Element;

    if (!currentPageElement) return;
    const imgElement = currentPageElement.querySelector('img');
    if (!imgElement) return;

    const rect = imgElement.getBoundingClientRect();
    const percentDeltaX = (deltaX / rect.width) * 100;
    const percentDeltaY = (deltaY / rect.height) * 100;

    if (isDragging) {
      const newX = Math.max(0, Math.min(100 - initialPosition.width, initialPosition.x + percentDeltaX));
      const newY = Math.max(0, Math.min(100 - initialPosition.height, initialPosition.y + percentDeltaY));

      updateField(selectedFieldTempId, {
        position: {
          ...field.position,
          x: newX,
          y: newY
        }
      });
    } else if (isResizing && resizeHandle) {
      let newX = initialPosition.x;
      let newY = initialPosition.y;
      let newWidth = initialPosition.width;
      let newHeight = initialPosition.height;

      if (resizeHandle.includes('e')) {
        newWidth = Math.max(5, Math.min(100 - initialPosition.x, initialPosition.width + percentDeltaX));
      }
      if (resizeHandle.includes('s')) {
        newHeight = Math.max(2, Math.min(100 - initialPosition.y, initialPosition.height + percentDeltaY));
      }
      if (resizeHandle.includes('w')) {
        const delta = Math.min(initialPosition.width - 5, percentDeltaX);
        newWidth = initialPosition.width - delta;
        newX = initialPosition.x + delta;
        newX = Math.max(0, newX);
        newWidth = Math.max(5, newWidth);
      }
      if (resizeHandle.includes('n')) {
        const delta = Math.min(initialPosition.height - 2, percentDeltaY);
        newHeight = initialPosition.height - delta;
        newY = initialPosition.y + delta;
        newY = Math.max(0, newY);
        newHeight = Math.max(2, newHeight);
      }

      updateField(selectedFieldTempId, {
        position: {
          ...field.position,
          x: newX,
          y: newY,
          width: newWidth,
          height: newHeight
        }
      });
    }
  }, [touchEnabled, isDragging, isResizing, selectedFieldTempId, dragStart, initialPosition, resizeHandle, fields, updateField]);

  const handleMouseUp = useCallback(() => {
    if (touchEnabled) return; // Touch devices use touch events
    setIsDragging(false);
    setIsResizing(false);
    setResizeHandle(null);
  }, [touchEnabled]);

  // Attach global mouse event listeners for mobile layout on non-touch devices
  useEffect(() => {
    if (touchEnabled || !isMobile) return; // Only for mobile layout on non-touch devices

    document.addEventListener('mousemove', handleMouseMove);
    document.addEventListener('mouseup', handleMouseUp);

    return () => {
      document.removeEventListener('mousemove', handleMouseMove);
      document.removeEventListener('mouseup', handleMouseUp);
    };
  }, [touchEnabled, isMobile, handleMouseMove, handleMouseUp]);

  return {
    isDragging,
    isResizing,
    handleMouseDown,
    handleMouseUp
  };
};