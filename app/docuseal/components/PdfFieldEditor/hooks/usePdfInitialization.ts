import {  useEffect, useRef } from 'react';
import { Field } from '../types';

export const usePdfInitialization = (
  template: any,
  pageWidth: number,
  pageHeight: number,
  isInitialized: boolean,
  setIsInitialized: React.Dispatch<React.SetStateAction<boolean>>,
  setFields: React.Dispatch<React.SetStateAction<Field[]>>,
  setOriginalFields: React.Dispatch<React.SetStateAction<Record<number, any>>>,
  setPartners: React.Dispatch<React.SetStateAction<string[]>>,
  setCurrentPartner: React.Dispatch<React.SetStateAction<string>>,
  setDeletedIds: React.Dispatch<React.SetStateAction<Set<number>>>,
  mobilePdfDimensions: { width: number; height: number }[],
  deletedIds: Set<number>,
  isPdfLoaded: boolean
) => {
  const initialTemplateIdRef = useRef<number | null>(null);
  const initialFieldsLengthRef = useRef<number>(0);

  useEffect(() => {
    const uniqueFields = Array.from(
      new Map((template.fields || []).map((f: any) => [f.id, f])).values()
    );

    if (uniqueFields.length !== template.fields?.length) {
      console.warn('⚠️ Duplicate fields detected! Original:', template.fields?.length, 'Unique:', uniqueFields.length);

      // Log field names to identify duplicates
      const fieldNames = (template.fields || []).map((f: any) => f.name);
      const nameCounts: Record<string, number> = {};
      fieldNames.forEach(name => {
        nameCounts[name] = (nameCounts[name] || 0) + 1;
      });
      const duplicateNames = Object.entries(nameCounts).filter(([_, count]) => count > 1);
      console.warn('Duplicate field names:', duplicateNames);
    }

    const initialFields = uniqueFields.map((f: any) => {
      // Convert position from pixels to decimal (0-1) if needed
      let position = f.position;
      if (f.position && typeof f.position.x === 'number') {
        const pageW = pageWidth || 600;
        const pageH = pageHeight || 800;
        
        // Check if position is in pixels (values > 1) or already in decimal (0-1)
        if (f.position.x > 1 || f.position.y > 1 || f.position.width > 1 || f.position.height > 1) {
          // Position is in pixels, convert to decimal (0-1)
          position = {
            ...f.position,
            x: f.position.x / pageW,
            y: f.position.y / pageH,
            width: f.position.width / pageW,
            height: f.position.height / pageH
          };
          console.log('Converted position for field', f.name, 'from pixels to decimal:', f.position, '->', position);
        } else {
          // Already in decimal format (0-1)
          console.log('Position already in decimal format for field', f.name, ':', position);
        }
      }
      
      return {
        ...f,
        tempId: `field-${f.id}`,
        options: f.options || (f.field_type === 'radio' ? ['Option 1', 'Option 2'] : f.field_type === 'multiple' ? ['Option 1', 'Option 2', 'Option 3'] : f.field_type === 'select' ? ['Option 1', 'Option 2', 'Option 3'] : f.field_type === 'cells' ? { columns: 3, widths: [1,1,1] } : f.options),
        position: position
      };
    });

    setFields(initialFields);
    setOriginalFields(Object.fromEntries(uniqueFields.map((f: any) => {
      // Convert position from pixels to decimal (0-1) if needed
      let position = f.position;
      if (f.position && typeof f.position.x === 'number') {
        const pageW = pageWidth || 600;
        const pageH = pageHeight || 800;
        
        // Check if position is in pixels (values > 1) or already in decimal (0-1)
        if (f.position.x > 1 || f.position.y > 1 || f.position.width > 1 || f.position.height > 1) {
          // Position is in pixels, convert to decimal (0-1)
          position = {
            ...f.position,
            x: f.position.x / pageW,
            y: f.position.y / pageH,
            width: f.position.width / pageW,
            height: f.position.height / pageH
          };
          console.log('Converted original position for field', f.name, 'from pixels to decimal:', f.position, '->', position);
        }
      }
      
      return [f.id, {
        ...f,
        options: f.options || (f.field_type === 'radio' ? ['Option 1', 'Option 2'] : f.field_type === 'multiple' ? ['Option 1', 'Option 2', 'Option 3'] : f.field_type === 'select' ? ['Option 1', 'Option 2', 'Option 3'] : f.field_type === 'cells' ? { columns: 3, widths: [1,1,1] } : f.options),
        position: position
      }];
    })));
    setDeletedIds(new Set());

    const uniquePartners = [...new Set(initialFields.map(f => f.partner).filter(Boolean))];
    const initialPartners = uniquePartners.length > 0 ? uniquePartners : ['First Party'];
    setPartners(initialPartners);
    setCurrentPartner(initialPartners[0]);


    // Mark as initialized for this template
    initialTemplateIdRef.current = template.id;
    initialFieldsLengthRef.current = initialFields.length;
    setIsInitialized(true);

  }, [template.id, pageWidth, pageHeight, isInitialized, isPdfLoaded]);

  return {
    initialTemplateIdRef,
    initialFieldsLengthRef
  };
};