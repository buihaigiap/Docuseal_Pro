import { useState, useEffect, useCallback } from 'react';
import { Field, DragState, InitialPosition } from '../types';

export const useTouchInteractions = (
  touchEnabled: boolean,
  isMobile: boolean,
  fields: Field[],
  selectedFieldTempId: string | null,
  setSelectedFieldTempId: React.Dispatch<React.SetStateAction<string | null>>,
  setActiveTool: React.Dispatch<React.SetStateAction<any>>,
  updateField: (tempId: string, updates: Partial<Field>) => void,
  mobilePdfDimensions: { width: number; height: number }[]
) => {
  const [isDragging, setIsDragging] = useState(false);
  const [isResizing, setIsResizing] = useState(false);
  const [dragStart, setDragStart] = useState<DragState>({ x: 0, y: 0 });
  const [initialPosition, setInitialPosition] = useState<InitialPosition>({ x: 0, y: 0, width: 0, height: 0 });
  const [resizeHandle, setResizeHandle] = useState<string | null>(null);

  const handleTouchStart = (e: React.TouchEvent, tempId: string, isResizeHandle?: boolean, handle?: string) => {
    if (!touchEnabled) return;
    e.preventDefault();
    e.stopPropagation();
    const touch = e.touches[0];
    const field = fields.find(f => f.tempId === tempId);
    if (!field || !field.position) return;

    setSelectedFieldTempId(tempId);
    setActiveTool('cursor');

    setDragStart({ x: touch.clientX, y: touch.clientY });
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

  const handleTouchMove = useCallback((e: TouchEvent) => {
    if (!touchEnabled || (!isDragging && !isResizing) || !selectedFieldTempId) return;
    e.preventDefault();
    const touch = e.touches[0];
    const deltaX = touch.clientX - dragStart.x;
    const deltaY = touch.clientY - dragStart.y;

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
    // Use original PDF dimensions for percentage calculations
    const pdfDimensions = mobilePdfDimensions[pageIndex];
    if (!pdfDimensions) return;

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
  }, [useTouchInteractions, isDragging, isResizing, selectedFieldTempId, dragStart, initialPosition, resizeHandle, fields, updateField, mobilePdfDimensions]);

  const handleTouchEnd = useCallback(() => {
    setIsDragging(false);
    setIsResizing(false);
    setResizeHandle(null);
  }, []);

  // Attach global touch event listeners for touch devices
  useEffect(() => {
    if (!useTouchInteractions) return; // Only for touch devices

    document.addEventListener('touchmove', handleTouchMove, { passive: false });
    document.addEventListener('touchend', handleTouchEnd);

    return () => {
      document.removeEventListener('touchmove', handleTouchMove);
      document.removeEventListener('touchend', handleTouchEnd);
    };
  }, [useTouchInteractions, handleTouchMove, handleTouchEnd]);

  return {
    isDragging,
    isResizing,
    handleTouchStart,
    handleTouchEnd
  };
};