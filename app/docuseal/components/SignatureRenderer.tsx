import React, { useRef, useEffect } from 'react';

interface SignatureRendererProps {
  data: string; // JSON string of point groups or typed text
  width?: number;
  height?: number;
  fieldType?: string;
}

const SignatureRenderer: React.FC<SignatureRendererProps> = ({ 
  data, 
  width = 200, 
  height = 100,
  fieldType
}) => {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  
  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    // Clear canvas
    ctx.clearRect(0, 0, width, height);

    try {
      const pointGroups = JSON.parse(data);
      
      if (!pointGroups || pointGroups.length === 0) {
        throw new Error('Empty data');
      }

      // It's vector data, render as before
      // Find bounds of signature to scale it properly
      let minX = Infinity, minY = Infinity;
      let maxX = -Infinity, maxY = -Infinity;

      pointGroups.forEach((group: any[]) => {
        group.forEach((point: any) => {
          minX = Math.min(minX, point.x);
          minY = Math.min(minY, point.y);
          maxX = Math.max(maxX, point.x);
          maxY = Math.max(maxY, point.y);
        });
      });

      const signatureWidth = maxX - minX;
      const signatureHeight = maxY - minY;
      
      // Calculate scale to fit signature in canvas with padding
      const padding = 10;
      const scaleX = (width - padding * 2) / signatureWidth;
      const scaleY = (height - padding * 2) / signatureHeight;
      const scale = Math.min(scaleX, scaleY, 1); // Don't scale up, only down

      // Calculate offset to center signature
      const offsetX = (width - signatureWidth * scale) / 2 - minX * scale;
      const offsetY = (height - signatureHeight * scale) / 2 - minY * scale;

      // Draw signature
      ctx.strokeStyle = '#000000';
      ctx.lineWidth = 2;
      ctx.lineCap = 'round';
      ctx.lineJoin = 'round';

      pointGroups.forEach((group: any[]) => {
        if (group.length === 0) return;

        ctx.beginPath();
        group.forEach((point: any, index: number) => {
          const x = point.x * scale + offsetX;
          const y = point.y * scale + offsetY;

          if (index === 0) {
            ctx.moveTo(x, y);
          } else {
            ctx.lineTo(x, y);
          }
        });
        ctx.stroke();
      });
    } catch (e) {
      // If not JSON or empty, treat as typed text
      if (fieldType === 'initials') {
        const scale = 3;
        const scaledWidth = width * scale;
        const scaledHeight = height * scale;
        
        // Set canvas to scaled size for better quality
        canvas.width = scaledWidth;
        canvas.height = scaledHeight;
        ctx.scale(scale, scale); // Apply scale
        
        const fontFamily = 'Arial';
        const fontStyle = 'italic';
        const fontWeight = 'normal';
        
        // Start with a large font size
        let fontSize = height * 1.5;
        ctx.font = `${fontStyle} ${fontWeight} ${fontSize}px ${fontFamily}`;
        
        // Measure actual text bounds
        let metrics = ctx.measureText(data);
        let actualHeight = metrics.actualBoundingBoxAscent + metrics.actualBoundingBoxDescent;
        
        // Scale font to fill height
        fontSize = (height / actualHeight) * fontSize;
        ctx.font = `${fontStyle} ${fontWeight} ${fontSize}px ${fontFamily}`;
        metrics = ctx.measureText(data);
        actualHeight = metrics.actualBoundingBoxAscent + metrics.actualBoundingBoxDescent;
        
        // Check width and reduce if needed
        let textWidth = metrics.width;
        if (textWidth > width) {
          fontSize = (width / textWidth) * fontSize;
          ctx.font = `${fontStyle} ${fontWeight} ${fontSize}px ${fontFamily}`;
          metrics = ctx.measureText(data);
          actualHeight = metrics.actualBoundingBoxAscent + metrics.actualBoundingBoxDescent;
        }
        
        ctx.fillStyle = '#000000';
        ctx.textAlign = 'center';
        ctx.textBaseline = 'alphabetic';
        
        // Clear and draw
        ctx.clearRect(0, 0, width, height);
        
        // Calculate Y to center text vertically, then push up for bottom
        const centerY = (height - actualHeight) / 2 + metrics.actualBoundingBoxAscent - (height * 0.01);
        
        ctx.fillText(data, width / 2, centerY);
      } else {
        // Default text rendering for signatures
        ctx.fillStyle = '#000000';
        ctx.font = `${Math.min(Math.max(width / 5, 12), height)}px sans-serif`;
        ctx.textAlign = 'center';
        ctx.textBaseline = 'middle';
        ctx.fillText(data || '', width / 2, height / 2);
      }
    }
  }, [data, width, height, fieldType]);

  return (
    <canvas 
      ref={canvasRef} 
      width={width} 
      height={height}
      className="max-w-full max-h-full"
    />
  );
};

export default SignatureRenderer;